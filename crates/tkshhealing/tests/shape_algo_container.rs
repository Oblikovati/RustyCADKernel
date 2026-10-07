//! Behavior tests of the ShapeAlgo package (the native comparison is the sh_algo_geom and
//! sh_algo_shape requests of tools/tkshhealing_corpus_shape_algo.py).
mod healing_support;

use healing_support::{edges_of, polygon, square_face};
use rustycad_tkbrep::topods::Wire;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_algo::{ShapeAlgo, ShapeAlgoAlgoContainer, ShapeAlgoToolContainer};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisWire;
use rustycad_tkshhealing::shape_extend::ShapeExtendWireData;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;

fn open_polyline(points: &[(f64, f64, f64)]) -> Wire {
    let mut maker = BRepBuilderAPIMakePolygon::new();
    for &(x, y, z) in points {
        maker.add_point(Point3::new(x, y, z)).unwrap();
    }
    maker.wire().unwrap()
}

fn polyline_curve(points: &[(f64, f64, f64)]) -> Handle<GeomBSplineCurve> {
    let n = points.len() as i32;
    let mut poles = Array1::with_bounds(1, n).unwrap();
    let mut knots = Array1::with_bounds(1, n).unwrap();
    let mut mults = Array1::with_bounds(1, n).unwrap();
    for (i, &(x, y, z)) in points.iter().enumerate() {
        let index = i as i32 + 1;
        poles.set_value(index, Point3::new(x, y, z)).unwrap();
        knots.set_value(index, i as f64).unwrap();
        mults
            .set_value(index, if index == 1 || index == n { 2 } else { 1 })
            .unwrap();
    }
    Handle::new(GeomBSplineCurve::new(&poles, &knots, &mults, 1, false).unwrap())
}

#[test]
fn package_container_is_set_by_init() {
    ShapeAlgo::init().unwrap();
    let current = ShapeAlgo::algo_container();
    assert!(!current.is_null());
    let other = Handle::new(ShapeAlgoAlgoContainer::new());
    ShapeAlgo::set_algo_container(&other);
    assert!(ShapeAlgo::algo_container() == other);
    ShapeAlgo::set_algo_container(&current);
}

#[test]
fn tool_container_is_replaceable() {
    let container = ShapeAlgoAlgoContainer::new();
    let tools = Handle::new(ShapeAlgoToolContainer::new());
    container.set_tool_container(&tools);
    assert!(container.tool_container() == tools);
    assert!(!tools.fix_shape().is_null());
}

#[test]
fn approx_drops_the_poles_after_the_last_coincident_pair() {
    let curve = polyline_curve(&[
        (0., 0., 0.),
        (1., 1., 0.),
        (2., 0., 0.),
        (2., 0., 0.),
        (3., 1., 0.),
        (4., 0., 0.),
    ]);
    let pieces = ShapeAlgoAlgoContainer::new()
        .approx_bspline_curve(&curve)
        .unwrap();
    assert_eq!(pieces.len(), 1);
}

#[test]
fn approx_keeps_a_curve_without_coincident_poles_whole() {
    let curve = polyline_curve(&[(0., 0., 0.), (1., 1., 0.), (2., 0., 0.)]);
    let pieces = ShapeAlgoAlgoContainer::new()
        .approx_bspline_curve(&curve)
        .unwrap();
    assert_eq!(pieces.len(), 1);
}

#[test]
fn connect_next_wire_reverses_the_next_wire_tail_to_tail() {
    let sewd = Handle::new(
        ShapeExtendWireData::from_wire(&open_polyline(&[(0., 0., 0.), (1., 0., 0.)]), true, true)
            .unwrap(),
    );
    let next = Handle::new(
        ShapeExtendWireData::from_wire(&open_polyline(&[(3., 0., 0.), (1., 0., 0.)]), true, true)
            .unwrap(),
    );
    let saw = Handle::new(ShapeAnalysisWire::new());
    saw.load_wire_data(&sewd);
    let connection = ShapeAlgoAlgoContainer::new()
        .connect_next_wire(&saw, &next, 1e-3)
        .unwrap();
    assert!(connection.connected);
    assert!(connection.reversed_next && !connection.reversed_wire);
    assert_eq!(connection.distance, 0.);
    assert_eq!(sewd.nb_edges(), 2);
}

#[test]
fn homo_wires_gives_wires_of_equal_edge_counts() {
    let triangle = polygon(&[(0., 0., 0.), (2., 0., 0.), (0., 1., 0.)]);
    let square = polygon(&[(0., 0., 1.), (1., 0., 1.), (1., 1., 1.), (0., 1., 1.)]);
    let (first, second) = ShapeAlgoAlgoContainer::new()
        .homo_wires(&triangle, &square, false)
        .unwrap()
        .expect("the native loop ends on the last edge");
    let counts = (edges_of(&first).len(), edges_of(&second).len());
    assert_eq!(counts.0, counts.1);
}

#[test]
fn face_queries_delegate_to_shape_analysis() {
    let face = square_face(0.);
    let container = ShapeAlgoAlgoContainer::new();
    assert_eq!(edges_of(&container.outer_wire(&face).unwrap()).len(), 4);
    let (umin, umax, vmin, vmax) = container.get_face_uv_bounds(&face).unwrap();
    assert!(umax > umin && vmax > vmin);
}
