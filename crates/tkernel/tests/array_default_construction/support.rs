use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::rc::Rc;

#[derive(Default)]
pub struct DefaultTrace {
    pub events: String,
    pub calls: usize,
    pub completed: usize,
    pub failure: Option<usize>,
}

struct DefaultMember {
    index: usize,
    trace: Rc<RefCell<DefaultTrace>>,
}

impl Drop for DefaultMember {
    fn drop(&mut self) {
        let mut trace = self.trace.borrow_mut();
        trace.events += &format!("G{},", self.index);
    }
}

pub struct DefaultCell {
    member: DefaultMember,
}

impl Drop for DefaultCell {
    fn drop(&mut self) {
        let mut trace = self.member.trace.borrow_mut();
        trace.events += &format!("D{},", self.member.index);
    }
}

fn construct_member(trace: &Rc<RefCell<DefaultTrace>>) -> DefaultMember {
    let mut events = trace.borrow_mut();
    let index = events.calls;
    events.calls += 1;
    events.events += &format!("M{index},");
    DefaultMember {
        index,
        trace: Rc::clone(trace),
    }
}

pub fn construct_cell(trace: &Rc<RefCell<DefaultTrace>>) -> DefaultCell {
    let member = construct_member(trace);
    trace.borrow_mut().events += &format!("C{},", member.index);
    let failed = trace.borrow().failure == Some(member.index);
    assert!(!failed, "default construction");
    trace.borrow_mut().completed += 1;
    DefaultCell { member }
}

pub fn native_row(label: &str, state: &str) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_array_default_construction.expected");
    let row = format!("{label}={state}");
    assert!(
        expected.lines().any(|line| line == row),
        "missing native row {row}"
    );
}

#[allow(unsafe_code)]
pub unsafe fn prefix_values(
    slots: &[MaybeUninit<DefaultCell>],
    from: usize,
    count: usize,
) -> String {
    slots[from..from + count]
        .iter()
        .map(|slot| unsafe { slot.assume_init_ref() }.member.index.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[allow(unsafe_code)]
pub unsafe fn cleanup_prefix(slots: &mut [MaybeUninit<DefaultCell>], from: usize, count: usize) {
    for slot in &mut slots[from..from + count] {
        unsafe { slot.assume_init_drop() };
    }
}
