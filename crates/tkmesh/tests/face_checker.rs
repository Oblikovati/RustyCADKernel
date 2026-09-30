//! Native intersection state, live references and repeat-call behavior.
use rustycad_tkernel::{
    ncollection::Shared,
    standard::{Handle, SharedState},
};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshFaceChecker, CurvePointReference, FaceCheckerParameters, FaceSegment},
    imesh_tools::MeshParameters,
};
#[path = "support/face_checker_fixture.rs"]
mod fixture;
#[path = "../../tkshhealing/tests/support/wire_foundation.rs"]
mod wire_fixture;
fn parameters() -> FaceCheckerParameters {
    Handle::new(Shared::new(SharedState::new(MeshParameters::default())))
}
#[test]
fn actual_runtime_ancestry_and_initial_null_result() {
    let (_model, face) = fixture::checker_model(&[0, 0, 0, 0, 0, 0, 0]);
    let checker = Handle::new(BRepMeshFaceChecker::new(&face, &parameters()));
    assert!(checker.is_instance_name("BRepMesh_FaceChecker"));
    assert!(checker.is_kind_name("Standard_Transient"));
    assert!(checker.get_intersecting_edges().is_null());
    assert!(checker.perform().unwrap());
    assert!(checker.get_intersecting_edges().read().is_empty());
}
#[test]
fn bowtie_intersections_name_the_actual_discrete_edges() {
    let (model, face) = fixture::checker_model(&[1, 1, 0, 0, 0, 0, 0]);
    let checker = BRepMeshFaceChecker::new(&face, &parameters());
    assert!(!checker.perform().unwrap());
    let edges = checker.get_intersecting_edges();
    assert_eq!(edges.read().len(), 2);
    assert!(
        edges
            .read()
            .contains(&model.get_edge(0).unwrap().downgrade())
    );
    assert!(
        edges
            .read()
            .contains(&model.get_edge(2).unwrap().downgrade())
    );
}
#[test]
fn small_loop_filter_uses_twice_face_deflection() {
    for (deflection, clear) in [(0, false), (3, true)] {
        let (_model, face) = fixture::checker_model(&[2, 0, 0, deflection, 0, 0, 0]);
        let checker = BRepMeshFaceChecker::new(&face, &parameters());
        assert_eq!(checker.perform().unwrap(), clear);
    }
}
#[test]
fn shared_parameter_owner_and_previous_result_handles_remain_live() {
    let (model, face) = fixture::checker_model(&[1, 0, 0, 0, 0, 0, 0]);
    let settings = parameters();
    let checker = BRepMeshFaceChecker::new(&face, &settings);
    assert!(!checker.perform().unwrap());
    let old = checker.get_intersecting_edges();
    let curve = model.get_edge(0).unwrap().get_pcurve(0).unwrap();
    for (index, point) in fixture::checker_points(0).into_iter().enumerate() {
        curve.set_point(index as i32, point).unwrap();
    }
    settings.write().in_parallel = true;
    assert!(checker.perform().unwrap());
    assert_eq!(old.read().len(), 1);
    assert!(checker.get_intersecting_edges().read().is_empty());
    assert!(old != checker.get_intersecting_edges());
}
#[test]
fn parallel_adjacent_wire_intersections_match_sequential() {
    let (model, face) = fixture::checker_model(&[0, 1, 1, 0, 0, 0, 0]);
    let settings = parameters();
    let checker = BRepMeshFaceChecker::new(&face, &settings);
    assert!(!checker.perform().unwrap());
    let first = checker.get_intersecting_edges();
    settings.write().in_parallel = true;
    assert!(!checker.perform().unwrap());
    let second = checker.get_intersecting_edges();
    for index in 0..model.edges_nb() {
        let edge = model.get_edge(index).unwrap().downgrade();
        assert_eq!(first.read().contains(&edge), second.read().contains(&edge));
    }
}
#[test]
fn segment_point_associations_reflect_changes_in_both_directions() {
    let (model, _face) = fixture::checker_model(&[0, 0, 0, 0, 0, 0, 0]);
    let edge = model.get_edge(0).unwrap();
    let curve = edge.get_pcurve(0).unwrap();
    let reference = CurvePointReference::new(curve.downgrade(), 0);
    let segment = FaceSegment::new(edge.downgrade(), Some(reference.clone()), Some(reference));
    assert_eq!(segment.edge_ptr, edge.downgrade());
    curve.set_point(0, Point2::new(7.0, 8.0)).unwrap();
    assert_eq!(
        segment.point1.as_ref().unwrap().get().unwrap(),
        Point2::new(7.0, 8.0)
    );
    segment
        .point2
        .as_ref()
        .unwrap()
        .set(Point2::new(9.0, 10.0))
        .unwrap();
    assert_eq!(curve.get_point(0).unwrap(), Point2::new(9.0, 10.0));
    let empty = FaceSegment::default();
    assert!(empty.edge_ptr.is_null() && empty.point1.is_none() && empty.point2.is_none());
}
#[test]
fn empty_faces_empty_curves_and_repeated_points_keep_native_success() {
    for kind in [-1, 0, 8] {
        let (_model, face) = fixture::checker_model(&[kind, 0, 0, 0, 0, 0, 0]);
        assert!(
            BRepMeshFaceChecker::new(&face, &parameters())
                .perform()
                .unwrap()
        );
    }
    let (_model, face) = fixture::checker_model(&[0, 1, 0, 0, 0, 0, 1]);
    assert!(
        BRepMeshFaceChecker::new(&face, &parameters())
            .perform()
            .unwrap()
    );
}
