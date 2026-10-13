// SPDX-License-Identifier: GPL-2.0-only
//! Real iterator Value errors must not be mistaken for normal end-of-sequence.
use super::contexts::checked_occurrences;
use super::outcome::{BodyKind, OrderedCoverage, StageCoverage, ValidationStage};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Shape, ShapeIterator, TShape, TShapeBase, Vertex};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use std::sync::atomic::{AtomicU32, Ordering};

struct FakeChangingChildCount {
    transient: StandardTransient,
    topology: TShapeBase,
    reads: AtomicU32,
}
impl_transient!(FakeChangingChildCount, "Martensite_FakeChangingChildCount", dyn TShape, transient, interfaces: [dyn TShape]);

impl FakeChangingChildCount {
    fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            topology: TShapeBase::new(ShapeEnum::Face),
            reads: AtomicU32::new(0),
        }
    }
}

impl TShape for FakeChangingChildCount {
    fn tshape_base(&self) -> &TShapeBase {
        // Use the real iterator and its stored topology; no alternate traversal.
        &self.topology
    }

    fn empty_copy(&self) -> Result<Handle<dyn TShape>, Failure> {
        let copied = Handle::new(Self::new());
        Ok(copied.cast())
    }

    fn nb_children(&self) -> i32 {
        let previous = self.reads.fetch_add(1, Ordering::Relaxed);
        i32::from(previous == 0)
    }
}

fn changing_shape() -> Shape {
    let mut shape = Shape::default();
    let original = FakeChangingChildCount::new();
    shape.set_tshape(&original.empty_copy().unwrap());
    shape
}

#[test]
fn empty_and_exhausted_iterators_are_normal_ends() {
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex(&mut vertex);
    let shape = vertex.into_shape();
    assert_eq!(checked_occurrences(&shape).unwrap().len(), 1);
    let iterator = ShapeIterator::new(&shape, true, true);
    assert!(!iterator.more());
    assert_eq!(
        iterator.value().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        checked_occurrences(&super::tests::box_face())
            .unwrap()
            .len(),
        14
    );
}

#[test]
fn iterator_error_between_more_and_value_propagates() {
    let failure = checked_occurrences(&changing_shape()).unwrap_err();
    check_iterator_failure(&failure);
}

#[test]
fn traversal_failure_is_typed_incomplete_not_passed_coverage() {
    let face = super::tests::box_face();
    let analyzer = BRepCheckAnalyzer::new(&face, false, false, true).unwrap();
    let shape = changing_shape();
    let findings = super::inspection::inspect_candidate(
        &analyzer,
        &shape,
        BodyKind::PlanarRegion,
        &[],
        ValidationStage::Topology,
    );
    let mut report = OrderedCoverage::pending();
    report.record(ValidationStage::Topology, findings);
    assert_eq!(report.topology, StageCoverage::Incomplete);
    assert_eq!(report.geometry, StageCoverage::Blocked);
    assert!(report.findings[0].affected.shape.is_equal(&shape));
    check_incomplete_iterator_report(&report);
}

fn check_iterator_failure(failure: &Failure) {
    assert_eq!(failure.exception_type(), "Standard_NoSuchObject");
    assert!(failure.message().contains("TopoDS_Iterator::Value"));
    assert!(failure.message().contains("position 0"));
}

fn check_incomplete_iterator_report(report: &OrderedCoverage) {
    let super::outcome::ValidationCause::NativeFailure(failure) = &report.findings[0].cause else {
        panic!("expected original iterator failure, not a secondary analyzer lookup failure");
    };
    check_iterator_failure(failure);
    assert!(!report.complete());
}
