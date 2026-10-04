use super::*;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkg3d::{geom::GeomCylindricalSurface, top_abs::ShapeEnum};
use rustycad_tkmath::gp::Axis3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
#[test]
fn native_closed_predicate_and_direct_seam_helper() {
    let (face, old) = cylindrical_seam();
    let (spatial, first, last) = BRepTool::curve(&old).unwrap();
    let mut edge = Edge::default();
    let builder = BRepBuilder::default();
    builder
        .make_edge_with_curve(&mut edge, &spatial, 1e-7)
        .unwrap();
    builder.range(&edge, first, last, false).unwrap();
    let pc = BRepTool::curve_on_face(&old, &face).unwrap();
    builder
        .update_edge_pcurve_on_face(&edge, &pc.pcurve, &face, 1e-7)
        .unwrap();
    assert!(!closed::is_closed(&old, &face).unwrap());
    assert_eq!(
        closed::update_pcurve(&old, &edge, &face, &Context::default()).unwrap(),
        0
    );
    assert!(BRepTool::is_closed_on_face(&edge, &face).unwrap());
}
fn cylindrical_seam() -> (Face, Edge) {
    let surface = Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.).unwrap()).cast();
    let face = BRepBuilderAPIMakeFace::from_surface_bounds(
        &surface,
        (0., std::f64::consts::TAU, -2., 2.),
        1e-7,
    )
    .unwrap()
    .face()
    .unwrap();
    for shape in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        let mut edge = Edge::unchecked(shape);
        if BRepTool::is_closed_on_face(&edge, &face).unwrap() {
            edge.set_orientation(Orientation::Forward);
            return (face, edge);
        }
    }
    panic!("cylindrical fixture has no seam; expected a periodic edge");
}
