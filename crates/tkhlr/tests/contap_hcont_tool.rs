//! Native process-wide contour sample range and arc contracts.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle,
    geom::{GeomPlane, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkhlr::contap::ContapHContTool as Tool;
use rustycad_tkmath::gp::Axis3;
fn plane(bounds: (f64, f64, f64, f64)) -> Adaptor3dSurfaceHandle {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    Handle::new(GeomAdaptorSurface::with_bounds(&surface, bounds).unwrap()).cast()
}
#[test]
fn sampling_reuses_the_last_counted_surface_range() {
    let first = plane((1.0, 9.0, 2.0, 18.0));
    let second = plane((0.0, 4.0, 0.0, 8.0));
    assert_eq!(Tool::nb_samples_u(&first, -1.0, 2.0).unwrap(), 2);
    assert_eq!(Tool::nb_samples_v(&first, -1.0, 2.0).unwrap(), 2);
    assert_eq!(Tool::nb_sample_points(&first).unwrap(), 5);
    assert_eq!(Tool::sample_point(&second, 1).unwrap(), (3.0, 6.0));
    assert_eq!(Tool::sample_point(&second, 4).unwrap(), (7.0, 14.0));
    assert_eq!(Tool::sample_point(&second, 99).unwrap(), (5.0, 10.0));
    assert_eq!(Tool::nb_sample_points(&second).unwrap(), 5);
    assert_eq!(Tool::sample_point(&first, 1).unwrap(), (1.0, 2.0));
}
#[test]
fn null_surface_reports_failure_without_dereferencing() {
    assert!(Tool::nb_samples_u(&Handle::default(), 0.0, 1.0).is_err());
    assert!(Tool::nb_samples_v(&Handle::default(), 0.0, 1.0).is_err());
    assert!(Tool::nb_sample_points(&Handle::default()).is_err());
    assert!(Tool::sample_point(&Handle::default(), 1).is_err());
}
#[test]
fn restriction_search_empty_results_match_native_constants() {
    let arc = Handle::default();
    assert!(!Tool::has_been_seen(&arc));
    assert_eq!(Tool::nb_points(&arc), 0);
    assert!(!Tool::is_vertex(&arc, 1));
    assert_eq!(Tool::nb_segments(&arc), 0);
    assert!(!Tool::is_all_solution(&arc));
    assert!(Tool::value(&arc, 1).is_err());
    assert!(Tool::vertex(&arc, 1).is_err());
    assert!(Tool::has_first_point(&arc, 1).is_err());
    assert!(Tool::has_last_point(&arc, 1).is_err());
}
