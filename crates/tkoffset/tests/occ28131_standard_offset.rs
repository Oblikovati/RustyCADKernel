//! Faithful port of `GeomFill_BSplineCurvesTest.OCC28131_StandardOffsetOfFilledFace`.
#[path = "../../tktopalgo/tests/support/occ28131.rs"]
mod fixture;

use rustycad_tkbrep::{brep::BRepTool, topexp::Explorer, topods::Shape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{geom::GeomSurface, top_abs::ShapeEnum};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkoffset::brep_offset::{BRepOffsetMakeOffset, BRepOffsetMode};
use rustycad_tkshhealing::shape_fix::ShapeFixShape;
use rustycad_tktopalgo::{
    brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape},
    brep_check::BRepCheckAnalyzer,
    brep_gprop::BRepGProp,
};

#[test]
fn geom_fill_bspline_curves_occ28131_standard_offset_of_filled_face() {
    let source = occ28131_face();
    assert!(!source.is_null());
    let mut maker = BRepOffsetMakeOffset::new();
    initialize_standard_maker(&mut maker, &source);
    maker.make_offset_shape().unwrap();
    assert!(maker.is_done());
    assert!(!maker.shape().is_null());
    assert_native_raw_result(maker.shape());
    assert_fixed_result(maker.shape());
}

fn initialize_standard_maker(maker: &mut BRepOffsetMakeOffset, source: &Shape) {
    maker.initialize(
        source,
        10.0,
        1.0e-7,
        BRepOffsetMode::Skin,
        false,
        false,
        JoinType::Arc,
        false,
        false,
    );
}

fn assert_native_raw_result(shape: &Shape) {
    assert_eq!(count(shape, ShapeEnum::Face), 2);
    assert!(!is_valid(shape));
    assert_near(
        surface_area(shape),
        1_695.591_173_430_126_2,
        1.0e-7,
        "raw area",
    );
}

fn assert_fixed_result(shape: &Shape) {
    let fixer = ShapeFixShape::new(shape.clone());
    fixer.perform().unwrap();
    let result = fixer.shape();
    assert!(is_valid(&result));
    assert_near(max_tolerance(&result), 0.408, 0.408 * 0.01, "max tolerance");
    assert_near(
        surface_area(&result),
        1693.76,
        1693.76 * 0.01,
        "surface area",
    );
}

fn occ28131_face() -> Shape {
    let surface = fixture::occ28131_surface().unwrap();
    let surface: Handle<dyn GeomSurface> = surface.cast();
    let mut maker = BRepBuilderAPIMakeFace::from_surface(&surface, 0.0).unwrap();
    maker.shape().unwrap().clone()
}

fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while explorer.more() {
        count += 1;
        explorer.next().unwrap();
    }
    count
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

fn is_valid(shape: &Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

fn assert_near(actual: f64, expected: f64, tolerance: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{label}={actual}, expected {expected} +/- {tolerance}"
    );
}
