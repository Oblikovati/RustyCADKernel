//! Probes read real live metadata fields; no owner reference is fabricated during callbacks.
use super::{CellMapNode, CellObjectNode, CellRecord};
use crate::ncollection::{BaseAllocator, CellAction, CellFilter, CellInspector};
use std::{cell::RefCell, collections::BTreeMap, fmt::Write, rc::Rc};
mod fixture;
mod layout;
mod unwind;
mod variance;
use fixture::{Target, filled, native_row, operation, remaining};
#[test]
fn native_head_middle_tail_remove_phases() {
    for mode in 0..3 {
        check_scenario(mode);
    }
}
#[test]
fn native_purge_all_and_partial_phases() {
    for mode in 3..5 {
        check_scenario(mode);
    }
}
#[test]
fn native_same_and_replacement_arena_clear_phases() {
    for mode in 5..7 {
        check_scenario(mode);
    }
}
fn check_scenario(mode: usize) {
    let (mut filter, probe) = filled(mode);
    operation(mode, &mut filter, &probe);
    let mut row = format!("phase{mode} {}", probe.borrow().events);
    probe.borrow_mut().cell = None;
    remaining(&mut row, &filter);
    native_row(&row);
}
#[derive(Default)]
struct Inspector {
    all: bool,
}
impl CellInspector for Inspector {
    type Target = Target;
    type Point = [f64; 1];
    const DIMENSION: usize = 1;
    fn coord(axis: usize, point: &Self::Point) -> f64 {
        point[axis]
    }
    fn is_equal(first: &Target, second: &Target) -> bool {
        first.number == second.number
    }
    fn inspect(&mut self, target: &Target) -> CellAction {
        write!(target.probe.borrow_mut().events, "I{};", target.number).unwrap();
        if self.all || target.number % 2 == 0 {
            CellAction::Purge
        } else {
            CellAction::Keep
        }
    }
}
