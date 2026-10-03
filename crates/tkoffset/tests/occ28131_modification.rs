//! Prerequisite exercise of OCC28131 through the modification layer, not a legacy mapping.
#[path = "../../tktopalgo/tests/support/occ28131.rs"]
mod fixture;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkoffset::brep_offset::BRepOffsetSimpleOffset;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

#[test]
fn occ28131_modification_geometry_matches_offset_expectations() {
    let result = occ28131_offset_modification();
    assert!(!result.is_null());
    assert!(
        BRepCheckAnalyzer::new(&result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    let max_tolerance = [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face]
        .into_iter()
        .map(|kind| BRepTool::max_tolerance(&result, kind).unwrap())
        .fold(0.0_f64, f64::max);
    assert!(
        (max_tolerance - 0.205).abs() <= 0.205 * 0.01,
        "tolerance={max_tolerance}"
    );
    let mut properties = GProps::new();
    BRepGProp::surface_properties(&result, &mut properties, false, false).unwrap();
    assert!(
        (properties.mass() - 1693.7).abs() <= 1693.7 * 0.01,
        "area={}",
        properties.mass()
    );
}

fn occ28131_offset_modification() -> Shape {
    let surface: Handle<dyn GeomSurface> = fixture::occ28131_surface().unwrap().cast();
    let mut maker = BRepBuilderAPIMakeFace::from_surface(&surface, 0.0).unwrap();
    let shape = maker.shape().unwrap().clone();
    let modification: Handle<dyn BRepToolsModification> =
        Handle::new(BRepOffsetSimpleOffset::new(&shape, 10.0, 1e-7).unwrap()).cast();
    let modifier = Modifier::with_modification(&shape, &modification).unwrap();
    assert!(modifier.is_done());
    let result = modifier.modified_shape(&shape).unwrap();
    result
}
