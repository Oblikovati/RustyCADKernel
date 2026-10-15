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

fn all_edges(s: &Shape) -> Vec<topods::Edge> {
    let mut ex = Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let mut out: Vec<topods::Edge> = Vec::new();
    while ex.more() {
        let e = topods::edge(ex.current()).unwrap();
        if !out.iter().any(|o| o.is_same(&e)) {
            out.push(e);
        }
        ex.next().unwrap();
    }
    out
}

#[test]
fn constant_fillets_on_all_cube_edges_give_the_rounded_cube() {
    // Rounded cube: the inner cube, six slabs, twelve quarter cylinders, eight sphere
    // octants.
    let (a, r) = (10., 1.);
    let solid = cube(a);
    let mut b = ChFi3dFilBuilder::new(&solid, FilletShape::Rational, 1.0e-2).unwrap();
    for e in all_edges(&solid) {
        b.add_with_radius(r, &e).unwrap();
    }
    b.compute().unwrap();
    assert!(b.builder_base().is_done());
    let result = b.builder_base().shape().unwrap();
    let c = a - 2. * r;
    let expected_volume =
        c.powi(3) + 6. * c * c * r + 3. * PI * r * r * c + 4. / 3. * PI * r.powi(3);
    let expected_area = 6. * c * c + 6. * PI * r * c + 4. * PI * r * r;
    let (v, s) = (volume(&result), area(&result));
    let mut ex = Explorer::new(&result, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut nb_faces = 0;
    while ex.more() {
        nb_faces += 1;
        ex.next().unwrap();
    }
    // Six planes, twelve quarter cylinders, eight sphere octants.
    assert_eq!(nb_faces, 26);
    assert!((v - expected_volume).abs() < 1.0e-3 * expected_volume, "volume {v} expected {expected_volume}");
    assert!((s - expected_area).abs() < 1.0e-3 * expected_area, "area {s} expected {expected_area}");
    let check = BRepCheckAnalyzer::new(&result, true, false, false).unwrap();
    assert!(check.is_valid().unwrap());
}

#[test]
fn a_symmetric_chamfer_on_a_cube_edge_removes_the_right_prism() {
    // The chamfer of distance d cuts a right isosceles prism of section d^2 / 2 and
    // replaces two strips of width d by a plane of width d sqrt(2).
    let (side, d) = (10., 2.);
    let solid = cube(side);
    let mut b = rustycad_tkfillet::chfi3d::ChFi3dChBuilder::new(&solid, 1.0e-2).unwrap();
    b.add_with_dist(d, &first_edge(&solid)).unwrap();
    b.compute().unwrap();
    assert!(b.builder_base().is_done());
    let result = b.builder_base().shape().unwrap();
    let expected_volume = side.powi(3) - d * d / 2. * side;
    let expected_area = 6. * side * side - 2. * d * side - d * d + d * 2f64.sqrt() * side;
    let (v, a) = (volume(&result), area(&result));
    assert!((v - expected_volume).abs() < 1.0e-6 * expected_volume, "volume {v} expected {expected_volume}");
    assert!((a - expected_area).abs() < 1.0e-6 * expected_area, "area {a} expected {expected_area}");
    assert!(BRepCheckAnalyzer::new(&result, true, false, false).unwrap().is_valid().unwrap());
}

#[test]
fn a_fillet_on_an_edge_to_a_four_edge_apex_matches_the_pinned_kernel() {
    // A lateral edge of a pyramid ends at an apex with four edges: the end of the fillet
    // is cut by the two other faces (PerformIntersectionAtEnd); the volume and area are
    // those of the pinned kernel (tools/oracle/tkfillet_probe_api.cxx, solid 1, edge 2).
    use rustycad_tkbrep::topexp;
    use rustycad_tkbrep::toptools::IndexedMapOfShape;
    use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeWedge;
    let mut w = BRepPrimAPIMakeWedge::with_limits(10., 10., 10., [5., 5., 5., 5.]).unwrap();
    let solid = w.shape().unwrap().clone();
    let mut edges = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(&solid, ShapeEnum::Edge, &mut edges).unwrap();
    let edge = topods::edge(edges.find_key(2).unwrap()).unwrap();
    let mut b = ChFi3dFilBuilder::new(&solid, FilletShape::Rational, 1.0e-2).unwrap();
    b.add_with_radius(0.5, &edge).unwrap();
    b.compute().unwrap();
    assert!(b.builder_base().is_done());
    let result = b.builder_base().shape().unwrap();
    let (v, a) = (volume(&result), area(&result));
    let native_v = f64::from_bits(4644564690667318038);
    let native_a = f64::from_bits(4644370541254930459);
    assert!((v - native_v).abs() <= 1.0e-12 * native_v, "volume {v} native {native_v}");
    assert!((a - native_a).abs() <= 1.0e-12 * native_a, "area {a} native {native_a}");
    assert!(BRepCheckAnalyzer::new(&result, true, false, false).unwrap().is_valid().unwrap());
}
