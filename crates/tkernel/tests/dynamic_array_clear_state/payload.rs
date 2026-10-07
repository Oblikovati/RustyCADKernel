use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray, DynamicArrayClearState};
use std::{cell::RefCell, panic::panic_any, rc::Rc};
pub(crate) type SharedProbe = Rc<RefCell<ClearProbe>>;
pub(crate) struct ClearProbe {
    pub(crate) allocator: BaseAllocator,
    pub(crate) fail: i32,
    pub(crate) events: String,
    pub(crate) slots: Vec<*const ClearItem>,
    pub(crate) ended: Vec<i32>,
    used: *const usize,
}
pub(crate) struct ClearItem {
    pub(crate) value: i32,
    probe: SharedProbe,
}
#[derive(Debug, PartialEq)]
pub(crate) struct ClearPanic(pub(crate) i32);
impl Drop for ClearItem {
    fn drop(&mut self) {
        let mut probe = self.probe.borrow_mut();
        if probe.used.is_null() {
            return;
        }
        let size = unsafe { *probe.used };
        let blocks = live_blocks(&probe.allocator);
        let same = std::ptr::eq(self, probe.slots[(self.value - 1) as usize]);
        probe.events.push_str(&format!(
            "D{}@{size}#{blocks}:{};",
            self.value,
            u8::from(same)
        ));
        probe.ended.push(self.value);
        if self.value == probe.fail {
            panic_any(ClearPanic(self.value));
        }
    }
}
struct ClearContext(SharedProbe);
impl Drop for ClearContext {
    fn drop(&mut self) {
        self.0.borrow_mut().used = std::ptr::null();
    }
}
pub(crate) fn live_blocks(allocator: &BaseAllocator) -> usize {
    allocator
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count as usize)
        .sum()
}
pub(crate) fn item(value: i32, probe: &SharedProbe) -> ClearItem {
    ClearItem {
        value,
        probe: probe.clone(),
    }
}
pub(crate) fn fixture() -> (DynamicArrayClearState<ClearItem>, SharedProbe) {
    let allocator = BaseAllocator::accumulating(4096).unwrap();
    let probe = shared_probe(&allocator);
    let mut array = DynamicArray::with_allocator(2, Some(allocator));
    for value in 1..=5 {
        probe
            .borrow_mut()
            .slots
            .push(array.append(item(value, &probe)));
    }
    let state = array
        .into_clear_state()
        .unwrap_or_else(|(_, failure)| panic!("{failure}"));
    (state, probe)
}
pub(crate) fn clear(
    state: &mut DynamicArrayClearState<ClearItem>,
    probe: &SharedProbe,
    release: bool,
) {
    unsafe {
        state.clear_with_destruction(release, |slot, used| {
            probe.borrow_mut().used = used;
            let _context = ClearContext(probe.clone());
            slot.drop_in_place();
        });
    }
}
pub(crate) fn repair(state: &mut DynamicArrayClearState<ClearItem>, probe: &SharedProbe) {
    state.repair_destroyed_with(|index| item(index as i32 + 1, probe));
    let mut observer = probe.borrow_mut();
    observer.events.clear();
    observer.ended.clear();
    observer.fail = 0;
}

fn shared_probe(allocator: &BaseAllocator) -> SharedProbe {
    Rc::new(RefCell::new(ClearProbe {
        allocator: allocator.clone(),
        fail: 0,
        events: String::new(),
        slots: Vec::new(),
        ended: Vec::new(),
        used: std::ptr::null(),
    }))
}
