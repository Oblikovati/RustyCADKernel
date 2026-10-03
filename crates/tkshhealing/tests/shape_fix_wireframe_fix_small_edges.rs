//! Native FixSmallEdges orchestration for empty repair sets.
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Builder, Compound, Shape, ShapeIterator, Wire},
    toptools::MapOfShape,
};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixWireframe, ShapeFixWireframeShapeLists},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;

fn unit_wire() -> Wire {
    BRepBuilderAPIMakePolygon::from_four_points(
        Point3::new(0., 0., 0.),
        Point3::new(2., 0., 0.),
        Point3::new(2., 2., 0.),
        Point3::new(0., 2., 0.),
        true,
    )
    .unwrap()
    .wire()
    .unwrap()
}

fn report(name: &str, tool: &ShapeFixWireframe, changed: bool) -> Vec<f64> {
    let shape = tool.shape();
    let edges = Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .count();
    let row = vec![
        f64::from(changed),
        f64::from(shape.is_null()),
        f64::from(tool.context().is_null()),
        edges as f64,
        f64::from(tool.status_small_edges(ShapeExtendStatus::Ok)),
        f64::from(tool.status_small_edges(ShapeExtendStatus::Done)),
    ];
    assert_eq!(row, expected(name), "{name}");
    row
}

fn expected(name: &str) -> Vec<f64> {
    include_str!("../../../tools/oracle/tkshhealing_wireframe_fix_small_edges.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect()
}

fn run_loaded(name: &str, shape: &Shape) -> Shape {
    let tool = ShapeFixWireframe::from_shape(shape);
    tool.set_precision(0.1);
    report(name, &tool, tool.fix_small_edges().unwrap());
    tool.shape()
}

#[test]
fn no_small_edges_preserves_shape_and_reports_native_status() {
    let tool = ShapeFixWireframe::new();
    report("null", &tool, tool.fix_small_edges().unwrap());
    let wire = unit_wire();
    run_loaded("wire", &wire.clone().into_shape());
    let tiny = BRepBuilderAPIMakePolygon::from_four_points(
        Point3::new(0., 0., 0.),
        Point3::new(0.01, 0., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(1., 1., 0.),
        true,
    )
    .unwrap()
    .wire()
    .unwrap();
    let result = run_loaded("small_wire", &tiny.into_shape());
    assert_eq!(topology_counts(&result), expected("small_topology"));
    assert_joined_bspline(&result);
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    Builder.add(&mut compound, &wire).unwrap();
    Builder.add(&mut compound, &wire).unwrap();
    run_loaded("compound", &compound.into_shape());
}

fn assert_joined_bspline(shape: &Shape) {
    let expected = expected("small_bspline");
    let mut observed = None;
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        let edge = topods::edge(&edge).unwrap();
        let (curve, first, last) = BRepTool::curve(&edge).unwrap();
        let spline_handle = curve.cast::<GeomBSplineCurve>();
        let Some(spline) = spline_handle.get() else {
            continue;
        };
        let start = spline.value(first).unwrap();
        let end = spline.value(last).unwrap();
        observed = Some(vec![
            first,
            last,
            f64::from(spline.degree()),
            f64::from(spline.nb_poles()),
            start.x(),
            start.y(),
            end.x(),
            end.y(),
        ]);
        break;
    }
    assert_eq!(observed.unwrap(), expected);
}

fn topology_counts(shape: &Shape) -> Vec<f64> {
    let edge_shapes: Vec<Shape> = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .collect();
    let vertex_occurrences = edge_shapes
        .iter()
        .map(|edge| ShapeIterator::new(edge, true, true).count())
        .sum::<usize>();
    vec![edge_shapes.len() as f64, vertex_occurrences as f64]
}

#[test]
fn empty_merge_map_returns_native_false() {
    let tool = ShapeFixWireframe::new();
    let mut small = MapOfShape::default();
    let mut edge_faces = ShapeFixWireframeShapeLists::default();
    let mut face_small = ShapeFixWireframeShapeLists::default();
    let mut multiple = MapOfShape::default();
    let changed = tool
        .merge_small_edges(
            &mut small,
            &mut edge_faces,
            &mut face_small,
            &mut multiple,
            false,
            -1.0,
        )
        .unwrap();
    assert_eq!(vec![f64::from(changed)], expected("merge_empty"));
}
