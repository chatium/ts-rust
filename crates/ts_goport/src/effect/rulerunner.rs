//! Port of Effect-TS/tsgo `internal/rulerunner`: runs the enabled rules on
//! one source file and applies the configured severities and directives.

use crate::effect::diag;
use crate::effect::directives::{
    Directive, DirectiveSet, build_directive_set, collect_effect_directives, to_category,
};
use crate::effect::etscore::{
    EffectPluginOptions, ResolvedEffectPluginOptions, Severity, diagnostics_enabled,
    is_command_line_mode,
};
use crate::effect::pluginoptions::{
    resolve_diagnostic_severity_for_file, resolve_effect_plugin_options_for_source_file,
};
use crate::effect::rule::{Rule, RuleContext, UNUSED_DIRECTIVE_NAME};
use crate::effect::rules;
use crate::effect::typeparser::TypeParser;
use crate::gostd::Context;
use crate::prelude::*;

/// A diagnostic and its rule, for directive processing.
struct RuleDiagnostic {
    rule: &'static Rule,
    diagnostic: Diagnostic,
}

// Go: rulerunner.MinVisibleSeverity
/// The least visible severity that can surface: in tsc mode without
/// `includeSuggestionsInTsc`, warnings; elsewhere (editor, tests), messages.
#[must_use]
pub fn min_visible_severity(effect_config: Option<&EffectPluginOptions>) -> Severity {
    let include_suggestions =
        effect_config.is_none_or(EffectPluginOptions::get_include_suggestions_in_tsc);
    if is_command_line_mode() && !include_suggestions {
        return Severity::Warning;
    }
    Severity::Message
}

// PORT: not in Effect-TS/tsgo, whose checkers run the rules in every mode,
// so its API answers carry Effect diagnostics. A standalone API process
// (`tsgo --api`) answers here as plain tsgo: no Effect rules and no Effect
// diagnostics (state note effectapi1-2026-10-07). An API session inside a
// language server shares the server's checkers, so it keeps the rules.
static API_RULES_OFF: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Marks this process as a standalone API process, where the Effect rules
/// and the plugin option check are off unless `TSGO_EFFECT_API=1` (read
/// once). `api::new_standalone_session` calls it.
pub fn set_api_process() {
    static EFFECT_API: std::sync::LazyLock<bool> =
        std::sync::LazyLock::new(|| std::env::var_os("TSGO_EFFECT_API").is_some_and(|v| v == "1"));
    API_RULES_OFF.store(!*EFFECT_API, std::sync::atomic::Ordering::Relaxed);
}

/// Test hook: undoes `set_api_process`, so this process runs the rules as
/// tsc does.
#[doc(hidden)]
pub fn clear_api_process() {
    API_RULES_OFF.store(false, std::sync::atomic::Ordering::Relaxed);
}

/// The Effect plugin options of `options` when this process runs the rules:
/// none in a standalone API process without `TSGO_EFFECT_API=1`
/// (`set_api_process`). The checker, the plugin option check and build info
/// (its version suffix and `effect` options, written and checked) all read
/// this. A standalone API build then writes and expects plain build info,
/// so a later tsc build checks the project again and reports the Effect
/// diagnostics.
#[must_use]
pub fn enabled_options(options: &CompilerOptions) -> Option<&EffectPluginOptions> {
    options
        .effect
        .as_deref()
        .filter(|_| !API_RULES_OFF.load(std::sync::atomic::Ordering::Relaxed))
}

// Go: rulerunner.Run
/// The Effect diagnostics of `sf`. `rule_names` None runs every rule.
pub fn run(
    ctx: &Context,
    program: &'static GoProgram,
    c: &mut Checker,
    sf: Node,
    effect_config: Option<&EffectPluginOptions>,
    rule_names: Option<&[&str]>,
    min_severity: Severity,
) -> Result<Vec<Diagnostic>, String> {
    if with_source_file_info(sf, |info| info.is_declaration_file)
        || is_source_file_from_external_library(sf)
    {
        return Ok(Vec::new());
    }
    let Some(effect_config) = effect_config else {
        return Ok(Vec::new());
    };

    let file_name = source_file_file_name(sf);
    let config_file_path = program.options.config_file_path.as_str();
    let case_sensitive = use_case_sensitive_file_names();
    let effective_config = resolve_effect_plugin_options_for_source_file(
        effect_config,
        file_name,
        config_file_path,
        case_sensitive,
    );
    let resolved_severity = resolve_diagnostic_severity_for_file(
        effect_config,
        file_name,
        config_file_path,
        case_sensitive,
    );
    let Some(resolved_severity) =
        resolved_severity.filter(|_| diagnostics_enabled(Some(effect_config)))
    else {
        return Ok(Vec::new());
    };

    let selected_rules = select_rules(rule_names)?;
    if selected_rules.is_empty() {
        return Ok(Vec::new());
    }

    let source_text = source_file_text(sf);
    let effect_directives = collect_effect_directives(&source_text);
    let mut directive_set = build_directive_set(&effect_directives);
    if directive_set.is_skip_file("*") {
        return Ok(Vec::new());
    }

    let mut tp = TypeParser::new(program, c);
    let all = collect_diagnostics(
        ctx,
        program,
        &mut tp,
        sf,
        effect_config,
        &effective_config,
        &resolved_severity,
        &directive_set,
        &selected_rules,
        min_severity,
    );
    let mut result = transform_diagnostics(
        all,
        sf,
        &mut directive_set,
        &resolved_severity,
        min_severity,
    );
    result.extend(unused_directive_diagnostics(
        sf,
        &effect_directives,
        &directive_set,
        &resolved_severity,
    ));
    Ok(result)
}

fn select_rules(rule_names: Option<&[&str]>) -> Result<Vec<&'static Rule>, String> {
    let Some(names) = rule_names else {
        return Ok(rules::ALL.to_vec());
    };
    names
        .iter()
        .map(|name| {
            rules::ALL
                .iter()
                .copied()
                .find(|r| r.name == *name)
                .ok_or_else(|| format!("unknown Effect diagnostic rule {name:?}"))
        })
        .collect()
}

fn severity_from_map(map: &IndexMap<String, Severity>, name: &str) -> Option<Severity> {
    map.get(name).copied()
}

#[allow(clippy::too_many_arguments)]
fn collect_diagnostics(
    ctx: &Context,
    program: &'static GoProgram,
    tp: &mut TypeParser<'_>,
    sf: Node,
    global_config: &EffectPluginOptions,
    options: &ResolvedEffectPluginOptions,
    resolved_severity: &IndexMap<String, Severity>,
    directive_set: &DirectiveSet,
    selected_rules: &[&'static Rule],
    min_severity: Severity,
) -> Vec<RuleDiagnostic> {
    let mut results = Vec::new();
    for &r in selected_rules {
        let config_severity =
            severity_from_map(resolved_severity, r.name).unwrap_or(r.default_severity);
        if !global_config.skip_disabled_optimization
            && config_severity.is_off()
            && !directive_set.has_enabling_directive(r.name)
        {
            continue;
        }
        // A rule below the minimum visible severity can only surface if a
        // directive raises it, and it must still run when any directive
        // references it so the directive is marked used.
        if !global_config.skip_disabled_optimization
            && !config_severity.at_least_as_visible_as(min_severity)
            && !directive_set.has_any_directive_for_rule(r.name)
        {
            continue;
        }
        if directive_set.is_skip_file(r.name) {
            continue;
        }
        let mut rule_ctx = RuleContext {
            context: ctx,
            program,
            tp: &mut *tp,
            source_file: sf,
            options,
            default_severity: r.default_severity,
        };
        for diagnostic in (r.run)(&mut rule_ctx) {
            results.push(RuleDiagnostic {
                rule: r,
                diagnostic,
            });
        }
    }
    results
}

fn transform_diagnostics(
    diags: Vec<RuleDiagnostic>,
    sf: Node,
    directive_set: &mut DirectiveSet,
    resolved_severity: &IndexMap<String, Severity>,
    min_severity: Severity,
) -> Vec<Diagnostic> {
    let line_map = get_ecma_line_starts(sf);
    let mut results = Vec::new();
    for rd in diags {
        let line = compute_line_of_position(&line_map, rd.diagnostic.pos);
        let default_severity =
            severity_from_map(resolved_severity, rd.rule.name).unwrap_or(rd.rule.default_severity);
        let effective = directive_set.get_effective_severity_and_mark_used(
            rd.rule.name,
            line,
            default_severity,
        );
        if effective.is_off() || !effective.at_least_as_visible_as(min_severity) {
            continue;
        }
        let mut diagnostic = rd.diagnostic;
        // Go makes a copy with the new category (createTransformedDiagnostic).
        diagnostic.category = to_category(effective);
        results.push(diagnostic);
    }
    results
}

fn unused_directive_diagnostics(
    sf: Node,
    all: &[Directive],
    directive_set: &DirectiveSet,
    resolved_severity: &IndexMap<String, Severity>,
) -> Vec<Diagnostic> {
    let severity =
        severity_from_map(resolved_severity, UNUSED_DIRECTIVE_NAME).unwrap_or(Severity::Warning);
    if severity.is_off() {
        return Vec::new();
    }
    let message = diag::X_effect_diagnostics_directive_has_no_effect;
    directive_set
        .get_unused_next_line_directives(all)
        .into_iter()
        .map(|d| {
            new_diagnostic_from_serialized(
                sf,
                TextRange::new(d.pos, d.end),
                i32::try_from(message.code()).unwrap_or(i32::MAX),
                to_category(severity),
                message.key(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                false,
                false,
                false,
            )
        })
        .collect()
}
