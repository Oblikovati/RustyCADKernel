//! Native linear transfer over matching geometry with independent parameter ranges.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::Face,
};
use rustycad_tkernel::{ncollection::HSequence, standard::Handle};
use rustycad_tkshhealing::{
    shape_analysis::{ShapeAnalysisTransferParameters, TransferParameters},
    shape_build::ShapeBuildEdge,
};
#[path = "divide_support/geometry.rs"]
mod geometry;
use geometry::edge_with_parameterizations;
fn transfer_values(values: &[f64]) -> Handle<HSequence<f64>> {
    let sequence = Handle::new(HSequence::<f64>::default());
    for &value in values {
        sequence.change_sequence(|s| {
            s.append(value);
        });
    }
    sequence
}
#[test]
fn default_transfer_and_sequence_ownership() {
    let tool = ShapeAnalysisTransferParameters::new();
    assert!(tool.is_same_range());
    let values = transfer_values(&[2., 3.5, 5.]);
    let result = tool.perform(&values, true).unwrap();
    assert!(values != result);
    assert_eq!(
        result.sequence(|s| s.iter().copied().collect::<Vec<_>>()),
        vec![2., 3.5, 5.]
    );
    assert!(tool.perform(&Handle::default(), true).is_err());
    assert!(
        tool.perform(&transfer_values(&[]), false)
            .unwrap()
            .sequence(|s| s.is_empty())
    );
}
#[test]
fn independent_ranges_transfer_both_directions_and_reinitialize() {
    let (edge, face) = edge_with_parameterizations();
    let tool = ShapeAnalysisTransferParameters::from_edge(&edge, &face).unwrap();
    assert!(!tool.is_same_range());
    for (spatial, planar) in [(2., 10.), (3.5, 13.), (5., 16.), (-1., 4.)] {
        assert_eq!(tool.perform_parameter(spatial, true).unwrap(), planar);
        assert_eq!(tool.perform_parameter(planar, false).unwrap(), spatial);
    }
    tool.set_max_tolerance(0.125);
    tool.init(&edge, &Face::default()).unwrap();
    assert!(tool.is_same_range());
    assert_eq!(
        tool.transfer_parameter_data().with(|f| f.max_tolerance),
        0.125
    );
}
#[test]
fn pcurve_range_transfer_orders_bounds_and_preserves_source() {
    let (edge, face) = edge_with_parameterizations();
    let tool = ShapeAnalysisTransferParameters::from_edge(&edge, &face).unwrap();
    let target = ShapeBuildEdge.copy(&edge, true).unwrap();
    tool.transfer_range(&target, 15., 11., true).unwrap();
    let (_, first, last) = BRepTool::curve(&target).unwrap();
    assert_eq!((first, last), (2.5, 4.5));
    assert_eq!(BRepTool::range_on_face(&target, &face).unwrap(), (11., 15.));
    assert_eq!(BRepTool::range(&edge).unwrap(), (2., 5.));
}
#[test]
fn spatial_range_transfer_retains_reversed_bounds() {
    let (edge, face) = edge_with_parameterizations();
    let tool = ShapeAnalysisTransferParameters::from_edge(&edge, &face).unwrap();
    let target = ShapeBuildEdge.copy(&edge, true).unwrap();
    tool.transfer_range(&target, 4.5, 2.5, false).unwrap();
    assert_eq!(BRepTool::range(&target).unwrap(), (4.5, 2.5));
    assert_eq!(BRepTool::range_on_face(&target, &face).unwrap(), (15., 11.));
    assert_eq!(
        BRepTool::same_range(&target).unwrap(),
        BRepTool::same_range(&edge).unwrap()
    );
}
#[test]
fn collapsed_spatial_span_uses_native_unit_scale() {
    let (edge, face) = edge_with_parameterizations();
    BRepBuilder::default().range(&edge, 2., 2., true).unwrap();
    let tool = ShapeAnalysisTransferParameters::from_edge(&edge, &face).unwrap();
    assert_eq!(tool.perform_parameter(3., true).unwrap(), 11.);
    assert_eq!(tool.perform_parameter(11., false).unwrap(), 3.);
}
#[path = "divide_support/transfer_criterion.rs"]
mod transfer_criterion;
#[test]
fn sequence_uses_derived_scalar_transfer_and_shared_base_state() {
    let (edge, face) = edge_with_parameterizations();
    let derived = Handle::new(transfer_criterion::OffsetTransfer::default());
    let virtual_tool = derived.cast::<dyn TransferParameters>();
    virtual_tool.init(&edge, &face).unwrap();
    let values = transfer_values(&[2., 3.5, 5.]);
    let result = virtual_tool.perform(&values, true).unwrap();
    assert_eq!(
        result.sequence(|s| s.iter().copied().collect::<Vec<_>>()),
        vec![11., 14., 17.]
    );
    assert!(
        !derived
            .cast::<ShapeAnalysisTransferParameters>()
            .is_same_range()
    );
    assert_eq!(virtual_tool.perform_parameter(13., false).unwrap(), 2.5);
}
