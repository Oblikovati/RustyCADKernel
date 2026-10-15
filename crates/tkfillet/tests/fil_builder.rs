//! ChFi3d_FilBuilder on boxes, checked against closed forms: a constant-radius fillet
//! on an edge of a cube removes `(r^2 - pi r^2 / 4) L` of volume and replaces two strips
//! of width `r` by a quarter cylinder; the result passes BRepCheck.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkfillet::chfi3d::{ChFi3dFilBuilder, FilletShape};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::f64::consts::PI;

fn cube(side: f64) -> Shape {
    BRepPrimAPIMakeBox::from_sizes(side, side, side)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn first_edge(s: &Shape) -> topods::Edge {
    topods::edge(Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape).unwrap().current()).unwrap()
}

fn volume(s: &Shape) -> f64 {
    let mut p = GProps::new();
    BRepGProp::volume_properties(s, &mut p, false, false, false).unwrap();
    p.mass()
}

fn area(s: &Shape) -> f64 {
    let mut p = GProps::new();
    BRepGProp::surface_properties(s, &mut p, false, false).unwrap();
    p.mass()
}

#[test]
fn a_constant_fillet_on_a_cube_edge_has_the_quarter_cylinder_volume_and_area() {
    let (side, r) = (10., 2.);
    let solid = cube(side);
    let mut b = ChFi3dFilBuilder::new(&solid, FilletShape::Rational, 1.0e-2).unwrap();
    b.add_with_radius(r, &first_edge(&solid)).unwrap();
    assert_eq!(b.builder_base().nb_elements(), 1);
    b.compute().unwrap();
    assert!(b.builder_base().is_done());
    let result = b.builder_base().shape().unwrap();
    let lost = (r * r - PI * r * r / 4.) * side;
    let expected_volume = side.powi(3) - lost;
    let expected_area = 6. * side * side - 2. * r * side - 2. * (r * r - PI * r * r / 4.) + PI * r / 2. * side;
    let v = volume(&result);
    let a = area(&result);
    assert!((v - expected_volume).abs() < 1.0e-3 * expected_volume, "volume {v} expected {expected_volume}");
    assert!((a - expected_area).abs() < 1.0e-3 * expected_area, "area {a} expected {expected_area}");
    let check = BRepCheckAnalyzer::new(&result, true, false, false).unwrap();
    assert!(check.is_valid().unwrap());
}
