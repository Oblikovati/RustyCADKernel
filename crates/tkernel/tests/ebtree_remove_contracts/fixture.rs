use rustycad_tkernel::ncollection::{BaseAllocator, BoundingVolume, EBTree};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Default)]
pub struct RemoveTrace(Rc<RefCell<RemoveEvents>>);
#[derive(Default)]
struct RemoveEvents {
    events: String,
    assignments: Vec<usize>,
    additions: Vec<usize>,
    rejected: (char, usize),
}
impl RemoveTrace {
    pub fn reset(&self, phase: char, count: usize) {
        *self.0.borrow_mut() = RemoveEvents {
            rejected: (phase, count),
            ..RemoveEvents::default()
        };
    }
    pub fn events(&self) -> String {
        self.0.borrow().events.clone()
    }
    pub fn addresses(&self) -> (Vec<usize>, Vec<usize>) {
        let events = self.0.borrow();
        (events.assignments.clone(), events.additions.clone())
    }
    fn event(&self, phase: char, value: i32, address: usize) {
        let mut events = self.0.borrow_mut();
        events.events.push_str(&format!("{phase}{value},"));
        match phase {
            'A' => events.assignments.push(address),
            'G' => events.additions.push(address),
            _ => {}
        }
        if events.rejected.0 == phase {
            events.rejected.1 -= 1;
            if events.rejected.1 == 0 {
                events.rejected.0 = '\0';
                panic!("bounds {phase} callback rejected; expected permitted phase");
            }
        }
    }
    pub fn bounds(&self, value: i32) -> RemoveBounds {
        RemoveBounds {
            value,
            trace: self.clone(),
        }
    }
}
pub struct RemoveBounds {
    pub value: i32,
    trace: RemoveTrace,
}
impl Clone for RemoveBounds {
    fn clone(&self) -> Self {
        self.trace.event('C', self.value, 0);
        self.trace.bounds(self.value)
    }
    fn clone_from(&mut self, source: &Self) {
        self.value = source.value;
        self.trace
            .event('A', self.value, std::ptr::from_ref(self).addr());
    }
}
impl Drop for RemoveBounds {
    fn drop(&mut self) {
        self.trace
            .event('D', self.value, std::ptr::from_ref(self).addr());
    }
}
impl BoundingVolume for RemoveBounds {
    fn add(&mut self, other: &Self) {
        self.value |= other.value;
        self.trace
            .event('G', other.value, std::ptr::from_ref(self).addr());
    }
    fn is_out(&self, _: &Self) -> bool {
        false
    }
    fn square_extent(&self) -> f64 {
        0.0
    }
}
pub type Editable = EBTree<i32, RemoveBounds>;
pub fn populated(trace: &RemoveTrace, count: i32) -> Editable {
    populated_with_allocator(trace, count, BaseAllocator::common())
}
pub fn populated_with_allocator(
    trace: &RemoveTrace,
    count: i32,
    allocator: BaseAllocator,
) -> Editable {
    let mut tree = Editable::with_allocator(allocator);
    for value in 1..=count {
        tree.add(value, trace.bounds(1 << value));
    }
    trace.reset('\0', 0);
    tree
}
pub fn caught(operation: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err()
}
pub fn expected_row(prefix: &str) -> &'static str {
    include_str!("../../../../tools/oracle/tkernel_ebtree_remove_contracts.expected")
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap()
}
pub fn ancestor_addresses(tree: &Editable) -> Vec<usize> {
    let base = tree.as_ubtree();
    let mut node = base.parent(tree.find_node(&1).unwrap()).unwrap();
    let mut addresses = Vec::new();
    loop {
        addresses.push(std::ptr::from_ref(node.bounds()).addr());
        let Some(parent) = base.parent(node) else {
            break;
        };
        node = parent;
    }
    addresses
}
