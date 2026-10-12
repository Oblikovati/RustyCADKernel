//! Actual native EvalD0 mutates shared edge vertices, so a later Orientation query raises before contour append.
use super::{
    StdSelectBrepSelectionTool, brep_contour_mutation_curve::BrepContourMutationCurveFake,
};
use crate::{select_mgr::SelectMgrEntityOwner, select3d::Select3DSensitivePoint};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face, Wire},
};
use rustycad_tkernel::{ncollection::Array2, standard::Handle};
use rustycad_tkg3d::geom::GeomBezierSurface;
use rustycad_tkmath::gp::Point3;
use std::sync::atomic::Ordering;
#[test]
fn native_curve_topology_mutation_rereads_orientation_before_sampling_and_preserves_output() {
    let curve = BrepContourMutationCurveFake::new();
    let edge = BrepContourMutationCurveFake::edge_fixture(&curve);
    let face = contour_mutation_face(&edge);
    let owner = SelectMgrEntityOwner::new(5).into_cast();
    let mut entities =
        vec![Handle::new(Select3DSensitivePoint::new(&owner, Point3::new(9., 8., 7.))).cast()];
    curve.activate(&edge);
    let result = invoke_mutating_contour(&face, &owner, &mut entities);
    assert_eq!(
        result.unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    let row = mutating_contour_row(&curve, entities.len(), &edge);
    assert_eq!(
        row,
        include_str!("../../tests/fixtures/colored-shape-compute/native-contour-orientation.txt")
    );
}

fn invoke_mutating_contour(
    face: &Face,
    owner: &Handle<dyn crate::select_mgr::SelectMgrEntityOwnerSource>,
    entities: &mut Vec<Handle<dyn crate::select3d::Select3DSensitiveEntity>>,
) -> Result<bool, rustycad_tkernel::standard::Failure> {
    StdSelectBrepSelectionTool::get_sensitive_for_face(face, owner, entities, false, 9, 500., true)
}
fn mutating_contour_row(curve: &BrepContourMutationCurveFake, count: usize, edge: &Edge) -> String {
    let calls = curve.calls.load(Ordering::Relaxed);
    format!(
        "contour-mutation 1 {calls} {count} {}\n",
        edge.nb_children()
    )
}
pub(super) fn contour_mutation_face(edge: &Edge) -> Face {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    let mut face = Face::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, edge).unwrap();
    let poles = contour_mutation_poles();
    let surface = Handle::new(GeomBezierSurface::new(&poles).unwrap()).into_cast();
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    builder.add(&mut face, &wire).unwrap();
    face
}
fn contour_mutation_poles() -> Array2<'static, Point3> {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles.set_value(1, 1, Point3::new(0., 0., 0.)).unwrap();
    poles.set_value(1, 2, Point3::new(0., 1., 0.)).unwrap();
    poles.set_value(2, 1, Point3::new(1., 0., 0.)).unwrap();
    poles.set_value(2, 2, Point3::new(1., 1., 1.)).unwrap();
    poles
}
