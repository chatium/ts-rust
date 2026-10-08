//! Port-only tests of watch builds after a deduplicated package copy gets
//! its own version (watchfix1 item 1; the int43 reviewer item 5 and the
//! watchfree1 skeptic's session C, op 7).
//!
//! Go keeps one copy of a package that two `node_modules` directories hold
//! with the same name and version (compiler/filesparser.go:448): the other
//! copy's path maps to the kept file (:473), but its parse stays in the
//! watcher's source file cache (execute/watcher.go:516 keeps every path of
//! `FilesByPath`). When the copies get different versions, the next build
//! takes that parse as a program file. The port published the left-out
//! parse as a store with the name only, so the next build stopped with
//! exit 70 ("program file ... was published as a store that is not a
//! source file"). The watch hosts now record each parse that they keep
//! (`watcher::note_kept_parse`).
//!
//! The expected errors are those of `tsgo-oracle-673a5f17d713` on the same
//! files with the OS watcher (`-w -p tsconfig.json --pretty false` and
//! `-b -w --pretty false`; target/continuation-r97-goport/watchfix1/watch,
//! sessions dd-w and ddts-bw).

use ts_goport::fswatch::{Event, EventKind};
use ts_goport::gostd::context;

use crate::support::child::{command_line_in_process, new_in_process_test_sys, run_test_in_child};
use crate::support::runner::TscInput;

const PROJECT: &str = "/home/src/workspaces/project";

/// The error of the first build: `pb` gets `pa`'s copy of `dep`, so its
/// `label` is a string.
const DEDUP_ERROR: &str =
    "src/use.ts(4,14): error TS2322: Type 'string' is not assignable to type 'number'.";

/// The project: `pa` and `pb` each hold a copy of `dep` 2.0.0 whose main
/// file has the extension `ext` (`.d.ts` or `.ts`). The copies differ:
/// `label` is a string in `pa`'s copy and a number in `pb`'s copy.
fn input(ext: &str) -> TscInput {
    let file = |name: &str, text: &str| (format!("{PROJECT}/{name}"), text.into());
    let mut files = vec![
        file(
            "tsconfig.json",
            r#"{"compilerOptions":{"strict":true,"module":"esnext","moduleResolution":"bundler","outDir":"out","rootDir":"src","types":[]},"include":["src"]}"#,
        ),
        file(
            "src/use.ts",
            "import { get as ga } from \"pa\";\nimport { get as gb } from \"pb\";\nexport const x: string = ga().label;\nexport const y: number = gb().label;\n",
        ),
    ];
    for (package, label) in [("pa", "string"), ("pb", "number")] {
        files.push(file(
            &format!("node_modules/{package}/package.json"),
            &format!(r#"{{ "name": "{package}", "version": "1.0.0", "types": "index.d.ts" }}"#),
        ));
        files.push(file(
            &format!("node_modules/{package}/index.d.ts"),
            "import type { D } from \"dep\";\nexport declare function get(): D;\n",
        ));
        files.push(file(
            &format!("node_modules/{package}/node_modules/dep/package.json"),
            &dep_package_json("2.0.0", ext),
        ));
        files.push(file(
            &format!("node_modules/{package}/node_modules/dep/index{ext}"),
            &format!("export interface D {{ label: {label} }}\n"),
        ));
    }
    TscInput {
        files: files.into_iter().collect(),
        ..Default::default()
    }
}

fn dep_package_json(version: &str, ext: &str) -> String {
    format!(r#"{{ "name": "dep", "version": "{version}", "types": "index{ext}" }}"#)
}

/// Runs `args` on the project of `input(ext)`, then sets the version of
/// `pb`'s copy of `dep` to each of `versions`, one watch cycle each. Checks
/// the output of the first build and of each cycle: whether it has
/// `DEDUP_ERROR` (`errors[0]` for the first build) and that it ends with
/// the error count.
fn run(args: &[&str], ext: &str, versions: &[&str], errors: &[bool]) {
    let sys = new_in_process_test_sys(&input(ext));
    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    let result = command_line_in_process(&context::background(), &sys, &args);
    let mut w = result
        .watcher
        .expect("expected Watcher to be non-nil in watch mode");
    let fs = sys.fs_from_file_map();
    let package_json = format!("{PROJECT}/node_modules/pb/node_modules/dep/package.json");
    let check = |step: &str, error: bool| {
        let out = sys.output_text();
        assert_eq!(out.contains(DEDUP_ERROR), error, "{step}: {out}");
        let found = if error {
            "Found 1 error. Watching for file changes."
        } else {
            "Found 0 errors. Watching for file changes."
        };
        assert!(out.contains(found), "{step}: {out}");
    };
    check("first build", errors[0]);
    for (i, version) in versions.iter().enumerate() {
        sys.set_output_bytes(Vec::new());
        let _ = fs.write_file(&package_json, &dep_package_json(version, ext));
        sys.mock_watch_backend().send_events(vec![Event {
            kind: EventKind::Update,
            path: package_json.clone(),
        }]);
        w.do_cycle();
        check(
            &format!("version {version} (cycle {})", i + 1),
            errors[i + 1],
        );
    }
}

#[test]
fn watch_takes_the_kept_parse_of_a_deduplicated_package_copy() {
    run_test_in_child(
        "tsctests::watch_dedup_package::watch_takes_the_kept_parse_of_a_deduplicated_package_copy",
        || {
            // Go: the first build has the error, and the later builds of
            // `-w` have none.
            run(
                &["--watch", "--pretty", "false"],
                ".d.ts",
                &["2.0.1", "2.0.0", "2.0.1"],
                &[true, false, false, false],
            );
        },
    );
}

#[test]
fn build_watch_takes_the_kept_parse_of_a_deduplicated_ts_package_copy() {
    run_test_in_child(
        "tsctests::watch_dedup_package::build_watch_takes_the_kept_parse_of_a_deduplicated_ts_package_copy",
        || {
            // The `.ts` copy: the `-b` host noted only `.d.ts` and `.json`
            // parses before. Go: the error comes back with the shared
            // version.
            run(
                &["--build", "--watch", "--pretty", "false"],
                ".ts",
                &["2.0.1", "2.0.0", "2.0.1"],
                &[true, false, true, false],
            );
        },
    );
}

/// `(made, dead)` file versions once the dead count reaches `dead` or 10 s
/// pass: a dropped version can die on the free thread.
fn file_versions(dead: usize) -> (usize, usize) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        ts_goport::program::wait_for_background_releases();
        let now = (
            ts_goport::ast::file_versions_made(),
            ts_goport::ast::dead_file_versions(),
        );
        if now.1 >= dead || std::time::Instant::now() > deadline {
            return now;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn watch_frees_each_new_parse_of_the_left_out_package_copy() {
    run_test_in_child(
        "tsctests::watch_dedup_package::watch_frees_each_new_parse_of_the_left_out_package_copy",
        || {
            // followups31 (R177 reviewer item 2): edits of `pb`'s copy of
            // `dep`, which keeps the shared version, so each build leaves
            // it out. Go builds again after each edit and gives the error
            // of the first build (followups31/runs/ddleft-w, Go N). Each
            // new parse of the copy is a freeable version
            // (`watcher::note_kept_parse`), and the one before it dies.
            // Before watchfix1 each one was a static store that lived
            // until exit (made and dead stay 0): 9 MiB for each edit of a
            // 335 KB copy.
            let sys = new_in_process_test_sys(&input(".d.ts"));
            let args: Vec<String> = ["--watch", "--pretty", "false"]
                .iter()
                .map(|arg| arg.to_string())
                .collect();
            let result = command_line_in_process(&context::background(), &sys, &args);
            let mut w = result
                .watcher
                .expect("expected Watcher to be non-nil in watch mode");
            let fs = sys.fs_from_file_map();
            let copy = format!("{PROJECT}/node_modules/pb/node_modules/dep/index.d.ts");
            assert_eq!(file_versions(0), (0, 0), "first build");
            for edit in 1..=3 {
                sys.set_output_bytes(Vec::new());
                let text = format!("export interface D {{ label: number; edit{edit}: true }}\n");
                let _ = fs.write_file(&copy, &text);
                sys.mock_watch_backend().send_events(vec![Event {
                    kind: EventKind::Update,
                    path: copy.clone(),
                }]);
                w.do_cycle();
                let out = sys.output_text();
                assert!(
                    out.contains(DEDUP_ERROR)
                        && out.contains("Found 1 error. Watching for file changes."),
                    "edit {edit}: {out}"
                );
                assert_eq!(file_versions(edit - 1), (edit, edit - 1), "edit {edit}");
            }
        },
    );
}
