use super::super::*;
#[derive(Clone, Copy)]
struct InsertionCase {
    name: &'static str,
    after: bool,
    moving: bool,
    index: i32,
    fail_default: bool,
    fail_move: usize,
    fail_copy: bool,
}
fn fixture_case(name: &'static str, after: bool, moving: bool, index: i32) -> InsertionCase {
    InsertionCase {
        name,
        after,
        moving,
        index,
        fail_default: false,
        fail_move: 0,
        fail_copy: false,
    }
}
fn success_cases() -> [InsertionCase; 6] {
    [
        fixture_case("before_copy", false, false, 1),
        fixture_case("after_move", true, true, 1),
        fixture_case("after_last_copy", true, false, 3),
        fixture_case("before_negative", false, false, -1),
        fixture_case("after_end", true, true, 4),
        fixture_case("after_negative", true, false, -1),
    ]
}
fn failing_case(name: &'static str, default: bool, shift: usize, copy: bool) -> InsertionCase {
    let mut case = fixture_case(name, false, name == "final_move_failure", 1);
    case.fail_default = default;
    case.fail_move = shift;
    case.fail_copy = copy;
    case
}
fn failure_cases() -> [InsertionCase; 5] {
    [
        failing_case("default_failure", true, 0, false),
        failing_case("first_move_failure", false, 1, false),
        failing_case("second_move_failure", false, 2, false),
        failing_case("final_copy_failure", false, 0, true),
        failing_case("final_move_failure", false, 4, false),
    ]
}
fn configure_case(case: InsertionCase) {
    INSERTION_PROBE.with(|probe| {
        let mut probe = probe.borrow_mut();
        probe.fail_default = case.fail_default;
        probe.fail_move = case.fail_move;
        probe.fail_copy = case.fail_copy;
    });
}
fn final_assignment(target: &mut InsertionItem, input: &mut InsertionItem, moving: bool) {
    if moving {
        move_assignment(target, input);
    } else {
        copy_assignment(target, input);
    }
}
fn apply_case(
    array: &mut DynamicArray<InsertionItem>,
    input: &mut InsertionItem,
    case: InsertionCase,
) {
    let assign = |target: &mut InsertionItem| final_assignment(target, input, case.moving);
    let result = if case.after {
        array.insert_after_with_assignment(case.index, move_assignment, assign)
    } else {
        array.insert_before_with_assignment(case.index, move_assignment, assign)
    };
    if result.is_err() {
        INSERTION_PROBE.with(|probe| probe.borrow_mut().events.push_str("E;"));
    }
}
fn output_row(
    array: &DynamicArray<InsertionItem>,
    input: &InsertionItem,
    case: InsertionCase,
) -> String {
    let mut output = format!(
        "{} {} {} {}",
        case.name,
        observed_events(),
        array.size(),
        input.value
    );
    for value in observed_values(array) {
        write!(output, " {value}").unwrap();
    }
    output.push('\n');
    output
}
fn case_row(case: InsertionCase) -> String {
    let mut array = insertion_fixture();
    let mut input = InsertionItem { value: 99 };
    configure_case(case);
    if catch_unwind(AssertUnwindSafe(|| {
        apply_case(&mut array, &mut input, case)
    }))
    .is_err()
    {
        INSERTION_PROBE.with(|probe| probe.borrow_mut().events.push_str("E;"));
    }
    output_row(&array, &input, case)
}
#[test]
fn native_insertion_rows_match_full_assignment_traces() {
    let cases = success_cases();
    let mut actual: String = cases[..5].iter().copied().map(case_row).collect();
    actual.extend(failure_cases().map(case_row));
    actual.push_str(&case_row(fixture_case("before_first_move", false, true, 0)));
    actual.push_str(&case_row(cases[5]));
    assert_eq!(
        actual,
        include_str!("../../../../tools/oracle/tkernel_dynamic_native_insertion.expected")
    );
}
