//! Focused IMeshData/BRepMeshData regression coverage; native twin is tkmesh_probe.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Edge, Face, Shape, Wire},
};
use rustycad_tkernel::{ncollection::IncrementalAllocatorObject, standard::Handle};
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{Axis3, Point2, Point3};
use rustycad_tkmesh::{brep_mesh_data::*, imesh_data::*};
fn allocator() -> Handle<IncrementalAllocatorObject> {
    Handle::new(IncrementalAllocatorObject::default())
}
fn source_face() -> Face {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}
fn source_edge() -> Edge {
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    edge
}
#[test]
fn status_bits_and_exact_masks_are_distinct() {
    let owner = StatusOwner::default();
    assert!(owner.is_equal(Status::NoError));
    assert!(!owner.is_set(Status::NoError));
    owner.set_status(Status::ReMesh);
    owner.set_status(Status::UnorientedWire);
    assert_eq!(owner.get_status_mask(), 24);
    assert!(owner.is_set(Status::ReMesh));
    assert!(!owner.is_equal(Status::ReMesh));
    owner.unset_status(Status::ReMesh);
    assert!(owner.is_equal(Status::UnorientedWire));
    owner.unset_status(Status::NoError);
    assert_eq!(owner.get_status_mask(), 16);
}
#[test]
fn curve_mutation_preserves_point_parameter_alignment() {
    let curve = DiscreteCurve::new(&allocator());
    curve.add_point(Point3::new(1.0, 2.0, 3.0), 2.0);
    curve
        .insert_point(0, Point3::new(4.0, 5.0, 6.0), 7.0)
        .unwrap();
    curve.set_point(1, Point3::new(8.0, 9.0, 10.0)).unwrap();
    curve.set_parameter(1, 11.0).unwrap();
    assert_eq!(curve.get_point(1).unwrap(), Point3::new(8.0, 9.0, 10.0));
    assert_eq!(curve.get_parameter(1).unwrap(), 11.0);
    curve.remove_point(0).unwrap();
    assert_eq!(curve.parameters_nb(), 1);
    assert_eq!(curve.get_parameter(0).unwrap(), 11.0);
    curve.remove_parameter(0).unwrap();
    assert_eq!(curve.parameters_nb(), 0);
}
#[test]
fn curve_clear_handles_zero_one_two_and_many_samples() {
    for count in 0..8 {
        let curve = DiscreteCurve::new(&allocator());
        for i in 0..count {
            curve.add_point(Point3::new(i.into(), 0.0, 0.0), i.into());
        }
        curve.clear(true);
        assert_eq!(curve.parameters_nb(), count.min(2));
        if count > 0 {
            assert_eq!(curve.get_parameter(0).unwrap(), 0.0);
        }
        if count > 1 {
            assert_eq!(curve.get_parameter(1).unwrap(), f64::from(count - 1));
        }
        curve.clear(false);
        assert_eq!(curve.parameters_nb(), 0);
    }
}
#[test]
fn pcurve_mutation_keeps_indices_and_parameters_with_points() {
    let curve = DiscretePCurve::new(FacePtr::default(), Orientation::External, &allocator());
    curve.add_point(Point2::new(1.0, 2.0), 3.0);
    curve.set_index(0, 42).unwrap();
    curve.insert_point(0, Point2::new(4.0, 5.0), 6.0).unwrap();
    assert_eq!(curve.get_index(0).unwrap(), 0);
    curve.set_parameter(0, 7.0).unwrap();
    curve.set_point(0, Point2::new(8.0, 9.0)).unwrap();
    assert_eq!(curve.get_parameter(0).unwrap(), 7.0);
    assert_eq!(curve.get_point(0).unwrap(), Point2::new(8.0, 9.0));
    curve.remove_point(0).unwrap();
    assert_eq!(curve.get_index(0).unwrap(), 42);
    curve.clear(true);
    assert_eq!(curve.parameters_nb(), 1);
    curve.remove_parameter(0).unwrap();
    assert_eq!(curve.parameters_nb(), 0);
    curve.clear(false);
    assert!(curve.get_index(0).is_err());
}
#[test]
fn pcurve_orientation_flags_and_checked_access_match_native() {
    for orientation in [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ] {
        let curve = DiscretePCurve::new(FacePtr::default(), orientation, &allocator());
        assert_eq!(curve.get_orientation(), orientation);
        assert!(curve.get_face().is_null());
        assert_eq!(curve.is_forward(), orientation != Orientation::Reversed);
        assert_eq!(curve.is_internal(), orientation == Orientation::Internal);
        assert_empty_pcurve_access(&curve);
    }
}
#[test]
fn array_adaptor_tracks_shared_parameter_mutation_and_runtime_ancestry() {
    let curve = Handle::new(DiscreteCurve::new(&allocator()));
    let adaptor = ParametersListArrayAdaptor::new(curve.cast());
    assert_eq!(adaptor.lower(), 0);
    assert_eq!(adaptor.upper(), -1);
    curve
        .get()
        .unwrap()
        .add_point(Point3::new(0.0, 0.0, 0.0), 7.0);
    assert_eq!(adaptor.upper(), 0);
    assert_eq!(adaptor.value(0).unwrap(), 7.0);
    let parameters = curve.cast::<dyn ParametersList>();
    parameters.get().unwrap().set_parameter(0, 8.0).unwrap();
    assert_eq!(adaptor.value(0).unwrap(), 8.0);
    assert!(curve.is_kind_name("IMeshData_Curve"));
    assert!(curve.is_kind_name("IMeshData_ParametersList"));
    assert!(!curve.cast::<dyn MeshPCurve>().is_some());
}
#[test]
fn face_status_validity_requires_exact_allowed_mask() {
    let face = DiscreteFace::new(&source_face(), &allocator()).unwrap();
    assert!(face.is_valid());
    face.status().set_status(Status::ReMesh);
    assert!(face.is_valid());
    face.status().set_status(Status::UnorientedWire);
    assert!(!face.is_valid());
    face.status().unset_status(Status::ReMesh);
    assert!(face.is_valid());
    face.status().set_status(Status::Failure);
    assert!(!face.is_valid());
}
#[test]
fn model_and_children_keep_original_topology_and_shared_identity() {
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    let model = Handle::new(DiscreteModel::new(&compound));
    let object = model.get().unwrap();
    assert_eq!(object.get_max_size(), 0.0);
    object.set_max_size(8.0);
    assert_eq!(object.get_max_size(), 8.0);
    assert!(object.get_shape().is_same(&compound));
    assert_eq!(object.faces_nb(), 0);
    assert_eq!(object.edges_nb(), 0);
    let face = assert_model_faces(object);
    let edge = object.add_edge(&source_edge()).unwrap();
    assert_eq!(object.edges_nb(), 1);
    assert!(object.get_edge(0).unwrap() == edge);
    object.set_shape(&face);
    assert!(object.get_shape().is_same(&face));
    assert!(model.is_kind_name("IMeshData_Model"));
    assert!(edge.is_kind_name("IMeshData_Shape"));
}
#[test]
fn edge_flags_are_copied_then_mutate_independently_of_topology() {
    let topology = source_edge();
    let edge = DiscreteEdge::new(&topology, &allocator()).unwrap();
    assert!(edge.get_edge().unwrap().is_same(&topology));
    assert!(edge.get_same_param());
    assert!(edge.get_same_range());
    assert!(!edge.get_degenerated());
    assert_eq!(edge.get_deflection(), f64::MAX);
    assert_eq!(edge.get_angular_deflection(), f64::MAX);
    assert_changed_edge_flags(&edge);
    edge.status().set_status(Status::Outdated);
    assert_eq!(edge.status().get_status_mask(), 64);
    edge.set_shape(&Shape::default());
    assert!(edge.get_shape().is_null());
    assert!(edge.get_edge().unwrap().is_null());
}
#[test]
fn pcurve_lookup_deliberately_uses_first_or_last_not_orientation_search() {
    let face: FaceHandle =
        Handle::new(DiscreteFace::new(&source_face(), &allocator()).unwrap()).cast();
    let edge = DiscreteEdge::new(&source_edge(), &allocator()).unwrap();
    assert!(edge.is_free());
    let first = edge.add_pcurve(face.downgrade(), Orientation::Forward);
    let middle = edge.add_pcurve(face.downgrade(), Orientation::Reversed);
    let last = edge.add_pcurve(face.downgrade(), Orientation::Internal);
    assert_eq!(edge.pcurves_nb(), 3);
    assert!(!edge.is_free());
    assert!(edge.get_pcurve(1).unwrap() == middle);
    assert_eq!(edge.get_pcurves(&face.downgrade()).unwrap(), vec![0, 1, 2]);
    assert_pcurve_selection(&edge, &face, &first, &last);
}
#[test]
fn edge_clear_visits_3d_and_all_pcurves() {
    let edge = DiscreteEdge::new(&source_edge(), &allocator()).unwrap();
    let pcurve = edge.add_pcurve(FacePtr::default(), Orientation::Internal);
    fill_edge_curves(&edge, &pcurve);
    edge.clear(true).unwrap();
    assert_eq!(edge.get_curve().get().unwrap().parameters_nb(), 2);
    assert_eq!(pcurve.get().unwrap().parameters_nb(), 2);
    edge.clear(false).unwrap();
    assert_eq!(pcurve.get().unwrap().parameters_nb(), 0);
    let replacement: CurveHandle = Handle::new(DiscreteCurve::new(&allocator())).cast();
    edge.set_curve(replacement.clone());
    assert!(edge.get_curve() == replacement);
}
#[test]
fn wire_and_pcurve_references_do_not_own_their_targets() {
    let face: FaceHandle =
        Handle::new(DiscreteFace::new(&source_face(), &allocator()).unwrap()).cast();
    let mut topology = Wire::default();
    BRepBuilder::default().make_wire(&mut topology);
    let wire = face.get().unwrap().add_wire(&topology, 0);
    assert_eq!(face.get().unwrap().wires_nb(), 1);
    assert!(face.get().unwrap().get_wire(0).unwrap() == wire);
    assert!(wire.get().unwrap().get_wire().unwrap().is_same(&topology));
    assert_weak_wire_edge(&wire);
    wire.get().unwrap().status().set_status(Status::OpenWire);
    assert!(wire.get().unwrap().status().is_set(Status::OpenWire));
    wire.get().unwrap().set_deflection(0.5);
    assert_eq!(wire.get().unwrap().get_deflection(), 0.5);
}

fn assert_empty_pcurve_access(curve: &DiscretePCurve) {
    assert_eq!(
        curve.get_point(-1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert!(curve.get_index(0).is_err());
    assert!(curve.get_parameter(-1).is_err());
    assert!(curve.set_index(0, 1).is_err());
    assert!(curve.set_point(-1, Point2::new(0.0, 0.0)).is_err());
}

fn assert_model_faces(object: &DiscreteModel) -> Face {
    let face = source_face();
    let first = object.add_face(&face).unwrap();
    let second = object.add_face(&face).unwrap();
    assert_eq!(object.faces_nb(), 2);
    assert!(object.get_face(0).unwrap() == first);
    assert!(first != second);
    assert!(first.get().unwrap().get_face().unwrap().is_same(&face));
    assert!(
        first
            .get()
            .unwrap()
            .get_surface()
            .get()
            .unwrap()
            .face()
            .is_same(&face)
    );
    face
}

fn assert_changed_edge_flags(edge: &DiscreteEdge) {
    edge.set_deflection(0.25);
    edge.set_angular_deflection(0.5);
    edge.set_same_param(false);
    edge.set_same_range(false);
    edge.set_degenerated(true);
    assert_eq!(edge.get_deflection(), 0.25);
    assert_eq!(edge.get_angular_deflection(), 0.5);
    assert!(!edge.get_same_param());
    assert!(!edge.get_same_range());
    assert!(edge.get_degenerated());
}

fn assert_pcurve_selection(
    edge: &DiscreteEdge,
    face: &FaceHandle,
    first: &PCurveHandle,
    last: &PCurveHandle,
) {
    let pointer = face.downgrade();
    let forward = edge.get_pcurve_for(&pointer, Orientation::Forward).unwrap();
    let reversed = edge
        .get_pcurve_for(&pointer, Orientation::Reversed)
        .unwrap();
    assert!(forward == *first);
    assert!(reversed == *last);
    assert!(first.get().unwrap().get_face().upgrade() == *face);
    assert_eq!(
        edge.get_pcurves(&FacePtr::default())
            .unwrap_err()
            .exception_type(),
        "Standard_NoSuchObject"
    );
}

fn fill_edge_curves(edge: &DiscreteEdge, pcurve: &PCurveHandle) {
    for i in 0..4 {
        edge.get_curve()
            .get()
            .unwrap()
            .add_point(Point3::new(i.into(), 0.0, 0.0), i.into());
        pcurve
            .get()
            .unwrap()
            .add_point(Point2::new(i.into(), 0.0), i.into());
    }
}

fn assert_weak_wire_edge(wire: &WireHandle) {
    let edge: EdgeHandle =
        Handle::new(DiscreteEdge::new(&source_edge(), &allocator()).unwrap()).cast();
    assert_eq!(
        wire.get()
            .unwrap()
            .add_edge(edge.downgrade(), Orientation::External),
        0
    );
    assert_eq!(edge.ref_count(), 1);
    assert_eq!(wire.get().unwrap().edges_nb(), 1);
    assert_eq!(
        wire.get().unwrap().get_edge_orientation(0).unwrap(),
        Orientation::External
    );
    let pointer = wire.get().unwrap().get_edge(0).unwrap();
    assert!(pointer.upgrade() == edge);
    drop(edge);
    assert!(pointer.upgrade().is_null());
}

#[test]
fn native_status_casts_accept_combined_and_unlisted_bits() {
    let owner = StatusOwner::default();
    owner.set_status(Status(0x400 | 24));
    assert!(owner.is_equal(Status(0x418)));
    assert!(owner.is_set(Status(24)));
    owner.unset_status(Status(24));
    assert_eq!(owner.get_status_mask(), 0x400);
    owner.unset_status(Status(-1));
    assert!(owner.is_equal(Status::NoError));
}
