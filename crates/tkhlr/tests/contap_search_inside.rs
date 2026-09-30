//! Interior search state transitions over the original contour function.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle,
    geom::{GeomPlane, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkhlr::contap::{ContapSurfFunction, ContapTheSearchInside};
use rustycad_tkmath::gp::Axis3;
fn plane() -> Adaptor3dSurfaceHandle {
    let geometry: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    Handle::new(GeomAdaptorSurface::with_bounds(&geometry, (-1., 1., -1., 1.)).unwrap()).cast()
}
#[test]
fn unperformed_search_reports_not_done() {
    let search = ContapTheSearchInside::new();
    assert!(!search.is_done());
    assert!(search.nb_points().is_err());
    assert!(search.value(1).is_err());
}
#[test]
fn plane_without_a_silhouette_has_no_seed() {
    let surface = plane();
    let mut function = ContapSurfFunction::new();
    function.set_surface(&surface).unwrap();
    let mut search = ContapTheSearchInside::new();
    search
        .perform_from(&mut function, &surface, 0., 0.)
        .unwrap();
    assert!(search.is_done());
    assert_eq!(search.nb_points().unwrap(), 0);
    assert!(search.value(1).is_err());
    search
        .perform_from(&mut function, &surface, 2., 2.)
        .unwrap();
    assert_eq!(search.nb_points().unwrap(), 0);
}
