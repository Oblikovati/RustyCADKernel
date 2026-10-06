use rustycad_tkernel::ncollection::{BaseAllocator, CellFilterCell};
use std::cell::RefCell;
#[derive(Default)]
pub struct Trace {
    pub events: RefCell<String>,
}
#[repr(C)]
pub struct Target<'a> {
    pub number: i32,
    trace: &'a Trace,
    copied: bool,
}
impl Clone for Target<'_> {
    fn clone(&self) -> Self {
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("C{},", self.number));
        Self {
            number: self.number,
            trace: self.trace,
            copied: true,
        }
    }
}
impl Drop for Target<'_> {
    fn drop(&mut self) {
        if self.copied {
            self.trace
                .events
                .borrow_mut()
                .push_str(&format!("D{},", self.number));
        }
    }
}
impl Trace {
    pub fn prepend<'a>(&'a self, cell: &mut CellFilterCell<Target<'a>>, number: i32) {
        cell.prepend_target(&Target {
            number,
            trace: self,
            copied: false,
        });
    }
}
pub fn coordinates(size: usize, offset: i32) -> Vec<i32> {
    (0..size).map(|index| offset + index as i32).collect()
}
pub fn assert_row(row: usize, actual: String) {
    let fixture =
        include_str!("../../../../tools/oracle/tkernel_cell_filter_cell_methods.expected");
    let expected = fixture.lines().nth(row).unwrap();
    // Native virtual Free counters have no equivalent injected override on BaseAllocator.
    // Retained bytes are compared separately; do not invent a Rust callback count.
    assert_eq!(
        actual,
        expected.trim_end_matches(" frees 0"),
        "native row {row}"
    );
}
pub fn live_nodes(arena: &BaseAllocator) -> usize {
    arena
        .reset(false)
        .unwrap_err()
        .message()
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap()
}
pub fn head(cell: &CellFilterCell<Target<'_>>) -> *const () {
    cell.targets()
        .next()
        .map_or(std::ptr::null(), |target| std::ptr::from_ref(target).cast())
}
mod movement;
pub use movement::{compare_assignment, compare_construction, compare_self};
mod safety;
