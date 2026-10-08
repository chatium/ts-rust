# Native Effect diagnostics

tsc-rs reports the diagnostics of the Effect language service
(`@effect/language-service`, codes TS377000 to TS377999) in the same
program and checker pass as the ordinary TypeScript diagnostics. No Go
binary, patch or second pass is needed.

The code is `crates/ts_goport/src/effect`, a port of
[Effect-TS/tsgo](https://github.com/Effect-TS/tsgo) at the
`@effect/tsgo@0.46.1` release commit `f1a7cad0`. Read
`crates/ts_goport/src/effect/PORTING.md` before you change it.

## What it does

- Reads the `@effect/language-service` entry in `compilerOptions.plugins`,
  with the reference defaults. A child config that `extends` a base merges
  only the keys it sets: `diagnosticSeverity` merges by rule, `overrides`
  are appended, and override globs stay relative to the config that
  declares them. As in the reference, a child `"plugins": []` (or `null`)
  replaces the plugin list but keeps the base's Effect options, so Effect
  still runs. To turn the rules off in a child, list the plugin with
  `"diagnostics": false`.
- Runs all 116 rules after the checker has checked each project file
  (not declaration files and not files from `node_modules`). Rule
  severities come from `diagnosticSeverity`, matching `overrides`, and
  `@effect-diagnostics` / `@effect-diagnostics-next-line` comments.
- The rules run once per file, after the unused-identifier check. The
  reference runs them before it, inside `checkSourceFile`. Their type
  queries can mark a name referenced (the parameter of an `x is T` type
  predicate), so the reference loses that TS6133. With the plugin, the
  unused check always runs first. Without `noUnusedLocals` and
  `noUnusedParameters` it adds only suggestions: `tsc` does not print
  them, and the editor and the API show them.
- Effect errors and warnings fail `tsc`. Suggestions print and do not fail
  (`ignoreEffect{Errors,Warnings,Suggestions}InTscExitCode`).
  `@ts-ignore` does not hide Effect diagnostics.
- Effect options are stored in `.tsbuildinfo`, so a change to them
  invalidates the cached diagnostics. With the plugin on, `.tsbuildinfo`
  records the version `<version>+effect-tsgo.0.46.1`, as effect-tsgo does.
  Plain tsgo and tsc-rs without the plugin then check the project again
  instead of reading Effect diagnostics (plain tsgo panics on those:
  "Unknown diagnostic message"). tsc-rs from Theo PR #4, before this
  suffix, wrote the plain version with Effect options and diagnostics.
  tsc-rs checks that build info again too, as effect-tsgo does. Plain tsgo
  still panics on it until one tsc-rs run writes it again. The language
  server shows the same diagnostics. `tsc -v`, `tsc --help` and the
  language server's `serverInfo` print the plain version; effect-tsgo
  prints `<version>+effect-tsgo.0.46.1` there.
- With no plugin entry, nothing runs. An entry with only a `name` runs every
  rule at its default severity. `"diagnostics": false` or
  `"diagnosticSeverity": null` turns the rules off.

## Where it hooks in

Each site names the reference patch it ports (`_patches/typescript/NNN-*`):
`checker_p03.rs` (in `checkSourceFile`, after the unused check, once per
file), `relater_p5.rs` (relation
errors), `program.rs` (`@ts-ignore` and `noEmitOnError`),
`execute/tsc/emit.rs` (exit code), `execute_tsc.rs` (tsc mode),
`frontend/tsoptions/*` (parse, merge, validate), and
`execute/incremental/*` (build info).

The messages are generated from
`crates/ts_goport/data/effect/effectDiagnosticMessages.json` by
`node scripts/effect/gen-effect-messages.mjs`.

## Checks

All take the reference Effect-patched `tsc` as `--ref`:

- `scripts/effect/reference-cases.mjs`: the reference's own 529 test cases
  (Effect v3 and v4), compiler against compiler.
- `scripts/effect/focused-fixtures.mjs`: `crates/ts_goport/tests/effect_fixtures`,
  each with expected codes and exit status.
- `scripts/effect/incremental-check.mjs`: `--incremental` and `--watch`
  after source, imported type, config and directive edits. With `--plain`
  (the tsgo oracle), plain tsgo and tsc-rs also take turns on one
  `.tsbuildinfo`.
- `scripts/effect/t3-parity.mjs` and `scripts/effect/bench-t3.mjs`: T3 Code
  parity and timing.

## Not ported

Editor features of the language service: quick fixes, refactors, hover,
completions, inlay hints, document symbols and goto. tsc mode never runs
them.
