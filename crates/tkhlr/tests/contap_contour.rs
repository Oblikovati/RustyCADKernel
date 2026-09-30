//! Original Contap_Contour state rules, overloads and retained mutable function.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dTopolTool, h_surface_tool::Adaptor3dSurfaceHandle},
    geom::{GeomPlane, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkhlr::contap::ContapContour;
use rustycad_tkmath::gp::{Axis3, Point3, Vector3};
fn plane() -> (Adaptor3dSurfaceHandle, Adaptor3dTopolTool) {
    let geometry: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let surface: Adaptor3dSurfaceHandle =
        Handle::new(GeomAdaptorSurface::with_bounds(&geometry, (-1., 1., -1., 1.)).unwrap()).cast();
    let domain = Adaptor3dTopolTool::with_surface(&surface).unwrap();
    (surface, domain)
}
fn assert_unfinished(contour: &mut ContapContour) {
    assert!(!contour.is_done());
    assert!(contour.is_empty().is_err());
    assert!(contour.nb_lines().is_err());
    assert!(contour.line(1).is_err());
    assert!(contour.surface_function().is_err());
}
#[test]
fn default_contour_requires_view_and_success_before_result_access() {
    let (surface, domain) = plane();
    let mut contour = ContapContour::default();
    assert_unfinished(&mut contour);
    assert!(contour.perform(&surface, &domain).is_err());
    assert_unfinished(&mut contour);
}
#[test]
fn initializing_each_view_invalidates_results() {
    let (surface, domain) = plane();
    let direction = Vector3::new(0., 0., 1.);
    let mut contour = ContapContour::with_surface_direction(&surface, &domain, direction).unwrap();
    assert!(contour.is_empty().unwrap());
    contour.init_direction(direction).unwrap();
    assert_unfinished(&mut contour);
    contour.perform(&surface, &domain).unwrap();
    contour.init_direction_angle(direction, 0.2).unwrap();
    assert_unfinished(&mut contour);
    contour.perform(&surface, &domain).unwrap();
    contour.init_eye(Point3::new(0., 0., 3.));
    assert_unfinished(&mut contour);
}
#[test]
fn constructors_and_perform_overloads_share_native_state() {
    let (surface, domain) = plane();
    let direction = Vector3::new(0., 0., 1.);
    let eye = Point3::new(0., 0., 3.);
    let mut contours = [
        ContapContour::with_direction(direction).unwrap(),
        ContapContour::with_direction_angle(direction, 0.2).unwrap(),
        ContapContour::with_eye(eye),
    ];
    for contour in &mut contours {
        assert_unfinished(contour);
        contour.perform(&surface, &domain).unwrap();
        assert!(contour.is_empty().unwrap());
    }
    let mut contour =
        ContapContour::with_surface_direction_angle(&surface, &domain, direction, 0.2).unwrap();
    assert!(contour.is_empty().unwrap());
    contour = ContapContour::with_surface_eye(&surface, &domain, eye).unwrap();
    assert!(contour.is_empty().unwrap());
    contour
        .perform_direction(&surface, &domain, direction)
        .unwrap();
    assert!(contour.is_empty().unwrap());
    contour
        .perform_direction_angle(&surface, &domain, direction, 0.2)
        .unwrap();
    assert!(contour.is_empty().unwrap());
    contour.perform_eye(&surface, &domain, eye).unwrap();
    assert!(contour.is_empty().unwrap());
}
#[test]
fn zero_direction_invalidates_before_normalization_failure() {
    let (surface, domain) = plane();
    let mut contour =
        ContapContour::with_surface_direction(&surface, &domain, Vector3::new(0., 0., 1.)).unwrap();
    assert!(contour.init_direction(Vector3::new(0., 0., 0.)).is_err());
    assert_unfinished(&mut contour);
    contour.perform(&surface, &domain).unwrap();
    assert!(contour.is_empty().unwrap());
    assert!(
        contour
            .init_direction_angle(Vector3::new(0., 0., 0.), 0.2)
            .is_err()
    );
    assert_unfinished(&mut contour);
    contour.perform(&surface, &domain).unwrap();
    assert!(contour.is_empty().unwrap());
}
#[test]
fn completed_surface_function_is_mutable_and_line_indices_are_one_based() {
    let (surface, domain) = plane();
    let mut contour =
        ContapContour::with_surface_direction(&surface, &domain, Vector3::new(1., 0., 0.)).unwrap();
    assert!(contour.nb_lines().unwrap() > 0);
    assert!(contour.line(1).is_ok());
    assert!(contour.line(0).is_err());
    assert!(contour.line(contour.nb_lines().unwrap() + 1).is_err());
    contour
        .surface_function()
        .unwrap()
        .set_eye(Point3::new(0., 0., 3.));
    contour.perform(&surface, &domain).unwrap();
    assert!(contour.is_empty().unwrap());
}
