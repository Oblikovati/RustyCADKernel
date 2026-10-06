use rustycad_tkernel::ncollection::{BaseAllocator, BoundingVolume, UBTree};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Default)]
pub struct ParentTrace(Rc<RefCell<ParentEvents>>);
#[derive(Default)]
struct ParentEvents {
    events: String,
    bounds_address: usize,
    object_address: usize,
    active: bool,
    rejected: char,
    stable: bool,
}
thread_local! {
    static DEFAULT_PARENT_TRACE: RefCell<ParentTrace> = RefCell::new(ParentTrace::default());
}
impl ParentTrace {
    pub fn install_defaults(&self) {
        DEFAULT_PARENT_TRACE.with(|slot| *slot.borrow_mut() = self.clone());
    }
    pub fn item<const KIND: char>(&self, value: i32) -> ParentPayload<KIND> {
        ParentPayload {
            value,
            trace: self.clone(),
        }
    }
    pub fn start(&self, tree: &Tree, rejected: char) {
        let root = tree.root().unwrap();
        *self.0.borrow_mut() = ParentEvents {
            bounds_address: std::ptr::from_ref(root.bounds()).addr(),
            object_address: std::ptr::from_ref(root.object()).addr(),
            rejected,
            stable: true,
            ..ParentEvents::default()
        };
    }
    pub fn finish(&self) -> (String, bool) {
        let mut events = self.0.borrow_mut();
        events.active = false;
        (events.events.clone(), events.stable)
    }
    fn event(&self, phase: char, kind: char, value: i32, address: usize) {
        let mut events = self.0.borrow_mut();
        if phase == 'A' && address == events.bounds_address {
            events.active = true;
        }
        if !events.active || (kind == 'B' && address != events.bounds_address) {
            return;
        }
        events.events.push_str(&format!("{phase}{kind}{value},"));
        if phase == 'A' && kind == 'O' {
            events.stable &= address == events.object_address;
        }
        reject_parent_phase(&mut events, phase, kind);
    }
}
fn reject_parent_phase(events: &mut ParentEvents, phase: char, kind: char) {
    let rejected = match events.rejected {
        'B' => phase == 'A' && kind == 'B',
        'N' => phase == 'N',
        'O' => phase == 'A' && kind == 'O',
        _ => false,
    };
    if rejected {
        events.rejected = '\0';
        panic!("parent {kind} phase {phase} rejected; expected permitted callback");
    }
}
pub struct ParentPayload<const KIND: char> {
    pub value: i32,
    trace: ParentTrace,
}
impl<const KIND: char> Default for ParentPayload<KIND> {
    fn default() -> Self {
        let trace = DEFAULT_PARENT_TRACE.with(|slot| slot.borrow().clone());
        // A failing constructor never completes an object whose destructor could run.
        trace.event('N', KIND, 0, 0);
        trace.item(0)
    }
}
impl<const KIND: char> Clone for ParentPayload<KIND> {
    fn clone(&self) -> Self {
        self.trace.event('C', KIND, self.value, 0);
        self.trace.item(self.value)
    }
    fn clone_from(&mut self, source: &Self) {
        self.value = source.value;
        self.trace
            .event('A', KIND, self.value, std::ptr::from_ref(self).addr());
    }
}
impl<const KIND: char> Drop for ParentPayload<KIND> {
    fn drop(&mut self) {
        self.trace
            .event('D', KIND, self.value, std::ptr::from_ref(self).addr());
    }
}
impl BoundingVolume for ParentPayload<'B'> {
    fn add(&mut self, _: &Self) {}
    fn is_out(&self, _: &Self) -> bool {
        false
    }
    fn square_extent(&self) -> f64 {
        0.0
    }
}
pub type Tree = UBTree<ParentPayload<'O'>, ParentPayload<'B'>>;
pub fn populated(trace: &ParentTrace, branch: bool) -> Tree {
    trace.install_defaults();
    let mut tree = Tree::with_allocator(BaseAllocator::incremental(1024));
    tree.add(trace.item(1), trace.item(10));
    if branch {
        tree.gemmate(
            tree.root_id().unwrap(),
            trace.item(30),
            trace.item(2),
            trace.item(20),
        );
    }
    tree
}
