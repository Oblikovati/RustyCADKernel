//! Deleted native assignment selects temporary construction and explicit copy/move transfer.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{ArrayError, DynamicArray};
use std::{
    cell::RefCell,
    fmt::Write,
    mem::ManuallyDrop,
    panic::{AssertUnwindSafe, catch_unwind},
};
#[derive(Default)]
struct ReconstructProbe {
    trace: String,
    failure: &'static str,
    constructions: usize,
    live: usize,
}
thread_local! { static PROBE: RefCell<ReconstructProbe> = RefCell::default(); }
struct NonassignableItem(i32);
impl NonassignableItem {
    fn new(value: i32) -> Self {
        PROBE.with(|p| p.borrow_mut().live += 1);
        Self(value)
    }
    fn copy(source: &Self) -> Self {
        attempt('K', source.0);
        Self::new(source.0)
    }
    fn moved(source: &mut Self) -> Self {
        let value = source.0;
        source.0 = -7;
        attempt('M', value);
        Self::new(value)
    }
}
fn attempt(kind: char, value: i32) {
    PROBE.with(|p| {
        let mut p = p.borrow_mut();
        write!(p.trace, "{kind}{value};").unwrap();
        p.constructions += 1;
        assert!(
            !((p.failure == "construct" && p.constructions == 1)
                || (p.failure == "transfer" && p.constructions == 2)),
            "constructor {kind}{value}: expected successful phase"
        );
    });
}
impl Default for NonassignableItem {
    fn default() -> Self {
        PROBE.with(|p| {
            let mut p = p.borrow_mut();
            p.trace.push_str("D0;");
            assert_ne!(p.failure, "default", "default0: expected success");
        });
        Self::new(0)
    }
}
impl Drop for NonassignableItem {
    fn drop(&mut self) {
        PROBE.with(|p| {
            let mut p = p.borrow_mut();
            write!(p.trace, "X{};", self.0).unwrap();
            p.live -= 1;
        });
    }
}
fn fixture(failure: &'static str) -> ManuallyDrop<DynamicArray<NonassignableItem>> {
    PROBE.with(|p| *p.borrow_mut() = ReconstructProbe::default());
    let mut array = DynamicArray::with_increment(2);
    for value in [10, 20, 30, 40] {
        array.append(NonassignableItem::new(value));
    }
    PROBE.with(|p| p.borrow_mut().failure = failure);
    ManuallyDrop::new(array)
}
unsafe fn dispatch<I: TryInto<usize> + Copy + std::fmt::Debug>(
    array: &mut DynamicArray<NonassignableItem>,
    index: I,
    source: usize,
    mode: u8,
) -> Result<&mut NonassignableItem, ArrayError> {
    let transfer = |v: &mut NonassignableItem| {
        if mode == 2 || mode == 3 {
            NonassignableItem::moved(v)
        } else {
            NonassignableItem::copy(v)
        }
    };
    // Caller suspends automatic Drop before destructive replacement and repairs dead slots.
    unsafe {
        if mode == 0 || mode == 3 {
            return array.set_value_from_index_with_reconstruction(
                index,
                source,
                NonassignableItem::copy,
                transfer,
            );
        }
        array.set_value_moved_from_index_with_reconstruction(
            index,
            source,
            NonassignableItem::moved,
            transfer,
        )
    }
}
fn actual(name: &str, index: i32, source: usize, mode: u8, failure: &'static str) -> String {
    let mut array = fixture(failure);
    let original = array.value(source).unwrap() as *const NonassignableItem;
    let target = existing_slot(&mut array, index);
    let (threw, same) = invoke_caught(&mut array, index, source, mode);
    let dead = failure == "transfer";
    // Raw block addressing remains valid even when the aliased source lifetime has ended.
    let current = unsafe {
        array
            .storage_extension()
            .block_at(source / 2)
            .unwrap()
            .add(source % 2)
    };
    let row = observed(
        name,
        &array,
        index,
        dead,
        (threw, same, original == current),
    );
    repair_cleanup(&mut array, target, dead);
    row
}
fn existing_slot(
    array: &mut DynamicArray<NonassignableItem>,
    index: i32,
) -> Option<*mut NonassignableItem> {
    if !(0..4).contains(&index) {
        return None;
    }
    Some(unsafe {
        array
            .storage_extension()
            .change_block_at(index as usize / 2)
            .unwrap()
            .add(index as usize % 2)
    })
}
fn invoke_caught(
    array: &mut DynamicArray<NonassignableItem>,
    index: i32,
    source: usize,
    mode: u8,
) -> (bool, bool) {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        // The caller established ManuallyDrop before entry and retains it through catch_unwind.
        unsafe {
            if index >= 0 && index % 2 == 1 {
                dispatch(array, index as usize, source, mode)
            } else {
                dispatch(array, index, source, mode)
            }
        }
        .map(|v| v as *const NonassignableItem)
    }));
    let same = matches!(outcome, Ok(Ok(pointer)) if pointer == array.value(index).unwrap());
    (!matches!(outcome, Ok(Ok(_))), same)
}
fn repair_cleanup(
    array: &mut ManuallyDrop<DynamicArray<NonassignableItem>>,
    target: Option<*mut NonassignableItem>,
    dead: bool,
) {
    // Do not dereference a dead target, including an aliased source, before reconstruction.
    if dead {
        unsafe {
            target.unwrap().write(NonassignableItem::new(55));
        }
    }
    PROBE.with(|p| p.borrow_mut().failure = "");
    unsafe {
        ManuallyDrop::drop(array);
    }
    assert_eq!(PROBE.with(|p| p.borrow().live), 0);
}
fn observed(
    name: &str,
    array: &DynamicArray<NonassignableItem>,
    index: i32,
    dead: bool,
    flags: (bool, bool, bool),
) -> String {
    let mut row = observed_header(name, array.size(), flags);
    for i in 0..array.size() {
        if dead && i == index as usize {
            row.push_str(" dead");
        } else {
            write!(row, " {}", array.value(i).unwrap().0).unwrap();
        }
    }
    row
}
fn observed_header(name: &str, size: usize, flags: (bool, bool, bool)) -> String {
    let (threw, same, stable) = flags;
    PROBE.with(|p| {
        let p = p.borrow();
        format!(
            "{name} {} size={size} live={} threw={} same={} stable={}",
            if p.trace.is_empty() { "none" } else { &p.trace },
            p.live,
            usize::from(threw),
            usize::from(same),
            usize::from(stable)
        )
    })
}
fn expected(name: &str) -> &'static str {
    include_str!("../../../tools/oracle/tkernel_dynamic_internal_reconstruction.expected")
        .lines()
        .find(|row| row.split_whitespace().next() == Some(name))
        .unwrap()
}
fn matrix(mode: u8) {
    for index in [0, 1, 2, 3, 7] {
        for source in 0..4 {
            let name = format!("reconstruct_m{mode}_i{index}_s{source}");
            assert_eq!(actual(&name, index, source, mode, ""), expected(&name));
        }
    }
}
fn failures(mode: u8) {
    for index in [0, 1] {
        for failure in ["construct", "transfer"] {
            if mode == 3 && failure == "transfer" {
                continue;
            }
            let name = format!("reconstruct_m{mode}_i{index}_{failure}");
            assert_eq!(actual(&name, index, 1, mode, failure), expected(&name));
        }
    }
    for (suffix, index, failure) in [("default", 7, "default"), ("negative", -1, "")] {
        let name = format!("reconstruct_m{mode}_{suffix}");
        assert_eq!(actual(&name, index, 1, mode, failure), expected(&name));
    }
}
#[test]
fn copied_source_prefers_copy_transfer() {
    matrix(0);
}
#[test]
fn moved_source_prefers_copy_transfer() {
    matrix(1);
}
#[test]
fn move_only_source_uses_move_transfer() {
    matrix(2);
}
#[test]
fn copied_source_failure_phases_and_repair() {
    failures(0);
}
#[test]
fn moved_source_copy_failure_phases_and_repair() {
    failures(1);
}
#[test]
fn move_only_failure_phases_and_repair() {
    failures(2);
}
#[test]
fn invalid_sources_cannot_enter_destructive_replacement() {
    let mut array = fixture("");
    for mode in [0, 1, 2] {
        // Rejection occurs before temporary construction or any lifetime transfer.
        assert!(unsafe { dispatch(&mut array, 0, 4, mode) }.is_err());
    }
    assert_eq!(array.size(), 4);
    assert!(PROBE.with(|p| p.borrow().trace.is_empty()));
    repair_cleanup(&mut array, None, false);
}

#[test]
fn copied_source_prefers_nothrow_move_transfer() {
    matrix(3);
}
#[test]
fn copied_source_nothrow_move_constructor_failures() {
    failures(3);
}
