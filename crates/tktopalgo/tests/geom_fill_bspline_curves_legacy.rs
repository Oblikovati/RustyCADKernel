//! Source-faithful first OCC28131 fill regression from GeomFill_BSplineCurves_Test.cxx.
#[path = "support/occ28131.rs"]
mod fixture;
use fixture::occ28131_surface;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

#[test]
fn geom_fill_bspline_curves_occ28131_fill_surface_from_bezier_and_interpolated_curves() {
    let surface = occ28131_surface().unwrap();
    let geometry: Handle<dyn GeomSurface> = surface.cast();
    let mut maker = BRepBuilderAPIMakeFace::from_surface(&geometry, 0.0).unwrap();
    let shape = if maker.is_done() {
        maker.shape().unwrap().clone()
    } else {
        rustycad_tkbrep::topods::Shape::default()
    };
    assert!(!shape.is_null());
    assert!(
        BRepCheckAnalyzer::new(&shape, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}
