// SPDX-License-Identifier: GPL-2.0-only
//! Authored metadata-only shapes pin the compatibility floor; they are not accepted bodies.
use super::*;
use rustycad_tkbrep::{
    brep::{BRepTEdge, BRepTFace, BRepTVertex, BRepTool},
    topods::{Edge, Face, Shape, TShape, Vertex},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

fn authored_tolerances(raw: f64) -> [(f64, f64); 3] {
    let vertex = BRepTVertex::new();
    vertex.set_tolerance(raw);
    let edge = BRepTEdge::new();
    edge.set_tolerance(raw);
    let face = BRepTFace::new();
    face.set_tolerance(raw);
    let stored = [vertex.tolerance(), edge.tolerance(), face.tolerance()];
    let effective = [
        BRepTool::vertex_tolerance(&Vertex::unchecked(shape(Handle::new(vertex).into_cast())))
            .unwrap(),
        BRepTool::edge_tolerance(&Edge::unchecked(shape(Handle::new(edge).into_cast()))).unwrap(),
        BRepTool::face_tolerance(&Face::unchecked(shape(Handle::new(face).into_cast()))).unwrap(),
    ];
    std::array::from_fn(|index| (stored[index], effective[index]))
}

fn shape(native: Handle<dyn TShape>) -> Shape {
    Shape::from_parts(native, Location::default(), Orientation::Forward)
}

#[test]
fn raw_nan_negative_and_subfloor_metadata_survive_the_unchanged_accessors() {
    for raw in [f64::NAN, -0.125, 0., 1e-8] {
        for (stored, effective) in authored_tolerances(raw) {
            assert_eq!(stored.to_bits(), raw.to_bits());
            assert_eq!(effective, 1e-7);
        }
    }
}

#[test]
fn cap_uses_raw_metadata_even_when_compatibility_accessor_is_larger() {
    let request = PrismToleranceRequest::new(StoredToleranceCap::refuse(0.).unwrap(), 0.).unwrap();
    for (raw, effective) in authored_tolerances(0.) {
        let row = request.observe(raw, effective).unwrap();
        assert_eq!(row.decision, StoredCapDecision::Within);
        assert_eq!((row.raw, row.effective), (0., 1e-7));
    }
}

#[test]
fn native_infinite_and_invalid_raw_metadata_is_not_hidden_by_effective_capture() {
    let request = PrismToleranceRequest::new(StoredToleranceCap::NoLimit, 0.).unwrap();
    for raw in [f64::NAN, -0.125, f64::INFINITY] {
        for (stored, effective) in authored_tolerances(raw) {
            let error = request.observe(stored, effective).unwrap_err();
            assert_eq!(error.quantity, ToleranceQuantity::RawStoredMetadata);
            assert_eq!(error.actual.to_bits(), raw.to_bits());
        }
    }
}
