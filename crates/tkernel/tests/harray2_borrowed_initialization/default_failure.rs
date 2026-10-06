//! Native Array1 construction failure can leak its prefix; these checks assert Rust safety only.
use rustycad_tkernel::ncollection::HArray2;
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
};

#[derive(Default)]
struct DefaultJournal {
    calls: usize,
    failure: usize,
    dropped: Vec<usize>,
}
thread_local! { static DEFAULT_JOURNAL: RefCell<DefaultJournal> = RefCell::default(); }
struct FailingDefault(usize);
impl Default for FailingDefault {
    fn default() -> Self {
        let (call, failure) = DEFAULT_JOURNAL.with(|log| {
            let mut log = log.borrow_mut();
            log.calls += 1;
            (log.calls, log.failure)
        });
        assert_ne!(
            call, failure,
            "default call={call}; expected a non-failing constructor"
        );
        Self(call)
    }
}
impl Drop for FailingDefault {
    fn drop(&mut self) {
        DEFAULT_JOURNAL.with(|log| log.borrow_mut().dropped.push(self.0));
    }
}
fn reset_default_journal(failure: usize) {
    DEFAULT_JOURNAL.with(|log| {
        *log.borrow_mut() = DefaultJournal {
            failure,
            ..Default::default()
        }
    });
}
/// Checks cleanup without claiming native leaking behavior equivalence; `check_default_failure(2)`.
pub fn check_default_failure(failure: usize) {
    reset_default_journal(failure);
    let donor = FailingDefault(0);
    let failed = catch_unwind(AssertUnwindSafe(|| {
        HArray2::with_value_ref_with_assignment(-1, 0, 3, 4, &donor, |_, _| {
            panic!("assignment after default failure={failure}; expected no assignment")
        })
    }));
    assert!(failed.is_err());
    DEFAULT_JOURNAL.with(|log| {
        let log = log.borrow();
        assert_eq!(log.calls, failure);
        assert_eq!(log.dropped, (1..failure).collect::<Vec<_>>());
    });
    assert_eq!(donor.0, 0);
}
