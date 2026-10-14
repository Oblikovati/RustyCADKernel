// SPDX-License-Identifier: GPL-2.0-only
use crate::sketch_region::{ArrangementSource, RegionSourceId};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

pub(super) fn two_root_edges() -> Vec<Shape> {
    curved_line_edges([
        Point3::new(0., 0., 0.),
        Point3::new(0.5, 1., 0.),
        Point3::new(1., 0., 0.),
    ])
}
pub(super) fn single_root_edges() -> Vec<Shape> {
    curved_line_edges([
        Point3::new(0., 0., 0.),
        Point3::new(0.5, 0., 0.),
        Point3::new(1., 1., 0.),
    ])
}
fn curved_line_edges(points: [Point3; 3]) -> Vec<Shape> {
    let curve = quadratic_edge(points);
    let line =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(-1., 0.25, 0.), Point3::new(2., 0.25, 0.))
            .unwrap()
            .edge()
            .unwrap()
            .into_shape();
    vec![curve, line]
}
pub(super) fn quadratic_edge(points: [Point3; 3]) -> Shape {
    let poles = Array1::from_shared(&points, 1);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}
pub(super) fn binding_sources(shapes: &[Shape]) -> Vec<ArrangementSource<'_>> {
    shapes
        .iter()
        .enumerate()
        .map(|(index, shape)| ArrangementSource {
            identity: RegionSourceId(index as u64 + 1),
            shape,
        })
        .collect()
}

pub(super) fn actual_binding_snapshot(
    edges: &[Shape],
) -> crate::sketch_region::OwnedPartitionSnapshot {
    use crate::sketch_region::{PartitionObservation, observe_owned_partition};
    let observed = observe_owned_partition(&binding_sources(edges), &mut |_| false);
    let PartitionObservation::Captured(snapshot) = observed.partition else {
        panic!(
            "source count={}; expected owned captured DS metadata",
            edges.len()
        );
    };
    snapshot
}
