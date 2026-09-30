//! Original contour walking state and native not-done exceptions.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkhlr::contap::{ContapSurfFunction, ContapTheIWalking};
use rustycad_tkmath::gp::Axis3;
#[test]
fn unperformed_walking_has_no_accessible_results() {
    let walk = ContapTheIWalking::new(1e-7, 0.01, 0.1, true).unwrap();
    assert!(!walk.is_done());
    assert!(walk.nb_lines().is_err());
    assert!(walk.value(1).is_err());
    assert!(walk.nb_single_pnts().is_err());
    assert!(walk.single_pnt(1).is_err());
}
#[test]
fn empty_perform_produces_a_done_empty_result_for_both_overloads() {
    let geometry: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let surface =
        Handle::new(GeomAdaptorSurface::with_bounds(&geometry, (-1., 1., -1., 1.)).unwrap()).cast();
    let mut function = ContapSurfFunction::new();
    function.set_surface(&surface).unwrap();
    let mut walk = ContapTheIWalking::new(1e-7, 0.01, 0.1, true).unwrap();
    walk.set_tolerance(1e-6, 0.02, 0.2);
    walk.perform(&[], &[], &mut function, &surface, false)
        .unwrap();
    assert!(walk.is_done());
    assert_eq!(walk.nb_lines().unwrap(), 0);
    assert_eq!(walk.nb_single_pnts().unwrap(), 0);
    walk.perform_open(&[], &mut function, &surface, true)
        .unwrap();
    assert!(walk.is_done());
    assert_eq!(walk.nb_lines().unwrap(), 0);
    assert!(walk.value(1).is_err());
}
