use rustycad_tkernel::ncollection::Array1;
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::rc::Rc;

#[derive(Default)]
pub struct DestructionTrace {
    pub events: String,
    pub live: [bool; 4],
    pub failure: Option<usize>,
}

struct DestructionMember {
    index: usize,
    trace: Rc<RefCell<DestructionTrace>>,
}

impl Drop for DestructionMember {
    fn drop(&mut self) {
        let mut trace = self.trace.borrow_mut();
        trace.events += &format!("G{},", self.index);
    }
}

pub struct DestructionCell {
    index: usize,
    member: DestructionMember,
}

impl Drop for DestructionCell {
    fn drop(&mut self) {
        let failed = self.member.trace.borrow_mut().destroy(self.index);
        assert!(!failed, "destruction");
    }
}

impl DestructionTrace {
    fn destroy(&mut self, index: usize) -> bool {
        self.events += &format!("D{index},");
        self.live[index] = false;
        self.failure == Some(index)
    }
}

pub fn live_slots(trace: &Rc<RefCell<DestructionTrace>>) -> [MaybeUninit<DestructionCell>; 4] {
    std::array::from_fn(|index| {
        MaybeUninit::new(DestructionCell {
            index,
            member: DestructionMember {
                index,
                trace: Rc::clone(trace),
            },
        })
    })
}

pub fn native_row(label: &str, state: &str) {
    let expected = include_str!("../../../../tools/oracle/tkernel_array_raw_destruction.expected");
    let row = format!("{label}={state}");
    assert!(
        expected.lines().any(|line| line == row),
        "missing native row {row}"
    );
}

#[allow(unsafe_code)]
pub unsafe fn invoke_destruction(slots: *mut DestructionCell, from: usize, to: usize) -> bool {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        Array1::destroy_nontrivial_raw(slots, from, to);
    }));
    if let Err(payload) = result {
        assert_eq!(payload.downcast_ref::<&str>(), Some(&"destruction"));
        return true;
    }
    false
}

#[allow(unsafe_code)]
pub unsafe fn survivor_values(slots: &[MaybeUninit<DestructionCell>], live: &[bool; 4]) -> String {
    slots
        .iter()
        .enumerate()
        .filter(|(index, _)| live[*index])
        .map(|(_, slot)| unsafe { slot.assume_init_ref() }.index.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[allow(unsafe_code)]
pub unsafe fn cleanup_survivors(
    slots: &mut [MaybeUninit<DestructionCell>],
    trace: &RefCell<DestructionTrace>,
) {
    trace.borrow_mut().failure = None;
    trace.borrow_mut().events.clear();
    let live = trace.borrow().live;
    for (index, slot) in slots.iter_mut().enumerate() {
        if live[index] {
            unsafe { slot.assume_init_drop() };
        }
    }
}
