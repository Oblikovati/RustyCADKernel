//! Native replay of BOPAlgo_Tools::EdgesToWires and WiresToFaces; see
//! tools/oracle/tkbo_edges_to_wires.cxx.
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkbo::bop_algo::tools::{edges_to_wires, wires_to_faces};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{self, Axis2, Circle3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};
use shape_dump::dump_result;
use std::fmt::Write;

fn compound(shapes: &[Shape]) -> Shape {
    let builder = BRepBuilder::default();
    let mut result = Compound::default();
    builder.make_compound(&mut result);
    for shape in shapes {
        builder.add(&mut result, shape).unwrap();
    }
    result.into_shape()
}

fn edges_of(shape: &Shape) -> Shape {
    let mut explorer = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let mut edges = Vec::new();
    while explorer.more() {
        edges.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    compound(&edges)
}

fn run(label: &str, edges: &Shape, shared: bool, rows: &mut String) {
    let mut wires = Shape::default();
    let error = edges_to_wires(edges, &mut wires, shared, 1.0e-8).unwrap();
    writeln!(
        rows,
        "case {label} shared {} error {error} null {}",
        i32::from(shared),
        i32::from(wires.is_null())
    )
    .unwrap();
    if wires.is_null() {
        return;
    }
    for (index, wire) in ShapeIterator::new(&wires, true, true).enumerate() {
        let count = ShapeIterator::new(&wire, true, true).count();
        writeln!(rows, "wire {index} edges {count}").unwrap();
    }
    dump_result(&wires, rows);
    let mut faces = Shape::default();
    let built = wires_to_faces(&wires, &mut faces, 1.0e-8).unwrap();
    writeln!(rows, "faces built {}", i32::from(built)).unwrap();
    dump_result(&faces, rows);
}

fn segment(first: Point3, second: Point3) -> Shape {
    BRepBuilderAPIMakeEdge::from_points(first, second)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

fn circle(center: Point3, radius: f64) -> Shape {
    let frame = Axis2::new(center, gp::dz()).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(frame, radius).unwrap()).unwrap();
    edge.edge().unwrap().into_shape()
}

fn array<T: Clone + Default>(values: &[T]) -> Array1<'static, T> {
    let mut array = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (index, value) in values.iter().enumerate() {
        array.set_value(index as i32 + 1, value.clone()).unwrap();
    }
    array
}

fn spline_arc() -> Shape {
    let poles =
        [(0.0, 0.0), (1.0, 2.0), (3.0, 2.0), (4.0, 0.0)].map(|(x, y)| Point3::new(x, y, 0.0));
    let curve = GeomBSplineCurve::new(
        &array(&poles),
        &array(&[0.0, 1.0]),
        &array(&[4, 4]),
        3,
        false,
    )
    .unwrap();
    let curve: Handle<dyn GeomCurve> = Handle::new(curve).into_cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

#[test]
fn edges_to_wires_and_faces_match_native() {
    let mut rows = String::new();
    let box_shape = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    run("box", &edges_of(&box_shape), true, &mut rows);
    run("box-intersected", &edges_of(&box_shape), false, &mut rows);
    let cylinder = BRepPrimAPIMakeCylinder::new(5.0, 10.0)
        .shape()
        .unwrap()
        .clone();
    run("cylinder", &edges_of(&cylinder), true, &mut rows);
    let mixed = compound(&[
        circle(Point3::new(0.0, 0.0, 0.0), 3.0),
        circle(Point3::new(0.0, 0.0, 5.0), 2.0),
        segment(Point3::new(-5.0, 0.0, 0.0), Point3::new(5.0, 0.0, 0.0)),
        segment(Point3::new(0.0, -5.0, 5.0), Point3::new(0.0, 5.0, 5.0)),
    ]);
    run("circles-lines", &mixed, false, &mut rows);
    let chord = segment(Point3::new(0.0, 0.0, 0.0), Point3::new(4.0, 0.0, 0.0));
    run(
        "spline-chord",
        &compound(&[spline_arc(), chord]),
        false,
        &mut rows,
    );
    run(
        "single",
        &segment(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)),
        true,
        &mut rows,
    );
    run("empty", &compound(&[]), true, &mut rows);
    let expected = include_str!("../../../tools/oracle/tkbo_edges_to_wires.expected");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "line {}", line + 1);
    }
    assert_eq!(rows.lines().count(), expected.lines().count());
}
