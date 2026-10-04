use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, face};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis3, Direction2, Point2};
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeWire,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_lib::build_curve3d;

const NATIVE_GLOBAL: &str =
    include_str!("../../../../tools/oracle/tkshhealing_unify_global_context.expected");

#[test]
fn concat_bspline_edges_also_concatenates_general_surface_pcurves() {
    let input = split_bezier_cylinder_face();
    assert!(
        BRepCheckAnalyzer::new(&input, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    assert!(
        NATIVE_GLOBAL
            .lines()
            .any(|line| { line == "cylinder_bezier_patch_input_valid 1" })
    );
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, true);

    unifier.build().unwrap();

    assert_unified_curve_count(&unifier.shape());
    assert_unified_shape_valid(&unifier.shape());
    assert_concatenated_pcurve(&unifier.shape());
}

fn assert_unified_shape_valid(shape: &Shape) {
    assert!(
        BRepCheckAnalyzer::new(shape, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    assert!(
        NATIVE_GLOBAL
            .lines()
            .any(|line| line == "cylinder_bezier_unified_valid 1")
    );
}

fn assert_unified_curve_count(shape: &Shape) {
    let count = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .count();
    assert_eq!(count, 4);
    assert!(
        NATIVE_GLOBAL
            .lines()
            .any(|line| line == "cylinder_bezier_unified_edges 4")
    );
}

fn assert_concatenated_pcurve(shape: &Shape) {
    let face = face(
        &Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
            .unwrap()
            .current()
            .clone(),
    )
    .unwrap();
    let edge = find_spline_edge(shape).unwrap();
    let pcurve = BRepTool::curve_on_face(&edge, &face).unwrap();
    assert_eq!((pcurve.first, pcurve.last), (0.0, 2.0));
    let adaptor = BRepAdaptorCurve2d::with_edge_on_face(&edge, &face).unwrap();
    assert_eq!(adaptor.get_type().unwrap(), CurveType::BSplineCurve);
    assert_eq!(adaptor.value(1.0).unwrap(), Point2::new(0.0, 2.0));
    assert!(
        NATIVE_GLOBAL
            .lines()
            .any(|line| { line == "cylinder_bezier_pcurve 6 0 2 0 2" })
    );
}

fn find_spline_edge(shape: &Shape) -> Option<rustycad_tkbrep::topods::Edge> {
    let edges = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .ok()?
        .map(|candidate| rustycad_tkbrep::topods::edge(&candidate))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let face = test_face(shape)?;
    edges
        .into_iter()
        .find(|edge| is_spline_pcurve_edge(edge, &face))
}

fn is_spline_pcurve_edge(
    edge: &rustycad_tkbrep::topods::Edge,
    face: &rustycad_tkbrep::topods::Face,
) -> bool {
    let is_spline_3d = BRepAdaptorCurve::with_edge(edge)
        .and_then(|adaptor| adaptor.get_type())
        .is_ok_and(|kind| kind == CurveType::BSplineCurve);
    let is_spline_2d = BRepAdaptorCurve2d::with_edge_on_face(edge, face)
        .and_then(|adaptor| adaptor.get_type())
        .is_ok_and(|kind| kind == CurveType::BSplineCurve);
    is_spline_3d && is_spline_2d
}

fn test_face(shape: &Shape) -> Option<rustycad_tkbrep::topods::Face> {
    let candidate = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .ok()?
        .current()
        .clone();
    face(&candidate).ok()
}

fn split_bezier_cylinder_face() -> Shape {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.0).unwrap()).cast();
    let curves = bezier_split_edges(&surface);
    let mut wire = BRepBuilderAPIMakeWire::new();
    for curve in curves {
        wire.add_edge(&curve).unwrap();
    }
    BRepBuilderAPIMakeFace::from_surface_wire(&surface, &wire.wire().unwrap(), true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn bezier_split_edges(surface: &Handle<dyn GeomSurface>) -> Vec<rustycad_tkbrep::topods::Edge> {
    [
        bezier_segment(surface, 0.0, 0.5, 2.0),
        bezier_segment(surface, 2.0, 3.5, 4.0),
    ]
    .into_iter()
    .chain(patch_closure_edges(surface))
    .collect()
}

fn bezier_segment(
    surface: &Handle<dyn GeomSurface>,
    start: f64,
    control: f64,
    end: f64,
) -> rustycad_tkbrep::topods::Edge {
    surface_bezier_edge(
        surface,
        [
            Point2::new(0., start),
            Point2::new(0., control),
            Point2::new(0., end),
        ],
    )
}

fn patch_closure_edges(surface: &Handle<dyn GeomSurface>) -> [rustycad_tkbrep::topods::Edge; 3] {
    [top_edge(surface), right_edge(surface), bottom_edge(surface)]
}

fn top_edge(surface: &Handle<dyn GeomSurface>) -> rustycad_tkbrep::topods::Edge {
    surface_line_edge(
        surface,
        Point2::new(0., 4.),
        Direction2::new(1., 0.).unwrap(),
        1.,
    )
}

fn right_edge(surface: &Handle<dyn GeomSurface>) -> rustycad_tkbrep::topods::Edge {
    surface_line_edge(
        surface,
        Point2::new(1., 4.),
        Direction2::new(0., -1.).unwrap(),
        4.,
    )
}

fn bottom_edge(surface: &Handle<dyn GeomSurface>) -> rustycad_tkbrep::topods::Edge {
    surface_line_edge(
        surface,
        Point2::new(1., 0.),
        Direction2::new(-1., 0.).unwrap(),
        1.,
    )
}

fn surface_bezier_edge(
    surface: &Handle<dyn GeomSurface>,
    poles: [Point2; 3],
) -> rustycad_tkbrep::topods::Edge {
    let mut values = Array1::with_size(3);
    for (index, pole) in poles.into_iter().enumerate() {
        values.set_value(index as i32, pole).unwrap();
    }
    let curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBezierCurve::new(&values).unwrap()).cast();
    make_surface_edge(surface, &curve, 1.0)
}

fn surface_line_edge(
    surface: &Handle<dyn GeomSurface>,
    origin: Point2,
    direction: Direction2,
    last: f64,
) -> rustycad_tkbrep::topods::Edge {
    let curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(origin, direction)).cast();
    make_surface_edge(surface, &curve, last)
}

fn make_surface_edge(
    surface: &Handle<dyn GeomSurface>,
    curve: &Handle<dyn Geom2dCurve>,
    last: f64,
) -> rustycad_tkbrep::topods::Edge {
    let edge = BRepBuilderAPIMakeEdge::from_pcurve_params(curve, surface, 0.0, last)
        .unwrap()
        .edge()
        .unwrap();
    build_curve3d(&edge, 1.0e-7, rustycad_tkmath::geom_abs::Shape::C1, 14, 0).unwrap();
    edge
}
