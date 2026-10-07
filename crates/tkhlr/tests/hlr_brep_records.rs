//! HLRBRep_EdgeData, HLRBRep_FaceData, HLRBRep_FaceIterator, HLRBRep_EdgeFaceTool and the
//! package functions.
use rustycad_tkbrep::topods::Face;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkhlr::hlr_algo::Projector;
use rustycad_tkhlr::hlr_brep::{
    EdgeData, EdgeFaceTool, FaceData, FaceIterator, Surface, TypeOfResultingEdge,
    poly_hlr_angle_and_deflection,
};
use rustycad_tkmath::gp::{Axis3, Direction3, Sphere3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use std::sync::Arc;

fn sphere_face() -> Face {
    let sphere = Sphere3::new(Axis3::default(), 2.).unwrap();
    BRepBuilderAPIMakeFace::from_sphere(sphere)
        .unwrap()
        .face()
        .unwrap()
}

#[test]
fn edge_flags_are_independent_bits() {
    let mut e = EdgeData::new();
    e.set_vertical(true);
    e.set_used(true);
    e.set_selected(false);
    assert!(e.vertical() && e.used() && !e.selected() && !e.simple());
    e.set_vertical(false);
    assert!(!e.vertical() && e.used());
}

#[test]
fn the_face_orientation_lives_in_the_low_bits() {
    let mut f = FaceData::new();
    f.set_torus(true);
    f.set_orientation(Orientation::Internal);
    assert_eq!(f.orientation(), Orientation::Internal);
    assert!(f.torus() && f.selected());
    f.set_orientation(Orientation::Forward);
    assert_eq!(f.orientation(), Orientation::Forward);
    assert!(f.torus());
}

#[test]
fn the_face_iterator_walks_every_wire_edge() {
    let mut f = FaceData::new();
    f.set(&sphere_face(), Orientation::Forward, false, 2)
        .unwrap();
    f.set_wire(1, 2).unwrap();
    f.set_wire(2, 1).unwrap();
    f.set_w_edge(1, 1, 7, Orientation::Reversed, true, false, false, false)
        .unwrap();
    f.set_w_edge(1, 2, 8, Orientation::Forward, false, false, false, false)
        .unwrap();
    f.set_w_edge(2, 1, 9, Orientation::Forward, false, true, false, false)
        .unwrap();
    let mut it = FaceIterator::new();
    it.init_edge(&f).unwrap();
    let mut seen = Vec::new();
    while it.more_edge() {
        seen.push((it.edge().unwrap(), it.beginning_of_wire(), it.end_of_wire()));
        it.next_edge().unwrap();
    }
    assert_eq!(
        seen,
        vec![(7, true, false), (8, false, true), (9, true, true)]
    );
}

#[test]
fn a_sphere_has_its_normal_curvature_in_every_direction() {
    let mut s = Surface::new();
    s.set_projector(Some(Arc::new(Projector::new().unwrap())));
    s.set_surface(&sphere_face()).unwrap();
    let d = Direction3::new(0., 0., 1.).unwrap();
    let k = EdgeFaceTool::curvature_value(&s, 0.3, 0.2, d).unwrap();
    assert!((k.abs() - 0.5).abs() < 1e-12);
}

#[test]
fn the_polyhedral_angle_is_clamped_then_mapped() {
    let (a, d) = poly_hlr_angle_and_deflection(0.);
    assert!((a - 5. * std::f64::consts::PI / 180.).abs() < 1e-15);
    assert_eq!(d, a * a * 0.5);
    assert_eq!(TypeOfResultingEdge::Sharp as i32, 5);
}
