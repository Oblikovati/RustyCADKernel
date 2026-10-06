use super::fixture::{AppendItem, AppendJournal, DerivedSequence, base_handle, values, wrapper};
use rustycad_tkernel::{
    ncollection::{BaseAllocator, HSequence},
    standard::{Handle, Transient},
};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn invoke(receiver: &HSequence<AppendItem>, source: &Handle<DerivedSequence>) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        receiver
            .append_handle_projected(source, DerivedSequence::selected)
            .unwrap();
    }))
    .is_err()
}
fn transfer_state(
    name: &str,
    target: &HSequence<AppendItem>,
    source: &DerivedSequence,
    state: (bool, bool),
    journal: &AppendJournal,
) -> String {
    format!(
        "{name} {} {} {} {} {} {}\n",
        values(target),
        values(&source.base),
        u8::from(state.0),
        u8::from(state.1),
        source.transient_base().ref_count(),
        journal.events()
    )
}
fn transfer_case(name: &str, same: bool, failure: i32) -> String {
    let journal = AppendJournal::default();
    let arena = BaseAllocator::incremental(4096);
    let target_arena = if same {
        arena.clone()
    } else {
        BaseAllocator::incremental(4096)
    };
    let source = simple_source(&arena, &journal, false, false, false);
    let owner = source.get().unwrap();
    let target = wrapper(&target_arena, &[0], &journal);
    let before = owner.base.sequence(|s| s.first().unwrap() as *const _);
    journal.reset(failure);
    let caught = invoke(&target, &source);
    let stable = target.sequence(|s| s.value(2).is_ok_and(|x| std::ptr::eq(x, before)));
    let row = transfer_state(name, &target, owner, (caught, stable), &journal);
    journal.reset(0);
    row
}
fn simple_source(
    arena: &BaseAllocator,
    journal: &AppendJournal,
    empty: bool,
    redirect: bool,
    fail: bool,
) -> Handle<DerivedSequence> {
    let mut source = DerivedSequence::new(arena, journal);
    if !empty {
        source.base = wrapper(arena, &[1, 2], journal);
    }
    if redirect {
        source.alternate = wrapper(arena, &[7, 8], journal);
    }
    source.redirect = redirect;
    source.fail_getter = fail;
    Handle::new(source)
}
fn simple_case(name: &str, itself: bool, empty: bool, redirect: bool, fail: bool) -> String {
    let journal = AppendJournal::default();
    let arena = BaseAllocator::incremental(4096);
    let source = simple_source(&arena, &journal, empty, redirect, fail);
    let owner = source.get().unwrap();
    let target = wrapper(&arena, &[0], &journal);
    let receiver = if itself { &owner.base } else { &target };
    journal.reset(0);
    let caught = invoke(receiver, &source);
    format!(
        "{name} {} {} {} {} {} {}\n",
        values(receiver),
        values(&owner.base),
        values(&owner.alternate),
        u8::from(caught),
        owner.transient_base().ref_count(),
        journal.events()
    )
}
fn base_case() -> String {
    let journal = AppendJournal::default();
    let source = base_handle(&journal);
    let target = HSequence::default();
    journal.reset(0);
    target.append_handle(&source).unwrap();
    assert!(journal.events().is_empty());
    format!(
        "base {} {} {} -\n",
        values(&target),
        values(source.get().unwrap()),
        source.get().unwrap().transient_base().ref_count()
    )
}
/// Derives every native line from current Rust state; `all_rows()`.
pub fn all_rows() -> String {
    let mut rows = transfer_case("same", true, 0) + &transfer_case("different", false, 0);
    rows += &transfer_case("copy_fail1", false, 1);
    rows += &transfer_case("copy_fail2", false, 2);
    rows += &simple_case("empty", false, true, false, false);
    rows += &simple_case("self", true, false, false, false);
    rows += &simple_case("redirect", false, false, true, false);
    rows += &simple_case("getter_fail", false, false, false, true);
    rows + &base_case()
}
