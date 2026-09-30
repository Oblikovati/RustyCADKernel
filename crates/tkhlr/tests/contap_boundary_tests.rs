//! Boundary-search result states on a plane domain.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dTopolTool,
    geom::{GeomPlane, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkhlr::contap::{ContapArcFunction, ContapTheSearch};
use rustycad_tkmath::gp::{Axis3, Direction3};
#[test]
fn unset_boundary_search_rejects_all_result_queries() {
    let search = ContapTheSearch::new();
    assert!(!search.is_done());
    assert!(search.all_arc_solution().is_err());
    assert!(search.nb_points().is_err());
    assert!(search.point(1).is_err());
    assert!(search.nb_segments().is_err());
    assert!(search.segment(1).is_err());
}
#[test]
fn repeated_boundary_search_replaces_results_after_direction_change() {
    let geometry: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let surface =
        Handle::new(GeomAdaptorSurface::with_bounds(&geometry, (-1., 1., -1., 1.)).unwrap()).cast();
    let domain = Adaptor3dTopolTool::with_surface(&surface).unwrap();
    let mut function = ContapArcFunction::new();
    function.set_surface(&surface).unwrap();
    let mut search = ContapTheSearch::new();
    search
        .perform(&mut function, &domain, 1e-7, 1e-7, false)
        .unwrap();
    assert!(search.is_done());
    assert_eq!(search.nb_points().unwrap(), 0);
    assert_eq!(search.nb_segments().unwrap(), 0);
    assert!(!search.all_arc_solution().unwrap());
    function.set_direction(Direction3::X);
    search
        .perform(&mut function, &domain, 1e-7, 1e-7, false)
        .unwrap();
    assert!(search.all_arc_solution().unwrap());
    assert_eq!(search.nb_segments().unwrap(), 4);
}
