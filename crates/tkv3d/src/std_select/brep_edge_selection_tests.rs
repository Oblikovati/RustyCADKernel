//! Genuine edge construction replays native retained output, direct append, type and sampled geometry.
use super::StdSelectBrepSelectionTool;
use crate::{select_mgr::*, select3d::*};
use rustycad_tkbrep::{brep::BRepBuilder, topods::Edge};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Handle, RuntimeTyped},
};
use rustycad_tkmath::{
    gp::{Axis2, Circle3, Ellipse3, Point3},
    poly::Polygon3D,
    toploc::Location,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
#[test]
fn native_edge_sensitive_geometry_and_output_order_replay() {
    let rows = include_str!("../../tests/fixtures/colored-shape-compute/native-edge-selection.txt");
    for (index, row) in rows.lines().enumerate() {
        replay_edge_sensitive(index, row);
    }
}
fn replay_edge_sensitive(index: usize, row: &str) {
    let scene = brep_edge_entities(index);
    let output = &scene.output;
    let selection = &scene.selection;
    let fields: Vec<_> = row.split_whitespace().collect();
    assert_eq!(output.dynamic_type().unwrap().name(), fields[2]);
    assert_eq!(output.nb_sub_elements(), fields[3].parse::<i32>().unwrap());
    assert_eq!(
        selection.entities().len(),
        fields[4].parse::<usize>().unwrap()
    );
    assert_eq!(
        output.object_address() == scene.sentinel.object_address(),
        fields[5] == "1"
    );
    replay_edge_points(output, &fields[6..]);
}
pub(super) struct BrepEdgeFixtureEntities {
    pub sentinel: Handle<dyn Select3DSensitiveEntity>,
    pub output: Handle<dyn Select3DSensitiveEntity>,
    pub selection: Handle<SelectMgrSelection>,
    owner: Handle<dyn SelectMgrEntityOwnerSource>,
}
pub(super) fn brep_edge_entities(index: usize) -> BrepEdgeFixtureEntities {
    let owner = SelectMgrEntityOwner::new(7).cast();
    let selection = Handle::new(SelectMgrSelection::new(0));
    let point = Select3DSensitivePoint::new(&owner, Point3::new(9., 8., 7.));
    let sentinel = Handle::new(point).cast();
    let output = Handle::clone(&sentinel);
    let mut scene = BrepEdgeFixtureEntities {
        sentinel,
        output,
        selection,
        owner,
    };
    initialize_edge_fixture_selection(index, &mut scene);
    scene
}
fn initialize_edge_fixture_selection(index: usize, scene: &mut BrepEdgeFixtureEntities) {
    let shape = brep_edge_fixture(index);
    StdSelectBrepSelectionTool::get_edge_sensitive(
        &shape,
        &scene.owner,
        &scene.selection,
        0.1,
        0.5,
        9,
        500.,
        &mut scene.output,
    )
    .unwrap();
}
fn replay_edge_points(output: &Handle<dyn Select3DSensitiveEntity>, fields: &[&str]) {
    let points = if output.is_kind(Select3DSensitiveSegment::static_type()) {
        let segment = output.cast::<Select3DSensitiveSegment>();
        vec![segment.start_point(), segment.end_point()]
    } else if output.is_kind(Select3DSensitivePoly::static_type()) {
        output.cast::<Select3DSensitivePoly>().points_3d()
    } else if output.is_kind(Select3DSensitiveCurve::static_type()) {
        output.cast::<Select3DSensitiveCurve>().core.points_3d()
    } else {
        vec![output.cast::<Select3DSensitivePoint>().point()]
    };
    assert_eq!(points.len() * 3, fields.len());
    for (point, native) in points.iter().zip(fields.chunks_exact(3)) {
        for (value, expected) in [point.x(), point.y(), point.z()].into_iter().zip(native) {
            assert!(
                (value - expected.parse::<f64>().unwrap()).abs() < 1e-12,
                "value={value}, expected {expected}"
            );
        }
    }
}
pub(super) fn brep_edge_fixture(index: usize) -> Edge {
    match index {
        0 => BRepBuilderAPIMakeEdge::from_points(Point3::new(1., 2., 3.), Point3::new(5., 2., 3.))
            .unwrap()
            .edge()
            .unwrap(),
        1 => BRepBuilderAPIMakeEdge::from_circ_params(
            Circle3::new(Axis2::default(), 2.).unwrap(),
            0.2,
            3.4,
        )
        .unwrap()
        .edge()
        .unwrap(),
        2 => BRepBuilderAPIMakeEdge::from_elips_params(
            Ellipse3::new(Axis2::default(), 3., 1.).unwrap(),
            0.2,
            5.4,
        )
        .unwrap()
        .edge()
        .unwrap(),
        4 => BRepBuilderAPIMakeEdge::from_circ_params(
            Circle3::new(Axis2::default(), 1e-8).unwrap(),
            0.,
            3.,
        )
        .unwrap()
        .edge()
        .unwrap(),
        _ => brep_polygon_fixture(index),
    }
}
fn brep_polygon_fixture(index: usize) -> Edge {
    let mut edge = Edge::default();
    let builder = BRepBuilder::default();
    builder.make_edge(&mut edge).unwrap();
    if index == 3 {
        return edge;
    }
    let mut nodes = Array1::with_bounds(-2, if index == 5 { -1 } else { 0 }).unwrap();
    for position in nodes.lower()..=nodes.upper() {
        let offset = f64::from(position + 2);
        nodes
            .set_value(position, Point3::new(offset, offset * offset, 1.))
            .unwrap();
    }
    let polygon = Handle::new(Polygon3D::from_nodes(&nodes));
    builder
        .update_edge_polygon3d(&edge, &polygon, &Location::default())
        .unwrap();
    edge
}
