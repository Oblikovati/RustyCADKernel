use rustycad_tkernel::ncollection::HashPolicy;
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind},
};
#[derive(Default)]
pub struct LookupTrace {
    pub events: RefCell<String>,
    pub code: Cell<usize>,
    pub reject_hash: Cell<bool>,
    pub reject_equal: Cell<usize>,
    pub equalities: Cell<usize>,
}
pub struct LookupPolicy<'a>(pub &'a LookupTrace);
impl HashPolicy<i32> for LookupPolicy<'_> {
    fn hash(&self, key: &i32) -> usize {
        self.0.events.borrow_mut().push_str(&format!("h{key},"));
        assert!(
            !self.0.reject_hash.get(),
            "hash rejected key {key}; expected an accepted key"
        );
        self.0.code.get()
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        self.0
            .events
            .borrow_mut()
            .push_str(&format!("e{left}:{right},"));
        self.0.equalities.set(self.0.equalities.get() + 1);
        assert_ne!(
            self.0.equalities.get(),
            self.0.reject_equal.get(),
            "equality rejected {left}:{right}; expected accepted keys"
        );
        left == right
    }
}
pub fn outcome(callback: impl FnOnce() -> bool) -> String {
    match catch_unwind(AssertUnwindSafe(callback)) {
        Ok(found) => usize::from(found).to_string(),
        Err(_) => "throw".to_owned(),
    }
}
pub fn native_row(label: &str, actual: String) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_ordered_protected_lookup.expected")
            .lines()
            .find(|line| line.starts_with(&format!("{label} ")))
            .unwrap();
    assert_eq!(actual, expected);
}
pub struct LookupCase {
    pub label: &'static str,
    pub filled: bool,
    pub hash: bool,
    pub key: i32,
    pub reject_hash: bool,
    pub reject_equal: usize,
}
impl LookupTrace {
    pub fn prepare(&self, case: &LookupCase) {
        self.events.borrow_mut().clear();
        self.equalities.set(0);
        self.reject_hash.set(case.reject_hash);
        self.reject_equal.set(case.reject_equal);
    }
    pub fn verify_lookup(&self, label: &str, result: &str, node: &str, hash: usize) {
        let events = self.events.borrow();
        let events = if events.is_empty() { "-" } else { &events };
        native_row(label, format!("{label} {result} {node} {hash} {events}"));
    }
}
pub fn cases() -> Vec<LookupCase> {
    let mut cases = vec![
        case("empty", false, false, 8, false, 0),
        case("empty_hash", false, true, 8, false, 0),
        case("empty_hash_throw", false, true, 8, true, 0),
    ];
    for (label, key) in [("first", 7), ("middle", 1), ("last", 4), ("miss", 8)] {
        cases.push(case(label, true, false, key, false, 0));
    }
    for (label, key) in [
        ("first_hash", 7),
        ("middle_hash", 1),
        ("last_hash", 4),
        ("miss_hash", 8),
    ] {
        cases.push(case(label, true, true, key, false, 0));
    }
    cases.extend(throw_cases());
    cases
}
fn case(
    label: &'static str,
    filled: bool,
    hash: bool,
    key: i32,
    reject_hash: bool,
    reject_equal: usize,
) -> LookupCase {
    LookupCase {
        label,
        filled,
        hash,
        key,
        reject_hash,
        reject_equal,
    }
}
fn throw_cases() -> [LookupCase; 8] {
    [
        case("hash_throw", true, false, 8, true, 0),
        case("hash_output_throw", true, true, 8, true, 0),
        case("equal1", true, false, 8, false, 1),
        case("equal1_hash", true, true, 8, false, 1),
        case("equal2", true, false, 8, false, 2),
        case("equal2_hash", true, true, 8, false, 2),
        case("equal3", true, false, 8, false, 3),
        case("equal3_hash", true, true, 8, false, 3),
    ]
}
