//! Model oracle operations mirrored by tkmesh_probe_models.cxx.
use super::*;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Edge, Face, Wire},
};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    geom::{GeomPlane, GeomSurface},
};
use rustycad_tkmath::gp::{Direction3, Plane3};
const STATUSES: [Status; 10] = [
    Status::NoError,
    Status::OpenWire,
    Status::SelfIntersectingWire,
    Status::Failure,
    Status::ReMesh,
    Status::UnorientedWire,
    Status::TooFewPoints,
    Status::Outdated,
    Status::Reused,
    Status::UserBreak,
];
fn status_report(owner: &StatusOwner, mask: i32, unset: i32) {
    owner.set_status(Status(mask));
    int(owner.get_status_mask());
    for value in STATUSES {
        flag(owner.is_set(value));
        flag(owner.is_equal(value));
    }
    owner.unset_status(Status(unset));
    int(owner.get_status_mask());
}
fn edge_report(edge: &dyn MeshEdge) {
    flag(edge.get_same_param());
    flag(edge.get_same_range());
    flag(edge.get_degenerated());
    real(edge.get_deflection());
    real(edge.get_angular_deflection());
    flag(edge.is_free());
    edge.set_same_param(false);
    edge.set_same_range(false);
    edge.set_degenerated(true);
    edge.set_deflection(-2.0);
    edge.set_angular_deflection(0.75);
    flag(edge.get_same_param());
    flag(edge.get_same_range());
    flag(edge.get_degenerated());
    real(edge.get_deflection());
    real(edge.get_angular_deflection());
}
fn planar_face(builder: &BRepBuilder) -> Face {
    let mut face = Face::default();
    let plane = Plane3::new(
        Point3::new(0.0, 0.0, 2.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(plane)).cast();
    builder
        .make_face_on_surface(&mut face, &surface, 1.0e-7)
        .unwrap();
    face
}
pub(super) fn model_case(mask: i32, unset: i32, orientation: Orientation) {
    let builder = BRepBuilder::default();
    let mut shape = Compound::default();
    builder.make_compound(&mut shape);
    let model = DiscreteModel::new(&shape);
    flag(model.get_shape().is_same(&shape));
    real(model.get_max_size());
    model.set_max_size(19.0);
    real(model.get_max_size());
    int(model.faces_nb());
    int(model.edges_nb());
    let (topo_face, face) = model_face(&builder, &model, mask, unset);
    let object = face.get().unwrap();
    let wire = model_wire(&builder, object, mask, unset);
    let edge = model_edge(&builder, &model, mask, unset);
    int(wire.get().unwrap().add_edge(edge.downgrade(), orientation));
    flag(wire.get().unwrap().get_edge(0).unwrap().upgrade() == edge);
    int(wire.get().unwrap().get_edge_orientation(0).unwrap() as i32);
    model_pcurves(&edge, &face);
    model_finish(&model, &topo_face, &edge, &wire, &face);
}

fn model_wire(builder: &BRepBuilder, face: &dyn MeshFace, mask: i32, unset: i32) -> WireHandle {
    let mut topo_wire = Wire::default();
    builder.make_wire(&mut topo_wire);
    let wire = face.add_wire(&topo_wire, 3);
    flag(face.get_wire(0).unwrap() == wire);
    let object = wire.get().unwrap();
    flag(object.get_wire().unwrap().is_same(&topo_wire));
    int(object.edges_nb());
    real(object.get_deflection());
    object.set_deflection(0.5);
    real(object.get_deflection());
    status_report(object.status(), mask, unset);
    wire
}
fn model_edge(builder: &BRepBuilder, model: &DiscreteModel, mask: i32, unset: i32) -> EdgeHandle {
    let mut topo_edge = Edge::default();
    builder.make_edge(&mut topo_edge).unwrap();
    let edge = model.add_edge(&topo_edge).unwrap();
    flag(model.get_edge(0).unwrap() == edge);
    flag(edge.get().unwrap().get_edge().unwrap().is_same(&topo_edge));
    edge_report(edge.get().unwrap());
    status_report(edge.get().unwrap().status(), mask, unset);
    edge
}
fn model_pcurves(edge: &EdgeHandle, face: &FaceHandle) {
    let object = edge.get().unwrap();
    let first = object.add_pcurve(face.downgrade(), Orientation::Forward);
    let second = object.add_pcurve(face.downgrade(), Orientation::Reversed);
    let third = object.add_pcurve(face.downgrade(), Orientation::Internal);
    int(object.pcurves_nb());
    flag(object.is_free());
    flag(object.get_pcurve(0).unwrap() == first);
    pcurve_lookup_report(object, face, &first, &third);
    model_clear(object, first.get().unwrap(), second.get().unwrap());
    let previous = object.get_curve();
    object.set_curve(
        Handle::new(DiscreteCurve::new(&Handle::new(
            IncrementalAllocatorObject::default(),
        )))
        .cast(),
    );
    flag(object.get_curve() != previous);
}
fn model_clear(edge: &dyn MeshEdge, first: &dyn MeshPCurve, second: &dyn MeshPCurve) {
    for i in 0..4 {
        edge.get_curve()
            .get()
            .unwrap()
            .add_point(Point3::new(i.into(), 0.0, 0.0), i.into());
        first.add_point(Point2::new(i.into(), 0.0), i.into());
    }
    edge.clear(true).unwrap();
    int(edge.get_curve().get().unwrap().parameters_nb());
    int(first.parameters_nb());
    int(second.parameters_nb());
    edge.clear(false).unwrap();
    int(edge.get_curve().get().unwrap().parameters_nb());
}

fn model_face(
    builder: &BRepBuilder,
    model: &DiscreteModel,
    mask: i32,
    unset: i32,
) -> (Face, FaceHandle) {
    let topo_face = planar_face(builder);
    let face = model.add_face(&topo_face).unwrap();
    flag(model.get_face(0).unwrap() == face);
    flag(face.get().unwrap().get_face().unwrap().is_same(&topo_face));
    let object = face.get().unwrap();
    point3(object.get_surface().get().unwrap().value(2.0, 3.0).unwrap());
    real(object.get_deflection());
    object.set_deflection(0.25);
    real(object.get_deflection());
    status_report(object.status(), mask, unset);
    flag(object.is_valid());
    int(object.wires_nb());
    (topo_face, face)
}

fn model_finish(
    model: &DiscreteModel,
    topo_face: &Face,
    edge: &EdgeHandle,
    wire: &WireHandle,
    face: &FaceHandle,
) {
    model.set_shape(topo_face);
    flag(model.get_shape().is_same(topo_face));
    int(model.faces_nb());
    int(model.edges_nb());
    flag(edge.is_kind_name("IMeshData_Shape"));
    flag(wire.is_kind_name("IMeshData_TessellatedShape"));
    flag(face.is_kind_name("IMeshData_Face"));
    println!();
}

fn pcurve_lookup_report(
    object: &dyn MeshEdge,
    face: &FaceHandle,
    first: &PCurveHandle,
    third: &PCurveHandle,
) {
    let pointer = face.downgrade();
    let forward = object
        .get_pcurve_for(&pointer, Orientation::Forward)
        .unwrap();
    let reversed = object
        .get_pcurve_for(&pointer, Orientation::Reversed)
        .unwrap();
    let external = object
        .get_pcurve_for(&pointer, Orientation::External)
        .unwrap();
    flag(forward == *first);
    flag(reversed == *third);
    flag(external == *third);
    flag(first.get().unwrap().get_face().upgrade() == *face);
    for index in object.get_pcurves(&face.downgrade()).unwrap() {
        int(index);
    }
}

pub(super) mod tools;

pub mod pipeline;

pub mod shape_tool;

pub mod wire_foundation;

pub mod model_builder;

pub mod deflection;

pub mod post_processor;

pub mod edge_parameter;

pub mod tessellation_extractor;

pub mod curve_tessellator;

pub mod edge_discret;

pub mod range_splitter;

pub mod pre_processor;

pub mod geom_tool;

pub mod editable_tree;

pub mod face_checker;

pub mod model_healer;

pub mod elements;

pub mod cell_failure;

pub mod vertex_inspector;

pub mod vertex_tool;

pub mod delaun_structure;

pub mod delaun_selector;

pub mod circle_tool;

pub mod delaun_core;
pub mod delaun_legacy;
