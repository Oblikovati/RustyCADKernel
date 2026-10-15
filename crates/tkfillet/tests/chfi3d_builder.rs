//! The state of a ChFi3d_Builder before any contour is added: the maps of the shape, no
//! result, and the queries on contours out of range.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkfillet::chfi3d::ChFi3dBuilder;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn unit_box() -> Shape {
    BRepPrimAPIMakeBox::from_sizes(1., 1., 1.)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

#[test]
fn a_new_builder_has_no_contour_and_no_result() {
    let b = ChFi3dBuilder::new(&unit_box(), 1.0e-2).unwrap();
    assert!(!b.is_done());
    assert!(!b.has_result());
    assert!(b.shape().is_err());
    assert!(b.bad_shape().is_err());
    assert_eq!(b.nb_elements(), 0);
    assert_eq!(b.nb_faulty_contours(), 0);
    assert_eq!(b.faulty_contour(1), 0);
    assert_eq!(b.nb_faulty_vertices(), 0);
    assert!(b.faulty_vertex(1).unwrap().is_null());
}

#[test]
fn contour_queries_out_of_range_are_neutral() {
    let solid = unit_box();
    let b = ChFi3dBuilder::new(&solid, 1.0e-2).unwrap();
    let edge = topods::edge(
        Explorer::new(&solid, ShapeEnum::Edge, ShapeEnum::Shape)
            .unwrap()
            .current(),
    )
    .unwrap();
    assert_eq!(b.contains(&edge).unwrap(), 0);
    assert_eq!(b.length(1).unwrap(), -1.0);
    assert!(b.value(1).is_err());
    assert!(b.stripe_status(1).is_err());
    assert!(b.first_vertex(1).unwrap().is_null());
}
