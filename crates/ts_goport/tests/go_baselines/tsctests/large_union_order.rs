//! Port-only test of the member order of a union over 4,096 members
//! (bigsort1, `Checker::sort_large_union_types` in checker/utilities_p1.rs).
//!
//! Go sorts the members of each union with `slices.SortStableFunc(types,
//! CompareTypes)` (checker/checker.go:26275 addTypesToUnion,
//! checker/utilities.go:414 CompareTypes). The port sorts a list over
//! `KEYED_SORT_MAX` (4,096) types with a table of keys, which must give
//! Go's order. Here a tuple of 5,729 members in a shuffled order (string
//! literals, interfaces, 5,120 intersections of two interfaces, and 569
//! repeats) is indexed with `number`, and the error prints the union with
//! `noErrorTruncation`. The expected text is the output of
//! `tsgo-oracle-673a5f17d713 -p tsconfig.json --pretty false` on the same
//! files (sha256 8c686304f0f1f1fa843b405bc06d9a336f6e3ded1c70ceff96feb6885465024d,
//! followups32/bigsort): the strings, then the interfaces, then the
//! intersections, each by name (Go `strings.Compare`), and the
//! intersections by their first member, then their second.

use ts_goport::execute::tsc::ExitStatus;

use crate::support::child::run_command_in_child;
use crate::support::runner::TscInput;
use crate::support::test_sys::new_test_sys;

const PROJECT: &str = "/home/src/workspaces/project";

/// The union members of the tuple, in the shuffled source order.
fn members() -> Vec<String> {
    let intersections =
        || (0..64).flat_map(|i| (0..80).map(move |j| (i, j, format!("A{i} & B{j}"))));
    let mut members: Vec<String> = intersections().map(|(_, _, m)| m).collect();
    members.extend((0..20).map(|k| format!("\"s{k}\"")));
    members.extend((0..20).map(|k| format!("C{k}")));
    // Repeats: every 9th intersection again.
    members.extend(
        intersections()
            .filter(|(i, j, _)| (i * 80 + j) % 9 == 0)
            .map(|(_, _, m)| m),
    );
    // A xorshift64 Fisher-Yates shuffle.
    let mut state = 0x2545_f491_4f6c_dd1du64;
    for i in (1..members.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        members.swap(i, (state % (i as u64 + 1)) as usize);
    }
    members
}

/// The names `{prefix}0` to `{prefix}{count - 1}`, sorted as Go strings.
fn sorted_names(prefix: &str, count: usize) -> Vec<String> {
    let mut names: Vec<String> = (0..count).map(|k| format!("{prefix}{k}")).collect();
    names.sort();
    names
}

#[test]
fn a_union_over_4096_members_prints_in_go_order() {
    let interfaces = |name: &str, field: &str, count: usize| -> String {
        (0..count)
            .map(|k| format!("interface {name}{k} {{ {field}{k}: {k} }}\n"))
            .collect()
    };
    let members = members();
    assert_eq!(members.len(), 5729);
    let text = format!(
        "{}{}{}type Tup = [{}];\ndeclare const t: Tup[number];\nexport const n: number = t;\n",
        interfaces("A", "a", 64),
        interfaces("B", "b", 80),
        interfaces("C", "c", 20),
        members.join(", ")
    );
    let input = TscInput {
        files: [
            (format!("{PROJECT}/a.ts"), text.into()),
            (
                format!("{PROJECT}/tsconfig.json"),
                r#"{"compilerOptions":{"noEmit":true,"noErrorTruncation":true,"strict":true,"types":[]},"files":["a.ts"]}"#.into(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    };
    let sys = new_test_sys(&input, false);
    let args = ["-p", "tsconfig.json", "--pretty", "false"].map(String::from);
    let result = run_command_in_child(&sys, &args).unwrap_or_else(|err| panic!("tsgo: {err}"));
    assert!(result.unported.is_none(), "unported {:?}", result.unported);
    assert_eq!(
        result.status,
        ExitStatus::DiagnosticsPresentOutputsGenerated
    );

    let mut union: Vec<String> = sorted_names("s", 20)
        .into_iter()
        .map(|s| format!("\"{s}\""))
        .collect();
    union.extend(sorted_names("C", 20));
    for a in sorted_names("A", 64) {
        union.extend(sorted_names("B", 80).iter().map(|b| format!("({a} & {b})")));
    }
    assert_eq!(union.len(), 40 + 64 * 80);
    let expected = format!(
        "a.ts(167,14): error TS2322: Type '{}' is not assignable to type 'number'.\n  \
         Type 'string' is not assignable to type 'number'.\n",
        union.join(" | ")
    );
    // The test system adds the list of files after the diagnostics.
    let output = sys.output_text();
    let output = output
        .split("!!! List files start")
        .next()
        .unwrap_or_default();
    assert!(
        output == expected,
        "the union is not in Go's order: {} bytes, Go has {}; first difference at byte {}",
        output.len(),
        expected.len(),
        output
            .bytes()
            .zip(expected.bytes())
            .position(|(a, b)| a != b)
            .unwrap_or(output.len().min(expected.len()))
    );
}
