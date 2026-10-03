//! Exact Rust port of `GeomFill_BSplineCurvesTest.OCC28131_SimpleOffsetOfFilledFace`.
#[path = "../../tktopalgo/tests/support/occ28131.rs"]
mod fixture;

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIMakeOffsetShape;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

#[test]
fn geom_fill_bspline_curves_occ28131_simple_offset_of_filled_face() {
    let face = occ28131_face();
    assert!(!face.is_null());
    let mut maker = BRepOffsetAPIMakeOffsetShape::new();
    maker.perform_by_simple(&face, 10.0).unwrap();
    assert!(maker.is_done());
    let result = maker.shape().unwrap().clone();
    assert!(!result.is_null());
    assert!(
        BRepCheckAnalyzer::new(&result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    assert_near(max_tolerance(&result), 0.205, 0.205 * 0.01, "tolerance");
    assert_near(surface_area(&result), 1693.7, 1693.7 * 0.01, "surface area");
}

fn occ28131_face() -> Shape {
    let source = fixture::occ28131_surface().unwrap();
    let surface: Handle<dyn GeomSurface> = source.cast();
    let mut maker = BRepBuilderAPIMakeFace::from_surface(&surface, 0.0).unwrap();
    maker.shape().unwrap().clone()
}

fn max_tolerance(shape: &Shape) -> f64 {
    [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face]
        .into_iter()
        .map(|kind| BRepTool::max_tolerance(shape, kind).unwrap())
        .fold(0.0_f64, f64::max)
}

fn surface_area(shape: &Shape) -> f64 {
    let mut properties = GProps::new();
    BRepGProp::surface_properties(shape, &mut properties, false, false).unwrap();
    properties.mass()
}

fn assert_near(actual: f64, expected: f64, tolerance: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{label}={actual}, expected {expected} +/- {tolerance}"
    );
}
