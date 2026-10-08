# Typechecker accountability

These rules were installed on September 5, 2026, at Theo's request. The machine-readable status is in [typechecker-state](typechecker-state/current.json): a small `current.json` and an append-only `history.jsonl`. Use `scripts/state` to read and write it.

The [reset plan](typechecker-reset-plan.md) explains the failure and recovery choices. These rules control execution if an older plan conflicts with them. They do not change the full port requirements.

## Standing continuation authorization

After the initial recovery phase closed, Theo instructed:

> You have my permission to do whatever you need to do. I have you on full access for a reason. Keep going

This authorizes continued work toward the Query core and Hono goal without a new permission request at each revision limit. The initial four-revision record remains complete and unchanged. Use `recovery-continuation` for later work and keep the cumulative revision numbers and all earlier results. The initial per-hypothesis and cumulative approval stops do not apply to this authorized continuation.

Keep experiments focused. After two measured attempts without useful recovery or Query progress, reassess the cause and dependency path with the independent reviewer. Reassessment is required. Another user approval is not required for work within this goal.

Necessary repository source, tests, helpers and project inputs may be read for this work. Use bounded production reads and fresh metadata. Keep raw type graphs and large failure payloads out of reports and conversation output. Keep ordinary project inputs unchanged.

Use as many independent analysis agents as the work needs, up to the available 40. Give each one separate file ownership. Keep one compiler writer and the root runtime queue while shared checker state remains under repair.

Read-only research helpers are always allowed and need no build or demo gate. A goal that asks for parallel work starts a workflow within 30 minutes. For a build that fails in many files, root builds once and gives each fix agent one file and its error list. Fix agents do not run Cargo. A later wave waits until the earlier wave it depends on is integrated and builds. [AGENTS.md](../AGENTS.md) has the details.

This authorization does not waive the [protected set](#protected-set), pinned Go evidence, exact source identity, independent verdicts or complete Query/Hono diagnostics, types and symbols against pinned Go. STOP still blocks acceptance and unrelated feature work. It permits continued diagnosis and regression repair under this authorization.

## Start or resume

1. Read this file, the saved state and the reset plan. Do this after a context reset as well as at goal start.
2. Check the user's current instruction. A historical active goal is not permission to ignore the pause. Theo's new goal can authorize the recovery block, but does not waive the acceptance rules.
3. Verify the candidate source and required evidence still exist. Do not rebuild a missing accepted baseline from the current compiler.
4. Reuse or create the regression auditor and independent reviewer. Give them this file and the saved state. Record their current agent names. Agent names can change across sessions. Their roles and limits cannot.
5. Select one primary implementer and record file ownership before any parallel work. Root owns the runtime queue and integration state. Keep the other compiler drafts frozen.
6. Record the recovery hypothesis, exact tests it must recover, current source and cumulative revision number before editing. Record any applicable block budget.

The paused state can be activated only by Theo's next instruction to start the work. Record that instruction in the state. Do not claim that the goal has started merely because the accountability files exist.

## Protected set

On 2026-09-28 Theo approved the retirement of the legacy cargo roster. The saved state holds his words in two notes:

- `legacy-removal-direction-2026-09-28`: "should we be deprecating the old stuff and removing it? this project is greenfield, we can delete whatever"
- `legacy-removal-rule-approval-2026-09-28`: "i approve any rule changes that allow removing the legacy code"

The legacy roster (the 6,055 and 6,330 names, the `ts_checker`, `ts_compiler`, `ts_parser`, `ts_binder` and `ts_fixture` stages and the current-source corpus) tests only the old stack. No roster test runs goport code. Thus goport's own evidence is now the protected set, and "never lose a pass" applies to it. A batch uses this set when its rules say `protectedSet: "goport"`. Root records the change in `acceptanceRuleChanges` (id `goport-protected-set`) and cites the approval note.

### What the set holds

- **goport tests, per name.** The 7 test binaries: the `ts_goport`, `goport_util` and `goport_lsproto` libs, and `go_baselines` (default), `multi_program`, `emit_pool` and `early_emit`. Also the ignored-by-default goport suites that the project runs by name (the `TestSubmodule` shards, `test_local`, the reference files, the transpile runner, the lib snapshot tests and `fswatch_linux`), and the unit tests of the kept crates `ts_scanner`, `ts_ast`, `ts_diagnostics`, `ts_path`, `ts_core` and `ts_jsnum`.
- **Gate items.** Every item in the gate `manifest.json`, one by one.
- **Bound runs.** The bound Query core and Hono runs.
- **LSP oracle.** `lsp_oracle.py check` on the batteries in `LSP_BATTERIES` of `candidate.sh`: `b1-inline`, `b1-query-core`, `b2-query-core`, `b1-hono`, `b2-hono` and `fourslash` (240,911 `same` requests at R131), against the pinned Go goldens. `candidate.sh side` runs it with the gate's `tsgo`. The run must have 0 diff, 0 goport_error, 0 timeout and 0 crash, and `oracle-compare.py` against the base run must show 0 lost, unrun and absent requests.
- **API oracle.** `scripts/goport/api_oracle.py check`, with `GOPORT_PIN` set to the batch pin, on the batteries in `API_BATTERIES` of `candidate.sh`: `qc`, `qc-proto`, `qc-callbacks`, `qc-lsp`, `tsp-lsp`, `qc-xchecker`, `hono`, `hono-xchecker`, `zod` and `effect` (108,203 `same` requests at R129). `candidate.sh side` runs it. From pin 16c25522e123 (bump B), it also runs the batteries in `API_EXT_BATTERIES`: `qc-ext`, `hono-ext` and `zod-ext` (the API methods that bump B adds or changes). The oracles of the older pins 52168999f3dc and dc37b5249ab6 have no goldens for them, so a run at those pins runs the 10. Pinned Go has known diffs here (4 at R129), so the test is per request: `oracle-compare.py` against the base run must show 0 lost, unrun and absent requests. The run must cover every battery of the base run, and the `goportSha` of each battery in `<dir>/manifest.json` must be the sha of the gate's `tsgo`.
- **np-suite.** `scripts/goport/np-suite.sh run <label> <bins>/tsgo` runs the typescript-go native-preview API client tests (459 tests at R129, all pass) against goport's `tsgo`, with `NP_GO_DIR` at the Go checkout of the batch pin. `np-suite.sh diff <base label> <new label>` must show no `pass ->` change. Use a new label for each run, because `run` deletes the dir of its label. No tool runs it for a candidate (see "np-suite is outside `candidate.sh`" below).
- **Quality.** rustfmt, clippy, and 0 warnings in `ts_goport` and in the kept crates.
- **Verdicts.** Both independent verdicts on the exact source.

Not in the set: the other LSP batteries (`complete-u1`, `b4`, `b3-zod`, `b3z-effect` and `fourslash-nonfatal`). Integration merges and pin bumps run them as extra checks and report the results with the batch. They are outside the set because `candidate.sh` runs one battery set and these need other golden roots or `--traces-dir`, and because `b4` has a Go-unstable auto-import request with no flaky mark (int13: the other answer appears in 13 of 25 saved full `b4` runs, and live Go at the pin also misses the golden), so a strict per-request rule fails it in about half of all runs. A new diff class or a crash in them is a finding for the reviewer.

### Baseline and base

- The first baseline is `docs/goport-protected/tests-r131.json.gz`: the `results.json` of the R131 test binaries, as compact JSON with sorted keys, compressed with `gzip -n -9` so that the bytes are the same each time. Every reader of a results file accepts `.json` and `.json.gz`. `docs/goport-protected/README.md` tells what it is, how to make it again and its sha256.
- Each accepted revision writes its `results.json` to its evidence cache. The test base for a candidate is the result of the last accepted revision. The gate base is the gate manifest of the last accepted revision.
- The LSP oracle, API oracle and np-suite base is the run of the last accepted revision. When that revision has no such run, run it once on that revision's saved release bins and use that run as the base. A legacy revision (R131 or R132) has no API or np-suite run. After a legacy batch, the API and np-suite bases are `api-r131` and `np-r131`, run on the R131 release bins (`target/continuation-r97-goport/evidence-cache/725433165cab344d/bins`), and the test base is the first baseline, also from R131. The `goport-protected-set` rule names `api-r131` in `apiBaseline`. So a loss in R132 still shows in the first goport batch.
- Do not make a base from a candidate that is not accepted. Do not rebuild a missing base from the current source.

### Tools

- `scripts/goport/build-goport-tests.sh <checkout> <testbin-dir>` builds all the test binaries at a checkout. It reads `cargo metadata`, so a new crate or `tests/*.rs` target joins the set with no edit (except the crates in its `NOT_PROTECTED` list). It writes the binaries, `SUITES`, `relbin/`, `COMMIT`, `TREE` and `bins.sha256`.
- `scripts/goport/goport-tests.sh <testbin-dir> <out-dir> [--pin PIN]` runs every protected suite from those binaries in a clean environment (`env -i`, and a bwrap over a git archive of the crates of `COMMIT`). It writes `<out-dir>/results.json`: one status for each name (`ok`, `failed`, `ignored` or `unrun`) and the list of `incomplete` suites. It refuses an out-dir that has results. A test failure still ends the log with `DONE`: the compare decides.
- `scripts/goport/compare-tests.py <base> <new> [--name-map TSV] [--out FILE]` gives the retained, recovered, lost, absent, unrun, new and removed names for each suite. It exits 1 when a base `ok` name is lost, absent or unrun, or when a name map line is rejected. It exits 2 on bad input, also on a results pin that is not 7 to 64 hex characters.
- **Name map.** A TSV with one line per moved, renamed or removed test and 5 columns: old suite, old name, new suite, new name and evidence. The evidence cell must not be empty. `-` as the new suite and new name removes a test. A line is rejected when its old name is still in the new results (except a removal line of a `go_baselines_reference` name that is `ignored` in the new results at a new Go pin: a stale Go reference file that no Go test at the new pin writes; `compare-tests.py` and the check count such lines as `mapRemovedIgnored`, and the reviewer checks the Go evidence of each), when its new name is a base name, or when it removes a name while the Go pin did not change and the suite is not a kept-crate suite. The check script applies the same rules. It takes the pin change from `upstreamPin.to` of the base batch and of the batch, not from the results files.
- `scripts/goport/gate-compare.py <base manifest> <new manifest> [--state FILE] [--out FILE]` compares the gate item by item with the gate rules below. It exits 1 on a regression.
- **Gate id map.** A TSV with one line per moved or removed corpus case. A move line has 3 columns: old id, new id and the case path (for example `corpus-diag/04640`, `corpus-diag/04704` and the case's `.ts` path). A removal line has 4 columns: old id, `-`, the case path and a note that names the Go commit that deletes the case file (`gate-compare.py` checks only that the note has a word of 7 to 40 hex digits). It removes a case only when `gate-compare.py` checks the removal against Go at both pins (see "Pin bumps"). Any other line with `-` or a fourth column is bad input. Only the corpus families (`corpus-diag` and `corpus-emit`) can have lines, and both ids of a line are in one family (the part before the first `/`). The case path of a corpus item is the source word at its fixed place in the item detail (`<class> <case path>` and `<class> exit <go>/<goport> <case path>`). Two lines cannot share an old id, a new id, or a case path in one family. The batch names the map in `batch.gateIdMap` (`path` and `sha256`).
- `scripts/goport/oracle-compare.py <base dir> <new dir> [--out FILE]` compares two LSP or API oracle results dirs per request. Each base `same` or `oracle_error_same` request must be so again. It exits 1 when one is lost, unrun or absent.

### Rules

- **No lost pass.** Each name that is `ok` in the base must be `ok` in the candidate. A lost, absent or unrun name is STOP. New passes do not compensate for a lost pass.
- **Gate items.** A MATCH item in the base must stay MATCH, or become ALLOWED by an allow entry (same id, condition and case path) that the base manifest's allow list also has ("reallowed"). The oracle's trace order changes with threads, so some single-threaded-equal items move between MATCH and ALLOWED on the same bins. MATCH to ALLOWED by an entry that the base did not have is a regression. `gate-allow.txt` is a protected path, so a candidate cannot add an entry. An ALLOWED item must carry `allowedBy`: the gate checked its `gate-allow.txt` condition again in the new run. A new FAIL, a new item that is FAIL and a removed item id are regressions. The gate verdict alone is not the test, because the R131 verdict is FAIL (editor long growth).
- **Allow entries name their case.** A corpus id names another case at another Go pin. So an allow entry of an item whose detail names a Go test case (`corpus-diag`, `corpus-emit` and `f1`) also names that case path: `<id> | <case path> | <condition> | <reason>`. The gate applies the entry only to the item of that id whose case path (the source word at its fixed place in the detail) is the entry's path. At another pin the entry does not apply to the other case at that id: the item stays FAIL, and the entry is unused. An entry never names another case than the one its reason and evidence are for. `gate-compare.py` compares allow entries by id, condition and case path. An ALLOWED item whose entry names another case path is a regression. A base entry of the old form (no path, from `gate.sh` before this rule, for example `r137-full`) has the case path of the base item of its id, the only case it could allow in that run. Other allow entries (for example `typesyms`) keep the form `<id> | <condition> | <reason>`.
- **Gate tools.** The gate manifest records the sha256 of every tool that judges its items: the oracle, the Go dumper, `gate.sh`, the allow list, each stage script (most live under `target/`, which no protected path covers) and the saved Go outputs the stages judge against (`oracleCaches`: each `oracle-sweep` file, and one hash per family for `errcopies-oracle`, `oracle`, `project-inputs-extra/**/oracle*.txt` (every oracle file that `sweep-extra2` reads, also `oracle.variant.txt`, `variant/oracle.txt` and `variant-ts7/oracle.txt`), the `project-inputs-wide` oracle files, the f1 sample's `results/*.oracle.out` and lists, `/tmp/goport-emit-oracle`, and the corpus case lists `corpus-full/list.json`, `corpus-full/shards` and `corpus-int3/shards`, which give each corpus id its case path). `gate-compare.py` compares each hash with the base run. A changed or removed tool is a regression unless the batch lists that exact change (key, base sha256, new sha256, reason) in `batch.gateToolChanges`, and the reviewer judges each listed change. A pin bump lists its new oracle and cached Go outputs there.
- **Evidence builds.** Only the candidate checkout builds in the shared candidate target (`runtime/cargo-target`). Any other checkout builds in its own target. Before each build, `purge-foreign-fingerprints.py` removes the build records that name another checkout's files, so a stale crate cannot be linked (R132 side try 1).
- **Repeat runs.** Keep every run of a protected check on one source. To run a check again, move the old run aside (for example `tests-run` to `tests-run.1` in the evidence cache) or use a new label. Do not delete a run. This includes the gate: `candidate.sh side` keeps the record of every gate run on the source (a failed run as `gate-fail-<label>.json` in the evidence cache), `verdict-request` lists every gate run, and `accept_revision.py` refuses a failed gate run that has no flake note. A base `ok` name, MATCH gate item, `same` oracle request or passing np-suite test that fails in any run of the source is lost, even when another run passes.
- **Flakes.** The only exception is a flake that the independent reviewer accepts. Before the verdicts, root records it with `scripts/state record note flake-r<N>-<short name>`: the name, the source, the path and result of every run, the base result, and the evidence that the change did not cause the failure (for example the same failure on the base source or on pinned Go, or a timeout in the log at high load and a pass on a quiet host). A flake record covers one name on one source. It does not change a baseline, expectation, golden or flaky mark. For the tests and the oracles, the batch still uses a run with no loss, because the check script reads one run of each. For the gate, the check script reads every gate run of the source in `gateRuns`, runs `gate-compare.py` on each again, and needs a flake note for each regression of a failed run.
- **Open defect.** The editor long-growth items `editor/query-core/long` and `editor/hono/long` can FAIL only while the batch has the open defect record `editor-long-growth` in `openDefects`, only when the failure is growth alone (no rss or answer failure), and only when the Rust growth is at most the fixed cap of the project: query-core 1.58 and hono 1.28 MiB/edit. Each cap is the highest growth of a good build in the R126 to R131 and bump B gates (query-core 1.43, hono 1.13) plus 0.15 MiB/edit of noise. The cap does not follow the base, so small growth cannot add up over revisions. `gate-compare.py` applies the caps.
  - The caps only go down. A batch that lowers the growth (for example a fix) can lower the cap of that project to its growth + 0.15 MiB/edit, and the reviewer checks the value against the gate runs. A cap also goes down by itself to 1.00, the lowest value of the gate's own limit (2 x Go + 1), once the base Rust growth of that project is at or under 1.00: from then on its item must be MATCH with growth at or under 1.00. A MATCH at a higher growth does not lower it, because the gate's limit follows Go's slope and the same bins can then FAIL (hono at 1.13 MiB/edit on the R131 bins: MATCH in `r131-full` at limit 1.88, FAIL in `r131-full-2` at limit 1.00). No batch can raise a cap. Only Theo can.
  - When the defect is closed, both items must be MATCH like every other item.
- **Pin bumps.** When a pin bump renames or removes Go tests, give a name map (format under "Tools") from each old name to its new name, with Go evidence at the new pin. The reviewer checks the map. A lost name without a map is STOP.
  - When the new pin renumbers gate items (Go adds or removes corpus cases, so the corpus-diag and corpus-emit ids of the same case change), give a gate id map (format under "Tools") in `batch.gateIdMap`. The gate corpus items are Go test cases, so this rule applies the name map rule above to them. `gate-compare.py` uses the map only when the base and new gate manifests are at different upstream pins. At one pin it has no effect: it cannot move an id.
  - A mapped id is the same item under its new id. The line's case path must equal the case path of the base item and of the new item, so a line cannot pair two different cases. At a new pin of layout `typescript` (microsoft/TypeScript, `tsc/`), only a case path under `_submodules/TypeScript/tests/cases/` moves to `testdata/tests/cases/`, with the renames of the pin's `testdata/promotedTestCollisions.txt`, and other paths stay the same; the new item's case path is the line's path moved in this way, and `gate-compare.py` checks it. A word that both details share (for example `MATCH`) is not a case path.
  - A base allow entry moves only with its own case: an entry whose id is a base item that a working line moves applies to the new id of that line, with its own case path. Any other entry of a mapped family (an old pin's id, a case without a line or a glob) gives no allowance.
  - A base id of a family that the map names is a removed id (a regression) when it has no working line: no line, a move line whose new id is not in the new run, a line whose case path names another case, or a removal line that fails the removal check. So it is never compared with another case at the same id.
  - The map removes a case only by a removal line (format under "Tools") that passes the removal check of `gate-compare.py`: the line's case path is the base item's, the case file is in the base pin's Go checkout, the new pin's Go checkout has neither that path nor that path moved as above, and no new item of the line's family has either path. `gate-compare.py` lists such a case in `idMap.removed`. A line that fails the check is broken. A base case that the new run does not hold and that no working removal line removes is a removed id, also when Go removed it or the gate sample changed. A pin that removes a sampled case needs a `gate-compare.py` change first that checks the removal against Go at both pins. The batch lists it as a tool change, and the reviewer judges it.
  - The gate samples follow the case paths, so a pin that adds cases keeps every base case. corpus-diag runs the pin's `corpus-int3/shards/shard-0.json` (the same 1,500 case paths at 52168999f3dc and 16c25522e123). corpus-emit runs the cases whose paths are in `scripts/goport/gate-emit-sample.txt` (its 1,501 cases at 52168999f3dc).
  - The history row and both verdicts carry `gateIdMapSha256`, the sha256 of `batch.gateIdMap` (null without a map). The check script requires it, checks the file hash, passes the map to `gate-compare.py`, and requires the saved gate compare output to name the same map sha256. The reviewer checks the map.
  - **Pin bumps: oracle base at the new pin** (reviewer ruling 10, 2026-09-29, `target/continuation-r97-goport/r139-diag/reviewer-ruling.md`). At a new pin Go's own oracle answers change. The oracle base can then be the base revision measured again at the new pin, under the protected-set base rule ("run it once on that revision's saved release bins"). There is no request map. The conditions:
    1. Base identity. The runs use the base batch's release bins (the tsgo sha256 of its gate manifest), the new pin's oracle, goldens and traces, the batteries of `candidate.sh` and the same host setup. The batch records them in `batch.oracleRebase` (`lsp` and `api`, each with `runs` [`label`, `dir`, `resultsSha256`], `binsSha256` and `oracleSha256`). The history row and both verdicts carry `oracleRebaseSha256`, the sha256 of its canonical JSON. `accept_revision.py` and the check script refuse it outside a pin-bump batch, and when a run's recorded tsgo or oracle differs or its files changed.
    2. Every run counts. Keep every rebase run. The protected base is every request that is `same` or `oracle_error_same` in any rebase run. The candidate is compared per key, with no map. A lost, unrun or absent request blocks, unless it is a recorded flake under the Flakes rule.
    3. Absolute parity at the new pin. The candidate's LSP run has 0 `diff`, `goport_error`, `oracle_error_diff`, `timeout` and `crash`. The API run has 0 `goport_error`, `crash` and `timeout`, and each `diff`, `id_only` or `oracle_error_diff` request is in `batch.oracleRebase.api.knownDiffs` with a reason: the known qc-callbacks `@callbacks` group, or a Go-side reason. A request of an answer set (condition 5) must have goport's answer in its set, and is then allowed. An unused known diff also blocks.
    4. Go-side change list. Root reports the base at the old pin against the base at the new pin on unchanged source, with counts per class. For each request that was `same` at the old pin and whose Go answer changed at the new pin, the report shows the candidate's status. One that is not `same` in the candidate, and not in an answer set, is an unported Go change: STOP.
    5. Answer sets. A request whose Go answer varies at the new pin (two or more recorded Go answers from Go reruns at that pin, and goport's answer is one of them) keeps a weaker protection in an answer set (`goport-oracle-answers/1`) in `batch.oracleAnswers` (`kind`, `path`, `sha256`, `pin`). Only a pin-bump batch can add answer sets, at its own pin. The history row and both verdicts carry `oracleAnswersSha256`, the sorted set sha256 values. After acceptance, `open_revision.py` gives them to the next batch as `protectedBase.oracleAnswers`, and `accept_revision.py` keeps them while the pin stays. `oracle-compare.py --answers` counts a base `flaky_oracle` request of a set as retained only when goport's answer stays in the set. At a later pin bump, such a request is protected like a `same` request. The Flakes rule is unchanged for repeat-run flakes.
    6. np-suite. Run the base np-suite at the new pin on the base bins, and compare the candidate per test. No change from pass is allowed. An np-suite run at the old pin stays recorded as invalid.
    7. Tools. Only the rebase and answer-set support goes to `main`, and the reviewer judges each path as pin-bump tooling. There is no request map tool.
- **Moved tests.** When tests move to a different binary or crate, or a kept crate's tests are deleted because goport's Go port replaces them, give a name map in the same way. R131 did this for the crate split (`target/continuation-r97-goport/buildspeed/split1/r131/lib-name-map.tsv`). The reviewer checks that each new name tests the same behavior.
- **Expectation changes.** An expectation change needs concrete pinned Go evidence, an explicit old-name map and independent review.
- **Batch fields.** `open_revision.py` writes `protectedSet: "goport"` and `protectedBase` (the base test results, gate manifest, and LSP and API runs). The batch needs a hex Go pin in `upstreamPin.to`. `accept_revision.py` writes these fields and the check script reads them:
  - `goportTests`: `results`, `sha256`, `base`, `baseSha256`, `compare` (with `lost`, `absent`, `unrun`, `retained` and `recovered`), `nameMap` (`path` and `sha256`, or null), `testbin`, `commit` and `compareOutput`.
  - `gate` and `gateCompare`: `base`, `baseSha256`, `new`, `sha256`, `regressions`, `knownOpen` and `output`. Also `gateVerdict`.
  - `gateRuns`: every gate run of the source, oldest first, with the batch gate last. Each entry has `label`, `manifest`, `sha256`, `compare` (`path` and `sha256`) and `regressions` with their flake notes. Each kept `gate-compare-fail-<label>.json` must have an entry.
  - `languageServerOracle` and `apiOracle`: `label`, `summary`, `dir`, `host`, `result`, `base` (`label` and `dir`), `compare` and `output`.
  - `quality` and `qualityEvidence`, and the bound runs `ordinaryQuery` and `latestHono`.
  - Both verdicts. The history row and both verdicts carry `goportTestsSha256`, `gateSha256`, `nameMapSha256` and `gateIdMapSha256`.

  These batches have no `fullResult`, `corpus` or `rosterCarryForward`.
- **np-suite is outside `candidate.sh`.** `candidate.sh side` runs every other part of the set, and `accept_revision.py` records it. No tool runs np-suite for a candidate, and the check script does not read it. Root runs `np-suite.sh run <new label> <bins>/tsgo` on the candidate's release bins and `np-suite.sh diff <base label> <new label>`, and records the run with `accept_revision.py --extra` as `npSuite` (label, dir, base label, and the diff output path and sha256). Both verdicts check it by hand.
- **Protected paths.** The files that judge the protected set are protected paths: this file, `AGENTS.md`, `docs/goport-protected/`, the check and state tools, `scripts/upstream/pin.py`, `UPSTREAM.json` (the oracle and Go checkout of each pin), `scripts/run-cargo-capped.sh` (it builds the test and release bins and runs clippy), and the goport pipeline, test runners, compares, oracles and gate scripts. `scripts/goport/open_revision.py --protected` prints the list. A candidate must not change a protected path: `candidate.sh check` and `open` refuse the change, unless the batch's `allowedChangedFiles` lists that exact path.
  - A pin-bump batch that must change a tool lists each exact path in its batch. For example, bump B lists `UPSTREAM.json`, `scripts/goport/gate.sh` and `scripts/goport/bound2.sh`, and `scripts/goport/candidate.sh` for its new API batteries. The independent reviewer judges each tool change: the pin must need it, and it must not drop or weaken a check, suite, gate item, battery or name of the base. A change that the reviewer does not accept is STOP.
  - Any other batch needs Theo's approval to change a protected path. Root records it in the state.
- Only Theo can change these rules. Root must not widen them by delegation.

### Retired legacy roster

New batches do not use the legacy roster, the roster drive, the current-source corpus or the roster carry-forward. R132 is the last revision under them: it was opened in batch port-18 under the legacy rules before this rule was recorded. Historical records and evidence stay unchanged. The check script still checks old batches in `history.jsonl` under the rules that applied to them. It uses this legacy mode (a batch with no `protectedSet`) only for revisions up to 132 (`recoveryRevision` 132 or lower). A later revision without `protectedSet: "goport"` is STOP, also with a roster carry-forward. The historical source references are:

- Accepted commit: `5c7c7bd20cb45ebc8f2171eed8478fa2797e8343`, 6,055 accepted passes.
- Code-equivalent checkpoint: `8f4943ac6dffa6785165e18a07a5b369a6811da7`. The measured run belongs to the first commit.
- Later passing-name reference: fingerprint `162fccf9061a30bac011c98f3086538a7ba64b5ca87e99ca6c78b69990b175be`, 6,330 passes. It also had 446 failures and is not an accepted compiler.

## Regression auditor role

Use this assignment when creating or resuming the agent:

> You are the independent regression auditor. You do not edit compiler code, test expectations, baseline records or the primary implementer's patch. Read the accountability rules and state. Compare the candidate with the base (the last accepted revision) for each goport test name with `compare-tests.py`, for each gate item with `gate-compare.py`, for each LSP and API oracle request with `oracle-compare.py` and for each np-suite test with `np-suite.sh diff`. Verify that the results name the exact candidate source (commit, tree and test binary hashes), that the base hashes match the saved base, and that each required run closed normally. Check every run of the source in its evidence cache, not only the run that the batch names, and every gate run that the verdict request lists: a loss in any run is lost unless the reviewer accepted its flake record. Keep base, candidate and later results separate. Identify every lost, absent and unrun name, every gate regression and every lost oracle request or np-suite test. Check the open-defect condition for the editor long-growth items. Check each name map and each expectation change against its concrete pinned Go evidence. Return one source-bound PASS or STOP verdict with the blocking facts. New passes cannot compensate for lost ones. Do not approve a focused run as full acceptance.

The auditor uses existing result parsers and logs. It does not inspect private test bodies or raw failure payloads to infer a cause. Cause investigation is a separate assigned task with its own access limits.

## Independent reviewer role

Use this assignment when creating or resuming the agent:

> You are the independent reviewer and progress reviewer. You do not edit the compiler patch or its expectations. Read the accountability rules, reset plan and state. Review the change against pinned Go, including its real callers, context, cache publication, recursion, diagnostics, negative cases and repeat behavior. Check that the protected set is complete: every test binary and suite of the base still runs, every base name and gate item is in the new results or in a checked name map, every named LSP and API battery and np-suite ran, and the baseline files match their recorded sha256. Accept a flake only with the evidence that the flake rule asks for. Check each name map against its Go evidence. Judge each change to a protected path that the batch lists. Check the actual ordinary Query result, Hono freshness, scope and cumulative revision count. A narrow static review or a later stopping point is not a completed feature. Return one source-bound PASS or STOP verdict. Stop new feature work when the operation remains incomplete, the protected set is incomplete, new regressions appear, the revision limit is reached, or the work changes targets without a decision. Do not reset the counter for new tests, traces, branches or agent replacements.

This is the existing independent code-review role with explicit authority to stop the work. It is not an additional approval committee. The reviewer checks the change, the completeness of the protected set and the measured outcome, not each preparation step.

## What STOP means

A STOP from either accountability agent blocks new feature work and acceptance. Root cannot override it with a summary, a different agent's opinion or a passing new test.

Within an active, authorized recovery phase, STOP permits diagnosis, repair of the regression or withdrawal of the current batch. It does not permit another feature or a larger experiment. During the current pause, only setup, documentation and read-only investigation are authorized.

To close a STOP, record the specific corrected evidence and obtain a new verdict on that exact source from the role that raised it. If the facts are disputed, report both positions to Theo. Do not replace the reviewer to obtain approval.

Only Theo can approve a change to the acceptance rules. Any exception must record his instruction, its scope, exact affected names, Go evidence and replacement mapping. The local check must not have a general ignore-regressions switch.

## Initial recovery revision limits

These limits govern the completed initial recovery trial. The standing continuation authorization above governs later work. Do not rewrite the initial history or use the continuation to waive a failed result.

- One recovery hypothesis permits at most two measured revisions.
- The initial recovery phase permits at most four measured revisions across at most two demonstrated causes.
- Failed and unaccepted revisions count. Record a candidate before running its checks. A repeat run of identical source does not become a new semantic revision, and does not reset a counter.
- After two measured implementation batches without useful ordinary Query progress, stop feature work and reassess the dependency path. During regression recovery, apply the cumulative recovery limit and report Query separately.
- A later assertion, changed error label, new passing control, trace, different agent or renamed batch does not reset these limits.
- At the cumulative limit, either required results pass or have individually approved Go-backed expectation updates, or implementation stops with the recovery comparison and remaining losses. A recorded cause or port gap does not clear a STOP. Do not renew the same experiment automatically.

Reaching a limit does not prove that restarting from green is cheaper. Use the recorded dependency comparison. If the evidence is insufficient, stop and ask Theo for direction.

## One batch record

Root maintains one saved state and one record per batch. Write it through `scripts/state record`. `record revision` appends a revision row, `record note` archives a named record, `record passing-result` appends an auditor result and `record current` changes `current.json`. Each write adds a line to `history.jsonl`. The last line for a revision number is its current row. Update the state before starting a revision and after its measured result. Do not rely on conversation memory or subagent messages alone.

Before replacing `state.batch`, save its record at `docs/typechecker-batches/<batch-id>.json` (`scripts/state batch --with-history`) and add that path to `batchRecords`. `record current` refuses a new batch until both exist. Keep the complete initial-phase `recoveryHistory` in the next batch. The auditor compares it with the prior saved records. Do not delete old revisions or start the history again at one.

The record must identify the source, hypothesis, changed scope, exact expected recoveries, commands, completed runs, result paths and hashes, ordinary Query outcome, latest Hono result, both independent verdicts and the next permitted action. Each verdict names its agent, batch and exact source. A verdict from another source is stale.

Keep the goport test comparison, the gate item comparison and new coverage separate. Keep the bound Query and Hono runs, the LSP and API oracles, np-suite and the gate's diagnostic, emit and typesyms results separate from test totals. Full acceptance needs every part of the [protected set](#protected-set). Missing evidence is STOP.

Do not store raw type graphs, private test bodies or large failure payloads in this file. Link the existing evidence. If an ignored evidence file is missing in a future checkout, stop and recover it from preserved records. Do not invent its contents.

## Automated check

The local entry point is `node scripts/check-typechecker-batch.mjs --help`. The script reads saved evidence and state. It must reject malformed or missing evidence, mixed source identities, lost or missing passing names, stale verdicts and STOP verdicts. Tooling tests use synthetic records and do not run the compiler.

Run the real check from the main repository root:

```sh
node scripts/check-typechecker-batch.mjs docs/typechecker-state
```

Run the tooling tests with all named results visible:

```sh
node --test --test-isolation=none --test-reporter=spec scripts/check-typechecker-batch.test.mjs scripts/state.test.mjs scripts/goport/compare-tests.test.mjs
```

The check and its tests need `python3`, because the check runs `gate-compare.py` and `oracle-compare.py`.

For a batch with `protectedSet: "goport"`, the script compares the goport test result files again with the name map rules, runs `gate-compare.py` on the pinned gate manifests and `batch.openDefects`, and runs `oracle-compare.py` again on the LSP and API results dirs. It must reject:

- a lost, absent or unrun goport test name, or a rejected name map line
- a gate regression, also an editor long-growth item over its cap, a removed id that the gate id map does not move, a gate id map line that names two cases, and a gate id map removal line that fails the removal check against Go at both pins
- a failed gate run of the source that `gateRuns` does not list, or a regression of a failed run with no flake note
- a lost, unrun or absent LSP or API request, an LSP run with a diff, goport_error, timeout or crash, and an API run that misses a base battery or ran another goport binary than the gate's `tsgo`
- a missing or non-hex `upstreamPin.to`
- a results, base, gate, name map or gate id map hash that does not match the saved files, the history row or a verdict, and a gate compare output with another map sha256 than `batch.gateIdMap`

For old batches (legacy mode, revisions up to 132) it keeps the old check: the fixed 6,055 accepted names, the later 6,330 passing names and `additionalPassingResultsForAuditor`. The script does not read np-suite or the repeat runs of the tests and oracles. It reads every gate run in `gateRuns`. It does not itself prove that the protected set is complete, Go equivalence or Query completion. The independent verdicts must address those requirements. A script PASS alone is not compiler acceptance.

This is a required pre-acceptance check, not a replacement Cargo runner. It does not prevent someone from calling Cargo or Git directly, and it cannot prove that a human-authored review is true. Root must honor the rules for actions outside the check. Keep existing resource limits and actual tool permissions.

The current invocation must return STOP because no recovery batch is authorized or accepted. A successful setup test must not clear the real state.

The original check supports `phase: "initial-recovery"`. The authorized continuation requires an explicit reviewed extension for `phase: "recovery-continuation"`. Keep the complete history and require the saved continuation authorization. Do not add an ignore-regressions path or use an unrecorded phase change to bypass the initial limit.

The script checks limits in the supplied history. For a state directory it also stops when `history.jsonl` no longer starts with its committed copy. It cannot detect a rewrite that was committed. The auditor must verify the history that each batch carries forward against `batchRecords`. This limitation is not permission to reset a counter.

## Approved rule changes

The 2026-10-07 light path below is the newest rule change. Before it, the 2026-09-28 [protected set](#protected-set) change was the newest. The opt-in crate rule, its extensions and the roster carry-forward below apply only to batches that use the legacy roster. They stay so that old records still pass the check. The pin-bump rule applies to all batches.

Theo approved two scoped rule changes on 2026-09-25 for batch
`recovery-continuation-go-checker-port-1`. They are saved in
`acceptanceRuleChanges` in `current.json`. The check script applies them only to
that batch id.

- **Opt-in crate rule.** An additive opt-in crate (`ts_goport`) can be accepted
  when there is no new loss in the protected tests and it has its own Go parity
  evidence. The inherited losses in the pinned R96 full result (290 original
  accepted names and 15 later-pass names, all in `ts_checker`) are reported but
  do not block. The inherited counts must match exactly, and the script pins the
  R96 hash and both counts as constants. The 290 include 14 names that were
  already ABSENT at R96. All other protected names must be present and run, and
  no protected PASS may be newly lost.
- **Unbound history rows.** Revisions 97, 98 and 99 were measured before their
  source was saved. They keep a null source and a null result, and get no credit.

On 2026-09-25 Theo also said: "Going forward, answer every question yourself."
Under that delegation, root extended both rules to batch
`recovery-continuation-go-checker-port-2`, and then to batch
`recovery-continuation-go-checker-port-3`, and then to batch
`recovery-continuation-go-checker-port-4`, and then to batch
`recovery-continuation-go-checker-port-5`, and then to batch
`recovery-continuation-go-checker-port-6`, and then to batch
`recovery-continuation-go-checker-port-7`, and then to batch
`recovery-continuation-go-checker-port-8`, then to batch
`recovery-continuation-go-checker-port-9`, and then to batch
`recovery-continuation-go-checker-port-10`, and then to batch
`recovery-continuation-go-checker-port-11`, and then to batch
`recovery-continuation-go-checker-port-12`, and then to batch
`recovery-continuation-go-checker-port-13` (a pin-bump batch: its Go pin follows
the pin-bump rule below), and then to batch
`recovery-continuation-go-checker-port-14` (same Go pin as batch 13), and then to batch
`recovery-continuation-go-checker-port-15` (same Go pin), and then to batch
`recovery-continuation-go-checker-port-16` (same Go pin), and then to batch
`recovery-continuation-go-checker-port-17` (same Go pin), and then to batch
`recovery-continuation-go-checker-port-18` (same Go pin), each time with the same pins and
scope. Each extension is a separate entry in `acceptanceRuleChanges`. The
records name root as the extender. This is not a new direct approval by Theo,
and it does not widen the rules.

On 2026-09-27 Theo approved a standing **pin-bump batch** rule ("Approve bumps, start
now"). typescript-go moved into `microsoft/TypeScript` under `tsc/`, and the pinned
commit `dc37b5249` is tree-identical to `4d44e1c49` there. A pin-bump batch moves the Go
pin from O to N (first N: the v7.0.2 content). At N, "pinned Go" means Go at N:
- The oracle is rebuilt from N (`tsgo-oracle-<N12>`, sha256 recorded). Old oracle
  binaries and old evidence stay, so old verdicts remain reproducible.
- Go-side evidence (gate caches, corpora, typesyms dumps, LS and API goldens, Query and
  Hono oracle outputs, Go reference baselines) is re-recorded against N into new
  pin-keyed directories; nothing old is overwritten.
- Acceptance happens only at N, with both independent verdicts. The protected set must
  not lose a pass. When the bump renames or removes Go tests, give the old-name map
  with Go evidence at N (see "Pin bumps" in the protected set rules). A Rust expectation
  that pins old Go behavior changes only with evidence from Go at N and an old-to-new
  name mapping, reviewed independently. (Before 2026-09-28 this bullet protected the
  legacy cargo roster.)
- A tool change that the bump needs in a protected path (for example `gate.sh`,
  `bound2.sh` or the API batteries in `candidate.sh`) is listed path by path in the
  batch, and the reviewer judges each change (see "Protected paths" in the protected
  set rules).
- The plan and tooling are in `target/continuation-r97-goport/upstream/drift.md`. Root
  must not widen this rule by delegation.

### Goport-only roster carry-forward (retired)

On 2026-09-28 Theo approved a standing rule, saved in `acceptanceRuleChanges` with id
`goport-only-roster-carry-forward`, `batchId: "*"` and `standing: true`. When no file
outside `crates/ts_goport` changed since an earlier measured revision, a batch reused
that revision's legacy roster result instead of the 45-minute roster drive. The proof
was an equal `roster_fp.py` fingerprint, recorded in `batch.rosterFingerprint` and
`batch.rosterCarryForward`.

The [protected set](#protected-set) retired this rule on the same day, with the roster.
Batches that use the goport protected set have no roster to carry. The check script
keeps the rule only to check old records. No new batch can use it.

### Light path for docs, tooling and simple fixes (2026-10-07)

Theo approved it on 2026-10-07: "I don't think we need our full conformance and testing suite for simple fixes and readme changes. Feel free to YOLO those a little bit." (state note `theo-approvals-2026-10-07`).

- **Docs and README changes, and scripts and tooling fixes** that do not change a protected tool's results go to `main` directly. They need no revision and no verdicts.
- **A simple code fix** (small, focused, with a test that fails without it) needs:
  - its focused tests;
  - `goport-tests.sh` against the last accepted revision with 0 lost;
  - bin identity on the 4 gate projects.

  Then it merges to `main` without a revision. The next integration measures it in the full protected set as usual. A loss found there is a regression of that revision.
- **Changes to the checker, the loader or the language server that can move output** keep the full path: a lane, a skeptic, an integration and a revision with two verdicts.

Any other rule change still needs Theo's approval.

## Project target and reporting

Query core remains first. The milestone is complete ordinary diagnostics matching pinned Go, plus a deliberate type error reported correctly in a separate copy. Full type and symbol parity with pinned Go (the gate's typesyms dumps) remains a requirement.

Hono remains the cross-project check. Run it after a recovered shared operation, at the Query milestone and once per full work day on the latest accepted compiler during continued work. Do not claim the current compiler passes Hono from an older run.

Every progress report states the goport test passes and gate items retained or lost, exact diagnostic changes, whether ordinary Query completes and the source/date of the latest Hono check. New test coverage is separate. Unavailable diagnostics are not zero diagnostics.

## Setup ownership

Root owns these instructions, the saved state and the reset plan. During setup, `audit_accepted_roster` owns the small accountability check and its tooling tests. `reset_process_audit` reviews setup without editing it. This setup assignment does not authorize either agent to resume compiler work.
