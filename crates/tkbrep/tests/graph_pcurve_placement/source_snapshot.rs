// SPDX-License-Identifier: GPL-2.0-only
//! Capture owned values of authored source representations, never Handle identity as isolation proof.
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{self, Shape, ShapeIterator},
};
use rustycad_tkmath::gp::{Point2, Point3};
#[derive(Debug, PartialEq)]
pub(super) struct SourceValues {
    pcurve_points: [Point2; 2],
    space_points: [Point3; 2],
    pcurve_range: [f64; 2],
    space_range: [f64; 2],
    edge_tolerance: f64,
    face_tolerance: f64,
}
pub(super) fn capture_source_values(source: &Shape) -> SourceValues {
    let face = topods::face(source).unwrap();
    let wire = first_child(source);
    let child = first_child(&wire);
    let edge = topods::edge(&child).unwrap();
    let pcurve = BRepTool::curve_on_face(&edge, &face).unwrap();
    let curve = BRepTool::curve_with_location(&edge).unwrap();
    assert!(pcurve.is_stored);
    SourceValues {
        pcurve_points: [0., 1.].map(|t| pcurve.pcurve.value(t).unwrap()),
        space_points: space_samples(&curve),
        pcurve_range: [pcurve.first, pcurve.last],
        space_range: [curve.first, curve.last],
        edge_tolerance: BRepTool::edge_tolerance(&edge).unwrap(),
        face_tolerance: BRepTool::face_tolerance(&face).unwrap(),
    }
}

fn first_child(source: &Shape) -> Shape {
    ShapeIterator::new(source, true, true)
        .value()
        .unwrap()
        .clone()
}
fn space_samples(curve: &rustycad_tkbrep::brep::EdgeCurve3D) -> [Point3; 2] {
    [0., 1.].map(|t| {
        curve
            .curve
            .value(t)
            .unwrap()
            .transformed(curve.location.transformation())
    })
}
