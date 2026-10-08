//! PORT: no Go counterpart (effectapi1). Effect-TS/tsgo runs the Effect
//! rules in every checker, so its API answers carry Effect diagnostics. A
//! port standalone API process (`api::new_standalone_session`, `tsgo --api`)
//! answers as plain tsgo, unless `TSGO_EFFECT_API=1`
//! (`effect::rulerunner::set_api_process`). The language server keeps the
//! rules, also for an API session inside it, which shares its checkers.
//!
//! The fixture needs no Effect package: `globalDate` (TS377068) flags
//! `new Date()`, and the plugin option check flags the unknown rule name
//! in the config diagnostics (`unknownRuleName`).
//!
//! A standalone API build without the rules writes plain build info, and
//! does not reuse build info with Effect diagnostics, so tsc and the API
//! each check a project again after the other (int50b skeptic problem 1).
//! The tests act as tsc in the same process with the flag reset
//! (`rulerunner::clear_api_process`).
//!
//! A test of the standalone API without the rules sets `TSGO_EFFECT_API=0`
//! in its child, so it also passes when the parent has `TSGO_EFFECT_API=1`.

use std::rc::Rc;

use ts_goport::api::{
    self, BuildParams, CreateBuildOrchestratorParams, CreateSnapshotParams,
    GetDefaultProjectForFileParams, GetDiagnosticsParams, GetProjectDiagnosticsParams,
    SnapshotRequestChangesParams,
};
use ts_goport::diagnostics::Category;
use ts_goport::effect::{diag, rulerunner};
use ts_goport::execute::incremental::build_info::build_info_version;
use ts_goport::execute::incremental::incremental::parse_build_info;
use ts_goport::execute::tsc::ExitStatus;
use ts_goport::ls::diagnostics::get_all_diagnostics;
use ts_goport::program::ls_program;

use super::api_util::{doc, nil_error};
use super::projecttestutil::{self, SessionUtils, TypingsInstallerOptions, files};
use super::util::{bg, open, program};

const CONFIG: &str = "/home/projects/p/tsconfig.json";
const INDEX: &str = "/home/projects/p/index.ts";
const CONFIG_TEXT: &str = r#"{"compilerOptions":{"strict":true,"plugins":[{"name":"@effect/language-service","diagnosticSeverity":{"globalDate":"error","noSuchRule":"error"}}]}}"#;
const INDEX_TEXT: &str = "export const value: string = 42;\nexport const date = new Date();\n";

const TS2322: i32 = 2322;
const GLOBAL_DATE: i32 = 377_068;

fn unknown_rule_name() -> i32 {
    let message =
        diag::Unknown_Effect_diagnostic_rule_0_in_diagnosticSeverity_effect_unknownRuleName;
    i32::try_from(message.code()).expect("an i32 code")
}

/// The codes of the semantic diagnostics of index.ts and of the config
/// diagnostics, as `session` answers them.
fn api_codes(session: &api::Session) -> (Vec<i32>, Vec<i32>) {
    let ctx = bg();
    let snapshot = nil_error(session.handle_create_snapshot(
        &ctx,
        &CreateSnapshotParams {
            snapshot_request_changes_params: SnapshotRequestChangesParams {
                open_projects: vec![doc(CONFIG)],
                ..Default::default()
            },
            ..Default::default()
        },
    ))
    .snapshot;
    let project = nil_error(session.handle_get_default_project_for_file(
        &ctx,
        &GetDefaultProjectForFileParams {
            snapshot,
            file: doc(INDEX),
        },
    ))
    .expect("a default project")
    .id;
    let semantic = nil_error(session.handle_get_semantic_diagnostics(
        &ctx,
        &GetDiagnosticsParams {
            snapshot,
            project: project.clone(),
            files: Some(vec![doc(INDEX)]),
        },
    ));
    let config = nil_error(session.handle_get_config_file_parsing_diagnostics(
        &ctx,
        &GetProjectDiagnosticsParams { snapshot, project },
    ));
    (
        semantic.iter().map(|d| d.code).collect(),
        config.iter().map(|d| d.code).collect(),
    )
}

/// `api_codes` for a standalone API session on the fixture.
fn standalone_codes() -> (Vec<i32>, Vec<i32>) {
    let (init, _) = projecttestutil::get_session_init_options(
        files(&[(CONFIG, CONFIG_TEXT), (INDEX, INDEX_TEXT)]),
        None,
        TypingsInstallerOptions::default(),
    );
    let session = api::new_standalone_session(&init, None);
    let codes = api_codes(&session);
    session.close();
    codes
}

child_test! {
    env &[("TSGO_EFFECT_API", "0")];
    fn standalone_api_answers_without_effect_rules() {
        assert_eq!(standalone_codes(), (vec![TS2322], vec![]));
    }
}

child_test! {
    env &[("TSGO_EFFECT_API", "1")];
    fn standalone_api_with_effect_api_answers_as_the_reference() {
        assert_eq!(
            standalone_codes(),
            (vec![TS2322, GLOBAL_DATE], vec![unknown_rule_name()])
        );
    }
}

child_test! {
    fn language_server_keeps_effect_rules() {
        let (project_session, _) =
            projecttestutil::setup(files(&[(CONFIG, CONFIG_TEXT), (INDEX, INDEX_TEXT)]));
        open(&project_session, &format!("file://{INDEX}"), INDEX_TEXT);
        let p = program(&project_session, &format!("file://{INDEX}"));
        let file = p.get_source_file(INDEX).expect("index.ts");
        let semantic: Vec<i32> = ls_program::get_semantic_diagnostics(
            &p,
            &projecttestutil::with_request_id(&bg()),
            file.root,
        )
        .iter()
        .map(|d| d.code())
        .collect();
        let config: Vec<i32> = p
            .get_config_file_parsing_diagnostics()
            .iter()
            .map(|d| d.code())
            .collect();
        assert_eq!((semantic, config), (vec![TS2322, GLOBAL_DATE], vec![unknown_rule_name()]));

        // An API session in the server shares its checkers and keeps the rules.
        let session = api::new_lsp_session(Rc::clone(&project_session), None);
        assert_eq!(
            api_codes(&session),
            (vec![TS2322, GLOBAL_DATE], vec![unknown_rule_name()])
        );
        session.close();
        project_session.close();
    }
}

const UNUSED_CONFIG: &str = "/home/projects/u/tsconfig.json";
const UNUSED_INDEX: &str = "/home/projects/u/index.ts";
/// The plugin and no noUnusedLocals or noUnusedParameters: the unused check
/// gives only suggestions.
const UNUSED_CONFIG_TEXT: &str =
    r#"{"compilerOptions":{"strict":true,"plugins":[{"name":"@effect/language-service"}]}}"#;
/// Two type-predicate parameters that the code never reads.
const UNUSED_INDEX_TEXT: &str = "export const isString = (value: string | number): value is string => true;\nexport function isNumber(input: unknown): input is number {\n    return true;\n}\n";

const TS6133: i32 = 6133;

/// The TS6133 suggestions for `value` and `input`: code, start and
/// category.
fn unused_parameter_suggestions() -> Vec<(i32, i32, Category)> {
    ["value", "input"]
        .iter()
        .map(|name| {
            let pos = UNUSED_INDEX_TEXT.find(name).expect("the parameter");
            let pos = i32::try_from(pos).expect("an i32 position");
            (TS6133, pos, Category::Suggestion)
        })
        .collect()
}

// effectfix1: the Effect rules run after the unused check. Their type
// queries mark a type-predicate parameter referenced, so before the fix a
// later suggestion request lost its TS6133 (the reference does too). The
// language server asks for the semantic diagnostics first.
child_test! {
    fn language_server_reports_unused_type_predicate_parameters() {
        let (project_session, _) = projecttestutil::setup(files(&[
            (UNUSED_CONFIG, UNUSED_CONFIG_TEXT),
            (UNUSED_INDEX, UNUSED_INDEX_TEXT),
        ]));
        open(&project_session, &format!("file://{UNUSED_INDEX}"), UNUSED_INDEX_TEXT);
        let p = program(&project_session, &format!("file://{UNUSED_INDEX}"));
        let file = p.get_source_file(UNUSED_INDEX).expect("index.ts");
        // The document diagnostics of the language server (Go
        // `getAllDiagnostics`): syntactic, semantic, then suggestion.
        let mut diagnostics: Vec<(i32, i32, Category)> =
            get_all_diagnostics(&projecttestutil::with_request_id(&bg()), &p, file.root)
                .iter()
                .map(|d| (d.code(), d.pos(), d.category()))
                .collect();
        diagnostics.sort_by_key(|&(_, pos, _)| pos);
        assert_eq!(diagnostics, unused_parameter_suggestions());
        project_session.close();
    }
}

// The same through the API, with the rules (`TSGO_EFFECT_API=1`):
// getSemanticDiagnostics, then getSuggestionDiagnostics.
child_test! {
    env &[("TSGO_EFFECT_API", "1")];
    fn standalone_api_with_effect_api_reports_unused_type_predicate_parameters() {
        let (init, _) = projecttestutil::get_session_init_options(
            files(&[(UNUSED_CONFIG, UNUSED_CONFIG_TEXT), (UNUSED_INDEX, UNUSED_INDEX_TEXT)]),
            None,
            TypingsInstallerOptions::default(),
        );
        let session = api::new_standalone_session(&init, None);
        let ctx = bg();
        let snapshot = nil_error(session.handle_create_snapshot(
            &ctx,
            &CreateSnapshotParams {
                snapshot_request_changes_params: SnapshotRequestChangesParams {
                    open_projects: vec![doc(UNUSED_CONFIG)],
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .snapshot;
        let project = nil_error(session.handle_get_default_project_for_file(
            &ctx,
            &GetDefaultProjectForFileParams {
                snapshot,
                file: doc(UNUSED_INDEX),
            },
        ))
        .expect("a default project")
        .id;
        let params = GetDiagnosticsParams {
            snapshot,
            project,
            files: Some(vec![doc(UNUSED_INDEX)]),
        };
        assert_eq!(nil_error(session.handle_get_semantic_diagnostics(&ctx, &params)), vec![]);
        let mut suggestions: Vec<(i32, i32, Category)> =
            nil_error(session.handle_get_suggestion_diagnostics(&ctx, &params))
                .iter()
                .map(|d| (d.code, d.pos, d.category))
                .collect();
        suggestions.sort_by_key(|&(_, pos, _)| pos);
        assert_eq!(suggestions, unused_parameter_suggestions());
        session.close();
    }
}

const BUILD_DIR: &str = "/home/projects/b";
const BUILD_CONFIG: &str = "/home/projects/b/tsconfig.json";
const BUILD_INDEX: &str = "/home/projects/b/index.ts";
const BUILD_INFO: &str = "/home/projects/b/out/tsconfig.tsbuildinfo";
/// A composite project with one Effect error and no TypeScript error. Build
/// info that records no error lets a later `tsc -b` call it up to date.
const BUILD_CONFIG_TEXT: &str = r#"{"compilerOptions":{"strict":true,"composite":true,"outDir":"out","plugins":[{"name":"@effect/language-service","diagnosticSeverity":{"globalDate":"error"}}]},"files":["index.ts"]}"#;
const BUILD_INDEX_TEXT: &str =
    "export const value: string = \"x\";\nexport const date = new Date();\n";

/// A standalone API session on the composite project, and its file system.
fn build_session() -> (Rc<api::Session>, SessionUtils) {
    let (init, utils) = projecttestutil::get_session_init_options(
        files(&[
            (BUILD_CONFIG, BUILD_CONFIG_TEXT),
            (BUILD_INDEX, BUILD_INDEX_TEXT),
        ]),
        None,
        TypingsInstallerOptions::default(),
    );
    (api::new_standalone_session(&init, None), utils)
}

/// Builds the composite project as `tsc -b` does, through a new build
/// orchestrator of `session`: the exit status and the diagnostic codes.
fn build(session: &api::Session) -> (ExitStatus, Vec<i32>) {
    let ctx = bg();
    let orchestrator = nil_error(session.handle_create_build_orchestrator(
        &ctx,
        &CreateBuildOrchestratorParams {
            root_names: vec![BUILD_CONFIG.to_string()],
            cwd: BUILD_DIR.to_string(),
            ..Default::default()
        },
    ))
    .build_orchestrator_id;
    let result = nil_error(session.handle_build(
        &ctx,
        &BuildParams {
            build_orchestrator_id: orchestrator,
            project: String::new(),
        },
    ));
    (
        result.status,
        result.diagnostics.iter().map(|d| d.code).collect(),
    )
}

/// The version of the written build info, and whether it records the
/// Effect options.
fn build_info(utils: &SessionUtils) -> (String, bool) {
    let (text, ok) = utils.fs_from_file_map().fs().read_file(BUILD_INFO);
    assert!(ok, "the build writes {BUILD_INFO}");
    let info = parse_build_info(&text).expect("valid build info");
    (info.version, info.effect.is_some())
}

fn plain_build_info() -> (String, bool) {
    (build_info_version(false).into_owned(), false)
}

fn effect_build_info() -> (String, bool) {
    (build_info_version(true).into_owned(), true)
}

child_test! {
    env &[("TSGO_EFFECT_API", "0")];
    fn standalone_api_build_writes_plain_build_info() {
        let (session, utils) = build_session();
        assert_eq!(build(&session), (ExitStatus::Success, vec![]));
        assert_eq!(build_info(&utils), plain_build_info());

        // A later tsc build finds build info of another version, checks the
        // project again and reports the Effect error.
        rulerunner::clear_api_process();
        assert_eq!(
            build(&session),
            (ExitStatus::DiagnosticsPresentOutputsGenerated, vec![GLOBAL_DATE])
        );
        assert_eq!(build_info(&utils), effect_build_info());
        session.close();
    }
}

child_test! {
    env &[("TSGO_EFFECT_API", "0")];
    fn standalone_api_build_checks_tsc_build_info_again() {
        let (session, utils) = build_session();
        rulerunner::clear_api_process();
        assert_eq!(
            build(&session),
            (ExitStatus::DiagnosticsPresentOutputsGenerated, vec![GLOBAL_DATE])
        );
        assert_eq!(build_info(&utils), effect_build_info());

        // The standalone API build does not take the Effect diagnostics of
        // that build info: it checks the project again without the rules.
        rulerunner::set_api_process();
        assert_eq!(build(&session), (ExitStatus::Success, vec![]));
        assert_eq!(build_info(&utils), plain_build_info());
        session.close();
    }
}

// Theo PR #4 before effectfix2 wrote build info with the Effect options and
// Effect diagnostics, but the plain version. effect-tsgo sees another
// version there and builds again. The standalone API without the rules
// does the same, and does not report the old Effect diagnostics.
child_test! {
    env &[("TSGO_EFFECT_API", "0")];
    fn standalone_api_build_checks_build_info_of_an_older_effect_build_again() {
        let (session, utils) = build_session();
        rulerunner::clear_api_process();
        assert_eq!(
            build(&session),
            (ExitStatus::DiagnosticsPresentOutputsGenerated, vec![GLOBAL_DATE])
        );
        let (text, ok) = utils.fs_from_file_map().fs().read_file(BUILD_INFO);
        assert!(ok, "the build writes {BUILD_INFO}");
        let version = |effect| format!(r#""version":"{}""#, build_info_version(effect));
        assert_eq!(text.matches(&version(true)).count(), 1, "{text}");
        let older = text.replace(&version(true), &version(false));
        assert!(utils.fs().write_file(BUILD_INFO, &older).is_ok());
        assert_eq!(build_info(&utils), (build_info_version(false).into_owned(), true));

        rulerunner::set_api_process();
        assert_eq!(build(&session), (ExitStatus::Success, vec![]));
        assert_eq!(build_info(&utils), plain_build_info());
        session.close();
    }
}
