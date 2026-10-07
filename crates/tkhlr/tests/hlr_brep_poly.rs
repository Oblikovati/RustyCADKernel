//! HLRBRep_PolyAlgo and HLRBRep_PolyHLRToShape on meshed boxes and cylinders.
mod support;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkhlr::hlr_algo::{EdgeStatus, Projector};
use rustycad_tkhlr::hlr_brep::{PolyAlgo, PolyHLRToShape};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use support::grid_mesh::mesh;

fn corner_view() -> Projector {
    let axis = Axis2::new(
        Point3::new(0., 0., 0.),
        Direction3::new(1., 1., 1.).unwrap(),
    )
    .unwrap();
    Projector::with_frame(axis).unwrap()
}

fn edges(s: &Shape) -> usize {
    Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .count()
}

fn poly(s: &Shape, n: i32, m: i32) -> Handle<PolyAlgo> {
    mesh(s, n, m).unwrap();
    let algo = Handle::new(PolyAlgo::with_shape(s).unwrap());
    let a = algo.get().unwrap();
    a.set_projector(&corner_view());
    a.update().unwrap();
    algo
}

fn a_box() -> Shape {
    BRepPrimAPIMakeBox::with_axes(Axis2::default(), 1., 2., 3.)
        .shape()
        .unwrap()
        .clone()
}

#[test]
fn a_shape_without_triangulation_gives_no_segment() {
    let s = a_box();
    let a = PolyAlgo::with_shape(&s).unwrap();
    assert_eq!((a.nb_shapes(), a.index(&s)), (1, 1));
    a.update().unwrap();
    a.init_hide().unwrap();
    assert!(!a.more_hide());
    a.remove(1).unwrap();
    assert_eq!(a.nb_shapes(), 0);
    assert_eq!(
        a.remove(1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn a_meshed_box_seen_on_a_corner_hides_three_edges() {
    let algo = poly(&a_box(), 4, 4);
    let a = algo.get().unwrap();
    let (mut status, mut hidden) = (EdgeStatus::new(), 0);
    a.init_hide().unwrap();
    while a.more_hide() {
        a.hide(&mut status).unwrap();
        if status.all_hidden() {
            hidden += 1;
        }
        a.next_hide().unwrap();
    }
    // Three edges of four segments each meet at the hidden corner.
    assert_eq!(hidden, 12);
    let mut t = PolyHLRToShape::new();
    t.update(&algo).unwrap();
    assert_eq!(edges(&t.h_compound().unwrap()), 12);
    assert_eq!(edges(&t.v_compound().unwrap()), 36);
    assert!(t.out_line_v_compound().unwrap().is_null());
}

#[test]
fn a_meshed_cylinder_has_outlines() {
    let s = BRepPrimAPIMakeCylinder::with_axes(Axis2::default(), 1., 2.)
        .shape()
        .unwrap()
        .clone();
    let algo = poly(&s, 16, 3);
    let mut t = PolyHLRToShape::new();
    t.update(&algo).unwrap();
    assert!(edges(&t.out_line_v_compound().unwrap()) > 0);
    let outlined = algo.get().unwrap().out_lined_shape(&s).unwrap();
    assert!(edges(&outlined) > edges(&s));
}
