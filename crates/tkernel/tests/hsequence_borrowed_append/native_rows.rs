use super::fixture::{BorrowedCase, BorrowedItem, live_nodes};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn append_state(
    case: &BorrowedCase,
    prefix: i32,
    fail: bool,
    caught: bool,
    donor: &BorrowedItem,
) -> String {
    format!(
        "append{prefix}_{} {} {} {} {} {} {}\n",
        u8::from(fail),
        case.values(),
        u8::from(caught),
        donor.number,
        donor.reads(),
        live_nodes(&case.arena),
        case.journal.events()
    )
}
fn append_case(prefix: i32, fail: bool) -> String {
    let case = BorrowedCase::new(prefix);
    let donor = BorrowedItem::new(9, fail, &case.journal);
    let caught = catch_unwind(AssertUnwindSafe(|| case.append(&donor))).is_err();
    let mut rows = append_state(&case, prefix, fail, caught, &donor);
    case.journal.clear();
    case.owner.change_sequence(|s| s.clear(None));
    rows += &format!(
        "clear{prefix}_{} {} {} {}\n",
        u8::from(fail),
        case.observer.snapshot().size,
        live_nodes(&case.arena),
        case.journal.events()
    );
    rows
}
fn recovery_case() -> String {
    let case = BorrowedCase::new(0);
    let bad = BorrowedItem::new(9, true, &case.journal);
    let good = BorrowedItem::new(10, false, &case.journal);
    assert!(catch_unwind(AssertUnwindSafe(|| case.append(&bad))).is_err());
    case.append(&good);
    format!(
        "recovery {} {} {} {} {}\n",
        case.values(),
        bad.reads(),
        good.reads(),
        live_nodes(&case.arena),
        case.journal.events()
    )
}
/// Derives allocator/publication/failure observations from real state; `all_rows()`.
pub fn all_rows() -> String {
    let mut rows = String::new();
    for prefix in [0, 2] {
        for fail in [false, true] {
            rows += &append_case(prefix, fail);
        }
    }
    rows + &recovery_case()
}
