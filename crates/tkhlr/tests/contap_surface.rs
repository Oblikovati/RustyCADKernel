//! Distinct normal conventions at singular and regular surfaces.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomPlane, GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkhlr::contap::ContapSurfProps;
use rustycad_tkmath::gp::{Axis3, Vector3};

fn adaptor(surface: Handle<dyn GeomSurface>) -> Adaptor3dSurfaceHandle {
    Handle::new(GeomAdaptorSurface::with_surface(&surface).unwrap()).cast()
}
#[test]
fn plane_has_constant_normal_and_zero_normal_derivatives() {
    let surface = adaptor(Handle::new(GeomPlane::new(Axis3::default())).cast());
    let (p, n, du, dv) = ContapSurfProps::norm_and_dn(&surface, 2.0, 3.0).unwrap();
    assert_eq!((p.x(), p.y(), p.z()), (2.0, 3.0, 0.0));
    assert_eq!(n, Vector3::new(0.0, 0.0, 1.0));
    assert_eq!(du, Vector3::default());
    assert_eq!(dv, Vector3::default());
    assert_eq!(
        ContapSurfProps::normale(&surface, 2.0, 3.0).unwrap(),
        (p, n)
    );
    assert_eq!(
        ContapSurfProps::deriv_and_norm(&surface, 2.0, 3.0)
            .unwrap()
            .3,
        n
    );
}
#[test]
fn indirect_plane_reverses_normal() {
    let mut frame = Axis3::default();
    frame.y_reverse();
    let surface = adaptor(Handle::new(GeomPlane::new(frame)).cast());
    assert_eq!(
        ContapSurfProps::normale(&surface, 0.0, 0.0).unwrap().1,
        Vector3::new(-0.0, -0.0, -1.0)
    );
}
#[test]
fn cone_apex_normale_is_zero_but_derivative_overloads_choose_nappe() {
    let surface =
        adaptor(Handle::new(GeomConicalSurface::new(Axis3::default(), 0.4, 0.0).unwrap()).cast());
    assert_eq!(
        ContapSurfProps::normale(&surface, 0.0, 0.0).unwrap().1,
        Vector3::default()
    );
    let normal = ContapSurfProps::deriv_and_norm(&surface, 0.0, 0.0)
        .unwrap()
        .3;
    assert!(normal.magnitude() > 0.99);
    assert_eq!(
        normal,
        ContapSurfProps::norm_and_dn(&surface, 0.0, 0.0).unwrap().1
    );
}
#[test]
fn sphere_normal_derivatives_are_scaled_surface_derivatives() {
    let surface =
        adaptor(Handle::new(GeomSphericalSurface::new(Axis3::default(), 2.0).unwrap()).cast());
    let (_, du, dv, normal) = ContapSurfProps::deriv_and_norm(&surface, 0.3, 0.2).unwrap();
    let (_, other, dnu, dnv) = ContapSurfProps::norm_and_dn(&surface, 0.3, 0.2).unwrap();
    assert_eq!(normal, other);
    assert_eq!(dnu, du / 2.0);
    assert_eq!(dnv, dv / 2.0);
}
#[test]
fn torus_uses_unnormalized_cross_product_branch() {
    let surface =
        adaptor(Handle::new(GeomToroidalSurface::new(Axis3::default(), 3.0, 0.5).unwrap()).cast());
    let (_, du, dv, n) = ContapSurfProps::deriv_and_norm(&surface, 0.0, 0.0).unwrap();
    assert_eq!(n, du.crossed(dv));
    assert_eq!(n.magnitude(), 1.75);
    assert_eq!(ContapSurfProps::normale(&surface, 0.0, 0.0).unwrap().1, n);
    assert_eq!(
        ContapSurfProps::norm_and_dn(&surface, 0.0, 0.0).unwrap().1,
        n
    );
}
#[test]
fn null_surface_reports_null_object_instead_of_native_dereference() {
    let surface = Handle::default();
    assert_eq!(
        ContapSurfProps::normale(&surface, 0.0, 0.0)
            .unwrap_err()
            .exception_type(),
        "Standard_NullObject"
    );
    assert!(ContapSurfProps::deriv_and_norm(&surface, 0.0, 0.0).is_err());
    assert!(ContapSurfProps::norm_and_dn(&surface, 0.0, 0.0).is_err());
}
