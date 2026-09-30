use super::super::{flag, int, point2};
use rustycad_tkernel::{
    ncollection::Shared,
    standard::{Handle, SharedState},
};
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshFaceChecker, CurvePointReference, FaceSegment},
    imesh_tools::MeshParameters,
};
#[path = "../../tests/support/face_checker_fixture.rs"]
pub(super) mod fixture;
pub fn face_checker_case(v: &[i32]) {
    let (model, face) = fixture::checker_model(v);
    let parameters = Handle::new(Shared::new(SharedState::new(MeshParameters {
        in_parallel: v[4] != 0,
        ..Default::default()
    })));
    let checker = BRepMeshFaceChecker::new(&face, &parameters);
    flag(checker.get_intersecting_edges().is_null());
    report_checker(&checker, &model);
    parameters.write().in_parallel = v[4] == 0;
    report_checker(&checker, &model);
    println!();
}
fn report_checker(
    checker: &BRepMeshFaceChecker,
    model: &Handle<dyn rustycad_tkmesh::imesh_data::MeshModel>,
) {
    match checker.perform() {
        Ok(result) => flag(result),
        Err(error) => print!("{} ", error.exception_type()),
    }
    let edges = checker.get_intersecting_edges();
    flag(!edges.is_null());
    int(edges.read().len() as i32);
    for index in 0..model.edges_nb() {
        flag(
            edges
                .read()
                .contains(&model.get_edge(index).unwrap().downgrade()),
        );
    }
}
pub fn face_segment_case() {
    let (model, _face) = fixture::checker_model(&[1, 0, 0, 0, 0, 0, 0]);
    let edge = model.get_edge(0).unwrap();
    let curve = edge.get_pcurve(0).unwrap();
    let first = CurvePointReference::new(curve.downgrade(), 0);
    let last = CurvePointReference::new(curve.downgrade(), 1);
    let segment = FaceSegment::new(edge.downgrade(), Some(first), Some(last));
    flag(segment.edge_ptr == edge.downgrade());
    let empty = FaceSegment::default();
    flag(empty.edge_ptr.is_null());
    flag(empty.point1.is_none());
    flag(empty.point2.is_none());
    point2(segment.point1.as_ref().unwrap().get().unwrap());
    curve
        .set_point(0, rustycad_tkmath::gp::Point2::new(7.0, 8.0))
        .unwrap();
    point2(segment.point1.as_ref().unwrap().get().unwrap());
    segment
        .point2
        .as_ref()
        .unwrap()
        .set(rustycad_tkmath::gp::Point2::new(9.0, 10.0))
        .unwrap();
    point2(curve.get_point(1).unwrap());
    println!();
}
