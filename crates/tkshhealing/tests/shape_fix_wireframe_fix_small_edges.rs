//! Native FixSmallEdges orchestration for empty repair sets.
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Builder, Compound, Shape, ShapeIterator, Wire},
    toptools::MapOfShape,
};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixWireframe, ShapeFixWireframeShapeLists},
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon};

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

fn tiny_wire() -> Wire {
    BRepBuilderAPIMakePolygon::from_four_points(
        Point3::new(0., 0., 0.),
        Point3::new(0.01, 0., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(1., 1., 0.),
        true,
    )
    .unwrap()
    .wire()
    .unwrap()
}

fn two_tiny_wire() -> Wire {
    let mut maker = BRepBuilderAPIMakePolygon::new();
    for point in [
        Point3::new(0., 0., 0.),
        Point3::new(0.01, 0., 0.),
        Point3::new(0.02, 0., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(1., 1., 0.),
    ] {
        maker.add_point(point).unwrap();
    }
    maker.close().unwrap();
    maker.wire().unwrap()
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
    let tiny = tiny_wire();
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

#[test]
fn one_face_pcurve_join_matches_native_edge_and_pcurve_counts() {
    let face = BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &tiny_wire(), true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape();
    let fixer = ShapeFixWireframe::from_shape(&face);
    fixer.set_precision(0.1);
    let changed = fixer.fix_small_edges().unwrap();
    let result = fixer.shape();
    let result_face = topods::face(&result).unwrap();
    let (edges, curves) = face_edge_curves(&result_face);
    let pcurves = curves.len();
    assert_eq!(
        vec![
            f64::from(changed),
            f64::from(fixer.status_small_edges(ShapeExtendStatus::Done)),
            edges as f64,
            Explorer::new(&result, ShapeEnum::Vertex, ShapeEnum::Shape)
                .unwrap()
                .count() as f64,
            pcurves as f64,
        ],
        expected("face_small")
    );
    assert_eq!(curves, expected_rows("face_pcurve"));
}

#[test]
fn adjacent_small_edges_join_into_one_native_face_curve() {
    let face = BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &two_tiny_wire(), true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape();
    let fixer = ShapeFixWireframe::from_shape(&face);
    fixer.set_precision(0.1);
    let changed = fixer.fix_small_edges().unwrap();
    let result = fixer.shape();
    let result_face = topods::face(&result).unwrap();
    let (edges, curves) = face_edge_curves(&result_face);
    assert_eq!(
        vec![
            f64::from(changed),
            f64::from(fixer.status_small_edges(ShapeExtendStatus::Done)),
            edges as f64,
            Explorer::new(&result, ShapeEnum::Vertex, ShapeEnum::Shape)
                .unwrap()
                .count() as f64,
            curves.len() as f64,
        ],
        expected("face_pair")
    );
    assert_eq!(curves, expected_rows("face_pair_pcurve"));
}

fn face_edge_curves(face: &rustycad_tkbrep::topods::Face) -> (usize, Vec<Vec<f64>>) {
    let mut edges = 0;
    let mut pcurves = Vec::new();
    for edge in Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .unwrap()
    {
        edges += 1;
        let curve = BRepTool::curve_on_face(&topods::edge(&edge).unwrap(), face).unwrap();
        if !curve.pcurve.is_null() {
            let start = curve.pcurve.value(curve.first).unwrap();
            let end = curve.pcurve.value(curve.last).unwrap();
            pcurves.push(vec![
                curve.first,
                curve.last,
                start.x(),
                start.y(),
                end.x(),
                end.y(),
            ]);
        }
    }
    (edges, pcurves)
}

fn expected_rows(name: &str) -> Vec<Vec<f64>> {
    include_str!("../../../tools/oracle/tkshhealing_wireframe_fix_small_edges.expected")
        .lines()
        .filter(|line| line.starts_with(&format!("{name} ")))
        .map(|line| {
            line.split_whitespace()
                .skip(1)
                .map(|value| value.parse().unwrap())
                .collect()
        })
        .collect()
}
