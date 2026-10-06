use super::*;
struct DestructionItem {
    value: i32,
    allocator: BaseAllocator,
    trace: Rc<RefCell<Vec<String>>>,
}
impl Drop for DestructionItem {
    fn drop(&mut self) {
        self.trace
            .borrow_mut()
            .push(format!("d{}@{}", self.value, live_blocks(&self.allocator)));
    }
}
fn phase_array(
    allocator: &BaseAllocator,
    trace: &Rc<RefCell<Vec<String>>>,
) -> DynamicArray<DestructionItem> {
    let mut target = DynamicArray::with_allocator(2, Some(allocator.clone()));
    for value in 1..=5 {
        target.emplace_append(|| DestructionItem {
            value,
            allocator: allocator.clone(),
            trace: Rc::clone(trace),
        });
    }
    target
}
pub(super) fn clear_case(name: &str, release: bool, scoped_drop: bool) -> String {
    let allocator = table_allocator();
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut target = phase_array(&allocator, &trace);
    if scoped_drop {
        drop(target);
        return phase_row(name, &allocator, &trace);
    }
    target.clear(release);
    phase_row(name, &allocator, &trace)
}
fn phase_row(name: &str, allocator: &BaseAllocator, trace: &RefCell<Vec<String>>) -> String {
    format!(
        "{name} {} {}\n",
        live_blocks(allocator),
        trace.borrow().join(",")
    )
}
