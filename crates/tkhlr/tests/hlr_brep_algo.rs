//! HLRBRep_Algo, HLRBRep_HLRToShape and HLRAppli_ReflectLines on a box seen on a
//! corner: nine visible and three hidden sharp edges.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkhlr::hlr_algo::Projector;
use rustycad_tkhlr::hlr_appli::ReflectLines;
use rustycad_tkhlr::hlr_brep::{Algo, HLRToShape, TypeOfResultingEdge};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn a_box() -> Shape {
    BRepPrimAPIMakeBox::with_axes(Axis2::default(), 1., 2., 3.)
        .shape()
        .unwrap()
        .clone()
}

fn edges(s: &Shape) -> usize {
    Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .count()
}

fn corner_algo() -> Handle<Algo> {
    let algo = Handle::new(Algo::new().unwrap());
    let a = algo.get().unwrap();
    a.add(&a_box(), 0);
    let axis = Axis2::new(
        Point3::new(0., 0., 0.),
        Direction3::new(1., 1., 1.).unwrap(),
    )
    .unwrap();
    a.set_projector(&Projector::with_frame(axis).unwrap());
    a.update().unwrap();
    a.hide().unwrap();
    algo
}

#[test]
fn a_box_seen_on_a_corner_shows_nine_sharp_edges() {
    let algo = corner_algo();
    let t = HLRToShape::new(&algo);
    assert_eq!(edges(&t.v_compound().unwrap()), 9);
    assert_eq!(edges(&t.h_compound().unwrap()), 3);
    assert!(t.out_line_v_compound().unwrap().is_null());
    let s = algo.get().unwrap().shape_bounds(1).unwrap();
    assert_eq!(s.bounds(), (1, 8, 1, 12, 1, 6));
}

#[test]
fn the_result_of_one_shape_is_the_result_of_the_algorithm() {
    let algo = corner_algo();
    let t = HLRToShape::new(&algo);
    let a = algo.get().unwrap();
    let original = a
        .shape_bounds(1)
        .unwrap()
        .shape()
        .get()
        .unwrap()
        .original_shape();
    assert_eq!(a.index(&original), 1);
    let one = t
        .compound_of_edges_of(&original, TypeOfResultingEdge::Sharp, true, false)
        .unwrap();
    assert_eq!(edges(&one), 9);
}

#[test]
fn the_reflect_lines_of_a_box_have_no_outline() {
    let mut lines = ReflectLines::new(&a_box()).unwrap();
    lines.set_axes(1., 1., 1., 0., 0., 0., 0., 0., 1.).unwrap();
    lines.perform().unwrap();
    assert!(lines.get_result().unwrap().is_null());
    let sharp = lines
        .get_compound_of_3d_edges(TypeOfResultingEdge::Sharp, true, true)
        .unwrap();
    assert_eq!(edges(&sharp), 9);
}
