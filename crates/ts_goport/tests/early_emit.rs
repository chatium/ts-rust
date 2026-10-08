//! The early emit of `tsc -p` and `tsc -b` with an incremental program
//! (`execute::incremental::Program::start_emit`). Each checker gets its emit
//! jobs right behind its check job, so it emits when its own check ends and
//! the emit pool runs during the check. The outputs, the build info, stdout
//! and the exit code must be the same as with `GOPORT_EARLY_EMIT=0`, which
//! keeps Go's barrier (the emit starts after the whole check).
//!
//! The fixture is `fixtures/emit_pool`: 4 program files and the es2020 lib
//! files, so 4 checkers get files. With declarations the JS parts of
//! `shapes.ts`, `legacy.js` and `index.ts` go to the emit pool and the d.ts
//! parts stay on the checker threads. `GOPORT_EMIT_THREADS=2` turns the pool
//! on at any core count. Each `tsgo` run writes to the same new directory
//! under the system temp dir, so the source map and build info paths are
//! the same. A passing test deletes it.
//!
//! The rule test checks `emit_can_start_with_check` on the same fixture:
//! each case that a check could see the outputs of keeps the barrier. Do not
//! set `GOPORT_EARLY_EMIT=0` for this test. Its F2 cases load
//! `tsconfig.rules.json` (the same config with `"exclude": []`): without an
//! `exclude`, the config excludes `outDir` and `declarationDir` from its
//! files, so no program file would be inside them.
//!
//! The emit-only tests make their own `tsc -b` solution in a scratch dir: a
//! task that checks nothing (every file's semantic diagnostics cached,
//! `noCheck`, or a syntax error) must emit on its checker threads too, so it
//! finishes when its own emit ends, as a Go builder does. The global
//! diagnostics test makes its own project there too.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ts_goport::core::enter_program;
use ts_goport::emitter::program_emit::emit_can_start_with_check;
use ts_goport::flags::{ModuleKind, ModuleResolutionKind};
use ts_goport::options::{CompilerOptions, Tristate};
use ts_goport::program::{release_program, try_load_version};

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/emit_pool");

const CONFIG: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/emit_pool/tsconfig.json"
);

/// `CONFIG` with `"exclude": []`, so program files can be in the output
/// directories.
const RULES_CONFIG: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/emit_pool/tsconfig.rules.json"
);

/// What one `tsgo` run wrote and printed.
#[derive(Debug, PartialEq)]
struct Run {
    /// The bytes of each file under the out dir, by relative path.
    files: BTreeMap<String, Vec<u8>>,
    stdout: String,
    status: Option<i32>,
}

#[test]
fn early_emit_writes_what_the_barrier_writes() {
    let root = scratch_dir();
    let out = root.join("out");
    let cases: [(&str, &[&str]); 3] = [
        ("js and d.ts", &[]),
        (
            "js only",
            &["--declaration", "false", "--declarationMap", "false"],
        ),
        ("d.ts only", &["--emitDeclarationOnly"]),
    ];
    for (case, extra) in cases {
        let barrier = tsgo(&out, extra, false);
        let early = tsgo(&out, extra, true);
        assert_eq!(
            barrier.status,
            Some(0),
            "{case}: the fixture must compile without diagnostics, so the check is sent: {}",
            barrier.stdout
        );
        assert!(
            barrier.files.contains_key("tsconfig.tsbuildinfo")
                && barrier.files.keys().any(|name| std::path::Path::new(name)
                    .extension()
                    .is_some_and(|e| e == "js")
                    || name.ends_with(".d.ts")),
            "{case}: the run must write outputs and build info: {:?}",
            barrier.files.keys()
        );
        assert_eq!(early, barrier, "{case}: the early emit against the barrier");
    }
    fs::remove_dir_all(&root).unwrap_or_else(|error| panic!("remove {}: {error}", root.display()));
}

#[test]
fn build_early_emit_writes_what_the_barrier_writes() {
    let root = scratch_dir();
    let out = root.join("out");
    // The fixture as a composite project, with its outputs and build info
    // in the scratch dir. The base config's `include` and `rootDir` stay
    // relative to the fixture.
    let config = root.join("tsconfig.json");
    let text = format!(
        r#"{{
  "extends": "{FIXTURE}/tsconfig.json",
  "compilerOptions": {{
    "composite": true,
    "outDir": "{out}",
    "tsBuildInfoFile": "{out}/tsconfig.tsbuildinfo"
  }}
}}
"#,
        out = out.display()
    );
    fs::write(&config, text).unwrap_or_else(|error| panic!("write {}: {error}", config.display()));
    let barrier = tsgo_build(&config, &out, false);
    let early = tsgo_build(&config, &out, true);
    assert_eq!(
        barrier.status,
        Some(0),
        "the fixture must build without diagnostics, so the check is sent: {}",
        barrier.stdout
    );
    assert!(
        barrier.files.contains_key("tsconfig.tsbuildinfo")
            && barrier.files.keys().any(|name| std::path::Path::new(name)
                .extension()
                .is_some_and(|e| e == "js")
                || name.ends_with(".d.ts")),
        "the build must write outputs and build info: {:?}",
        barrier.files.keys()
    );
    assert_eq!(early, barrier, "the early emit against the barrier");
    fs::remove_dir_all(&root).unwrap_or_else(|error| panic!("remove {}: {error}", root.display()));
}

/// K2 (tscbemit1). `tsc -b --builders 2` on the solution p1 p2 p3, with no
/// references. Only the emits of p1 and p2 are pending (`--noEmit` builds
/// after an edit checked them): p1 is one root with a large emit, p2 a small
/// writer whose new output adds `v2`, and p3 imports `v2` from p2's output.
/// A Go builder writes its task's outputs when the task's emit ends, so p2
/// ends first, its builder takes p3, and p3 loads after p2 wrote: exit 0,
/// no output (Go N gives that). Before tscbemit1 a task with nothing to
/// check emitted on the loading thread when it finished, so p1 and p2
/// finished in build order and p3 loaded before p2 wrote (TS2305).
#[test]
fn build_emit_only_task_finishes_when_its_emit_ends() {
    assert_eq!(
        build_emit_only_solution("", &[]),
        (Some(0), String::new()),
        "p2 finishes before p1, so p3 loads after p2 wrote v2"
    );
}

/// tscbemit2: as `build_emit_only_task_finishes_when_its_emit_ends`, with
/// `noCheck` in p1. Go's p1 does no semantic check either
/// (`GetSemanticDiagnostics` returns nil), so it only emits: exit 0, no
/// output (Go N gives that). Before tscbemit2 the port started no early
/// emit for it, so it finished first, in build order (TS2305).
#[test]
fn build_no_check_task_finishes_when_its_emit_ends() {
    assert_eq!(
        build_emit_only_solution(r#", "noCheck": true"#, &[]),
        (Some(0), String::new()),
        "p2 finishes before p1, so p3 loads after p2 wrote v2"
    );
}

/// tscbemit2: as `build_emit_only_task_finishes_when_its_emit_ends`, with a
/// syntax error in p1. Go's `GetDiagnosticsOfAnyProgram` stops at the
/// syntactic diagnostics, so p1 only emits: only p1's TS1109 (Go N gives
/// that). Before tscbemit2 the port started no early emit for it, so it
/// finished first, in build order, and p3 also gave TS2305.
#[test]
fn build_task_with_syntax_errors_finishes_when_its_emit_ends() {
    assert_eq!(
        build_emit_only_solution("", &[("bad.ts", "export const bad = ;\n")]),
        (
            Some(2),
            "p1/src/bad.ts(1,20): error TS1109: Expression expected.\n".to_owned()
        ),
        "p2 finishes before p1, so p3 loads after p2 wrote v2"
    );
}

/// tscbemit2: Go reads the global diagnostics before the emit, also with
/// `noCheck` or program diagnostics (here TS6053, a missing file), where
/// `start_check` does not read them, so the early emit reads them first.
/// With `lib` es5 the d.ts emit of a generator asks for the missing global
/// type `IterableIterator`, which adds TS2318 to the checker's global
/// diagnostics. Go never reports it (it read them before), so `tsc -p` and
/// `tsc -b` give only the program diagnostics (Go N gives that). A read after
/// the early emit adds TS2318.
#[test]
fn early_emit_reads_the_global_diagnostics_before_the_emit() {
    let missing = concat!(
        "error TS6053: File '{root}/src/missing.ts' not found.\n",
        "  The file is in the program because:\n",
        "    Part of 'files' list in tsconfig.json\n"
    );
    for (options, files, expected) in [
        (r#", "noCheck": true"#, r#""include": ["src"]"#, None),
        (
            "",
            r#""files": ["src/a.ts", "src/missing.ts"]"#,
            Some(missing),
        ),
    ] {
        let root = scratch_dir();
        let config = format!(
            r#"{{"compilerOptions": {{"composite": true, "strict": true, "target": "es2022",
  "module": "esnext", "moduleResolution": "bundler", "outDir": "dist", "rootDir": "src",
  "skipLibCheck": true, "lib": ["es5"]{options}}}, {files}}}"#
        );
        fs::write(root.join("tsconfig.json"), config).expect("write the config");
        fs::create_dir(root.join("src")).expect("create src");
        fs::write(root.join("src/a.ts"), "export function* g() { yield 1; }\n")
            .expect("write a.ts");
        let expected = expected.map_or_else(String::new, |text| {
            text.replace("{root}", &root.display().to_string())
        });
        let status = if expected.is_empty() { 0 } else { 2 };
        for args in [
            &["-p", "tsconfig.json"][..],
            &["-b", "tsconfig.json", "--force"],
        ] {
            let output = Command::new(env!("CARGO_BIN_EXE_tsgo"))
                .current_dir(&root)
                .args(args)
                .args(["--pretty", "false"])
                .env("GOPORT_EARLY_EMIT", "1")
                .output()
                .expect("run tsgo");
            assert_eq!(
                (
                    output.status.code(),
                    String::from_utf8_lossy(&output.stdout).into_owned()
                ),
                (Some(status), expected.clone()),
                "tsgo {args:?} with{options} and {files}: no TS2318 from the emit"
            );
        }
        fs::remove_dir_all(&root)
            .unwrap_or_else(|error| panic!("remove {}: {error}", root.display()));
    }
}

/// Makes the solution of `build_emit_only_task_finishes_when_its_emit_ends`
/// in a scratch dir, with `p1_options` added to p1's compiler options and
/// `p1_files` added to p1's `src`, runs its builds and returns the exit code
/// and stdout of the last one (`--builders 2`).
fn build_emit_only_solution(p1_options: &str, p1_files: &[(&str, &str)]) -> (Option<i32>, String) {
    let root = scratch_dir();
    let write = |path: &str, text: &str| {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("a file in a project"))
            .unwrap_or_else(|error| panic!("create the dir of {}: {error}", path.display()));
        fs::write(&path, text).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
    };
    let tsgo_b = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_tsgo"))
            .current_dir(&root)
            .arg("-b")
            .args(args)
            .args(["--pretty", "false"])
            .env("GOPORT_EARLY_EMIT", "1")
            .output()
            .expect("run tsgo -b")
    };
    // The code of 1,500 modules in one file: its emit takes far longer than
    // the load and emit of p2.
    let big = |tag: &str| {
        use std::fmt::Write as _;
        let mut text = String::new();
        for i in 0..1500 {
            write!(
                text,
                "export interface I{i} {{ a: number; b: string; c{i}: boolean }}\n\
                 export function f{i}(x: I{i}): I{i} {{ return {{ ...x }}; }}\n\
                 export class C{i} {{ constructor(public v: I{i}) {{}} get(): I{i} {{ return f{i}(this.v); }} }}\n\
                 export const k{i}: number = {i}; // {tag}\n"
            )
            .expect("write to a String");
        }
        text
    };
    let config = |extra: &str| {
        format!(
            r#"{{"compilerOptions": {{"composite": true, "strict": true, "target": "es2022",
  "module": "esnext", "moduleResolution": "bundler", "outDir": "dist", "rootDir": "src",
  "skipLibCheck": true{extra}}}, "include": ["src"]}}"#
        )
    };
    write(
        "tsconfig.json",
        r#"{"files": [], "references": [{"path": "./p1"}, {"path": "./p2"}, {"path": "./p3"}]}"#,
    );
    write("p1/tsconfig.json", &config(p1_options));
    for project in ["p2", "p3"] {
        write(&format!("{project}/tsconfig.json"), &config(""));
    }
    write("p1/src/index.ts", &big("v1"));
    for (name, text) in p1_files {
        write(&format!("p1/src/{name}"), text);
    }
    write("p2/src/s0.ts", "export const s0 = 0;\n");
    write("p2/src/s1.ts", "export const s1 = 1;\n");
    write("p2/src/index.ts", "export const v1 = 1;\n");
    write(
        "p3/src/a.ts",
        "import { v1, v2 } from \"../../p2/dist/index\";\nexport const a = v1 + v2;\n",
    );
    // The cold build: p3 cannot see `v2` yet.
    tsgo_b(&["tsconfig.json"]);
    write("p1/src/index.ts", &big("v2"));
    write(
        "p2/src/index.ts",
        "export const v1 = 1;\nexport const v2 = 2;\n",
    );
    // p1's errors are part of the case; p2 must check clean.
    tsgo_b(&["p1", "--noEmit"]);
    let output = tsgo_b(&["p2", "--noEmit"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "tsc -b p2 --noEmit: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let output = tsgo_b(&["tsconfig.json", "--builders", "2"]);
    fs::remove_dir_all(&root).unwrap_or_else(|error| panic!("remove {}: {error}", root.display()));
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

#[test]
fn rules_keep_the_barrier_when_a_check_could_see_the_outputs() {
    let out_dir = std::env::temp_dir()
        .join("goport-early-emit-rules")
        .to_string_lossy()
        .into_owned();
    // As in `early_emit_writes_what_the_barrier_writes`.
    let out = out_dir.clone();
    assert!(
        can_start(move |options| options.out_dir = out),
        "the fixture with a temp outDir must start its emit with the check"
    );

    // F1: `index.ts` imports "./shapes" without an extension.
    assert!(!can_start(|options| {
        options.module = ModuleKind::NODE_NEXT;
        options.module_resolution = ModuleResolutionKind::NODE_NEXT;
    }));
    // F2: the program files are inside the outDir or declarationDir.
    assert!(!can_start_with(RULES_CONFIG, |options| {
        options.out_dir = format!("{FIXTURE}/src");
    }));
    assert!(!can_start_with(RULES_CONFIG, |options| {
        options.declaration_dir = FIXTURE.to_string();
    }));
    // F3: an output directory under `node_modules`.
    let under_node_modules = format!("{out_dir}/node_modules/out");
    assert!(!can_start(move |options| {
        options.out_dir = under_node_modules;
    }));
    // F4 and the option rules.
    assert!(!can_start(|options| {
        options.preserve_symlinks = Tristate::True;
    }));
    assert!(!can_start(|options| {
        options.no_emit_on_error = Tristate::True;
    }));
    assert!(!can_start(|options| {
        options.single_threaded = Tristate::True;
    }));
}

/// Loads the fixture with `edit` applied to its options and returns
/// `emit_can_start_with_check` for it.
fn can_start(edit: impl FnOnce(&mut CompilerOptions)) -> bool {
    can_start_with(CONFIG, edit)
}

/// `can_start` with the fixture config `config`.
fn can_start_with(config: &str, edit: impl FnOnce(&mut CompilerOptions)) -> bool {
    let program = try_load_version(config, edit)
        .unwrap_or_else(|error| panic!("cannot load {config}: {error}"));
    let can_start = {
        let _scope = enter_program(Some(program));
        emit_can_start_with_check()
    };
    release_program(program);
    can_start
}

/// Runs `tsgo -p` on the fixture as an incremental program, with `out` as
/// the out dir and the build info in it, and `extra` arguments. `early`
/// false sets `GOPORT_EARLY_EMIT=0`. It removes `out` first and returns what
/// the run wrote there and printed.
fn tsgo(out: &Path, extra: &[&str], early: bool) -> Run {
    if out.exists() {
        fs::remove_dir_all(out).unwrap_or_else(|error| panic!("remove {}: {error}", out.display()));
    }
    let output = Command::new(env!("CARGO_BIN_EXE_tsgo"))
        .args(["-p", CONFIG, "--incremental", "--outDir"])
        .arg(out)
        .arg("--tsBuildInfoFile")
        .arg(out.join("tsconfig.tsbuildinfo"))
        .args(["--listEmittedFiles", "--pretty", "false"])
        .args(extra)
        .env("GOPORT_EMIT_THREADS", "2")
        .env("GOPORT_EARLY_EMIT", if early { "1" } else { "0" })
        .output()
        .expect("run tsgo");
    let mut files = BTreeMap::new();
    if out.exists() {
        read_files(out, out, &mut files);
    }
    Run {
        files,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        status: output.status.code(),
    }
}

/// Runs `tsgo -b` on `config`, whose outputs and build info are in `out`.
/// `early` false sets `GOPORT_EARLY_EMIT=0`. It removes `out` first, so the
/// project is out of date, and returns what the run wrote there and printed.
fn tsgo_build(config: &Path, out: &Path, early: bool) -> Run {
    if out.exists() {
        fs::remove_dir_all(out).unwrap_or_else(|error| panic!("remove {}: {error}", out.display()));
    }
    let output = Command::new(env!("CARGO_BIN_EXE_tsgo"))
        .arg("-b")
        .arg(config)
        .args(["--listEmittedFiles", "--pretty", "false"])
        .env("GOPORT_EMIT_THREADS", "2")
        .env("GOPORT_EARLY_EMIT", if early { "1" } else { "0" })
        .output()
        .expect("run tsgo -b");
    let mut files = BTreeMap::new();
    if out.exists() {
        read_files(out, out, &mut files);
    }
    Run {
        files,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        status: output.status.code(),
    }
}

/// Reads every file under `dir` into `files`, by path relative to `root`.
fn read_files(root: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
    {
        let path = entry.expect("out dir entry").path();
        if path.is_dir() {
            read_files(root, &path, files);
        } else {
            let name = path
                .strip_prefix(root)
                .expect("a path under the out dir")
                .to_string_lossy()
                .into_owned();
            let bytes =
                fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            files.insert(name, bytes);
        }
    }
}

/// A new directory under the system temp dir, by its real path (the
/// program sees real paths).
fn scratch_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after 1970")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("goport-early-emit-{}-{nanos}", std::process::id()));
    fs::create_dir(&dir).unwrap_or_else(|error| panic!("create {}: {error}", dir.display()));
    fs::canonicalize(&dir).expect("canonical scratch dir")
}
