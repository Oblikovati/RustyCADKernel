use super::super::*;
use rustycad_tkernel::ncollection::ArrayError;
use std::{fmt::Debug, mem::ManuallyDrop};
unsafe fn public_policy<I: TryInto<usize> + Copy + Debug>(
    array: &mut DynamicArray<ReplacementItem>,
    index: I,
    request: i32,
    mode: ReplacementMode,
    trace: &ReplacementTrace,
) -> Result<*const ReplacementItem, ArrayError> {
    let construct = || ReplacementItem::new(request, trace);
    use ReplacementItem as Item;
    // Outer fixture suspends automatic Drop and repairs the dead slot on destructive failures.
    unsafe {
        match mode {
            ReplacementMode::Assign => {
                array.emplace_value_with_assignment(index, construct, Item::assign_from)
            }
            ReplacementMode::Move | ReplacementMode::Copy => {
                emplace_transfer(array, index, construct, mode)
            }
            ReplacementMode::Direct => {
                array.emplace_value_with_direct_replacement(index, construct)
            }
        }
        .map(|value| value as *const Item)
    }
}
fn scenario(
    name: &str,
    request: i32,
    live: bool,
    mode: ReplacementMode,
    index: i32,
    wide: bool,
) -> String {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut array = ManuallyDrop::new(DynamicArray::with_increment(2));
    array.emplace_append(|| ReplacementItem::new(1, &trace));
    take_trace(&trace);
    let outcome = catch_unwind(AssertUnwindSafe(|| unsafe {
        if wide {
            public_policy(&mut array, index as usize, request, mode, &trace)
        } else {
            public_policy(&mut array, index, request, mode, &trace)
        }
    }));
    let row = outcome_row(name, &array, &outcome, live, &trace);
    // We never read/drop a dead slot. Reacquire its raw allocation pointer before repair.
    unsafe { repair_and_drop(&mut array, live, &trace) };
    assert_eq!(Rc::strong_count(&trace), 1);
    row
}
fn outcome_row(
    name: &str,
    array: &DynamicArray<ReplacementItem>,
    outcome: &std::thread::Result<Result<*const ReplacementItem, ArrayError>>,
    live: bool,
    trace: &ReplacementTrace,
) -> String {
    let observed_live = Rc::strong_count(trace) - 1;
    assert_eq!(observed_live, usize::from(live));
    let failed = !matches!(outcome, Ok(Ok(_)));
    let same = match outcome {
        Ok(Ok(pointer)) => *pointer == &array[0],
        _ => false,
    };
    let value = visible_value(array, live);
    let events = take_trace(trace);
    let events = if events.is_empty() { "none" } else { &events };
    format!(
        "{name} threw={} size={} live={} same={} value={value} trace={events}",
        usize::from(failed),
        array.size(),
        observed_live,
        usize::from(same)
    )
}
fn visible_value(array: &DynamicArray<ReplacementItem>, live: bool) -> String {
    if live {
        return array[0].value.to_string();
    }
    "dead".into()
}
unsafe fn emplace_transfer<I: TryInto<usize> + Copy + Debug>(
    array: &mut DynamicArray<ReplacementItem>,
    index: I,
    construct: impl FnOnce() -> ReplacementItem,
    mode: ReplacementMode,
) -> Result<&mut ReplacementItem, ArrayError> {
    let transfer = match mode {
        ReplacementMode::Move => ReplacementItem::transfer_move,
        _ => ReplacementItem::transfer_copy,
    };
    // The enclosing fixture owns the same reconstruction obligations as public_policy.
    unsafe { array.emplace_value_with_reconstruction(index, construct, transfer) }
}
unsafe fn repair_and_drop(
    array: &mut ManuallyDrop<DynamicArray<ReplacementItem>>,
    live: bool,
    trace: &ReplacementTrace,
) {
    if !live {
        let slot = array.storage_extension().change_block_at(0).unwrap();
        // Only repair the constructor-failed slot; other slots retain native logical ownership.
        unsafe { slot.write(ReplacementItem::new(5, trace)) };
    }
    // All logical slots are initialized again, so ordinary array destruction is now valid.
    unsafe { ManuallyDrop::drop(array) };
}
macro_rules! emplace_case {
    ($name:ident,$request:expr,$live:expr,$mode:ident,$index:expr,$wide:expr) => {
        #[test]
        fn $name() {
            let expected =
                include_str!("../../../../tools/oracle/tkernel_dynamic_emplace_policies.expected")
                    .lines()
                    .find(|row| row.starts_with(concat!(stringify!($name), " ")))
                    .unwrap();
            assert_eq!(
                scenario(
                    stringify!($name),
                    $request,
                    $live,
                    ReplacementMode::$mode,
                    $index,
                    $wide
                ),
                expected
            );
        }
    };
}
emplace_case!(assign, 2, true, Assign, 0, false);
emplace_case!(assign_ctor_fail, 99, true, Assign, 0, true);
emplace_case!(assign_fail, 77, true, Assign, 0, false);
emplace_case!(move_success, 2, true, Move, 0, false);
emplace_case!(move_ctor_fail, 99, true, Move, 0, true);
emplace_case!(copy, 2, true, Copy, 0, true);
emplace_case!(copy_ctor_fail, 99, true, Copy, 0, false);
emplace_case!(copy_transfer_fail, 88, false, Copy, 0, true);
emplace_case!(direct, 2, true, Direct, 0, false);
emplace_case!(direct_ctor_fail, 99, false, Direct, 0, true);
emplace_case!(direct_negative, 2, true, Direct, -1, false);
emplace_case!(copy_negative, 2, true, Copy, -1, false);
emplace_case!(assign_negative, 2, true, Assign, -1, false);
#[test]
fn new_index_constructs_gaps_without_invoking_transfer() {
    let mut array = DynamicArray::<i32>::with_increment(2);
    // New slots stay under the existing gap-construction ownership policy.
    unsafe {
        array
            .emplace_value_with_reconstruction(
                2_usize,
                || 7,
                |_| panic!("index 2: expected no transfer"),
            )
            .unwrap();
    }
    assert_eq!(array.iter().copied().collect::<Vec<_>>(), [0, 0, 7]);
}
#[test]
fn direct_policy_constructs_new_index_without_destroying_gaps() {
    let mut array = DynamicArray::<i32>::with_increment(2);
    // No destructive slot is selected, and this constructor cannot panic.
    unsafe {
        array
            .emplace_value_with_direct_replacement(2_i32, || 9)
            .unwrap();
    }
    assert_eq!(array.iter().copied().collect::<Vec<_>>(), [0, 0, 9]);
}
#[test]
fn new_index_constructs_gaps_without_invoking_assignment() {
    let mut array = DynamicArray::<i32>::with_increment(2);
    array
        .emplace_value_with_assignment(
            2_usize,
            || 11,
            |_, _| panic!("index 2: expected no assignment"),
        )
        .unwrap();
    assert_eq!(array.iter().copied().collect::<Vec<_>>(), [0, 0, 11]);
}
