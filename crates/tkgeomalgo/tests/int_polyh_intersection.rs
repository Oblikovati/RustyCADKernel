//! Direct translation of IntPolyh_Intersection_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_polyh::IntPolyhIntersection;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};

fn ax3(p: (f64, f64, f64), d: (f64, f64, f64)) -> Axis3 {
    Axis3::from_normal(
        Point3::new(p.0, p.1, p.2),
        Direction3::new(d.0, d.1, d.2).unwrap(),
    )
    .unwrap()
}

fn adaptor(surface: Handle<dyn GeomSurface>) -> Adaptor3dSurfaceHandle {
    Handle::new(GeomAdaptorSurface::with_surface(&surface).unwrap()).cast()
}

fn bounded(
    surface: Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
) -> Adaptor3dSurfaceHandle {
    Handle::new(GeomAdaptorSurface::with_bounds(&surface, bounds).unwrap()).cast()
}

// Helper to validate UV coordinates: they must be finite and within surface bounds.
fn validate_uv(the_u: f64, the_v: f64, the_surf_name: &str, the_line_idx: i32, the_pnt_idx: i32) {
    assert!(
        the_u.is_finite(),
        "{the_surf_name} U is not finite at line={the_line_idx} pt={the_pnt_idx}"
    );
    assert!(
        the_v.is_finite(),
        "{the_surf_name} V is not finite at line={the_line_idx} pt={the_pnt_idx}"
    );
}

// Regression test for bugs #13/#14: UV coordinate assignment in degenerate
// alpha branches of IntPolyh_MaillageAffinage must correctly pair SetUV1/SetUV2.
#[test]
fn IntPolyh_Intersection__SpherePlane_ValidUVCoordinates() {
    let an_axis = ax3((0.0, 0.0, 0.0), (0.0, 0.0, 1.0));

    let a_sphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(an_axis, 1.0).unwrap()).cast();
    let a_surf_s = adaptor(a_sphere);

    let a_plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(an_axis)).cast();
    let a_surf_p = adaptor(a_plane);

    let an_inter = IntPolyhIntersection::new(&a_surf_s, &a_surf_p).unwrap();

    assert!(an_inter.is_done());

    let a_nb_lines = an_inter.nb_section_lines();
    assert!(a_nb_lines > 0);

    // Verify all section-line points have valid UV coordinates on both surfaces.
    for i_l in 1..=a_nb_lines {
        let a_nb_pts = an_inter.nb_points_in_line(i_l).unwrap();
        for i_p in 1..=a_nb_pts {
            let [_x, _y, _z, u1, v1, u2, v2, _incidence] =
                an_inter.get_line_point(i_l, i_p).unwrap();

            validate_uv(u1, v1, "S1", i_l, i_p);
            validate_uv(u2, v2, "S2", i_l, i_p);
        }
    }
}

// Sphere-cylinder intersection produces section lines with correct UVs.
#[test]
fn IntPolyh_Intersection__SphereCylinder_ValidUVCoordinates() {
    let an_axis = ax3((0.0, 0.0, 0.0), (0.0, 0.0, 1.0));

    let a_sphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(an_axis, 2.0).unwrap()).cast();
    let a_surf_s = adaptor(a_sphere);

    let a_cyl_axis = ax3((1.0, 0.0, 0.0), (0.0, 0.0, 1.0));
    let a_cyl: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(a_cyl_axis, 1.0).unwrap()).cast();
    let a_surf_c = adaptor(a_cyl);

    let an_inter = IntPolyhIntersection::new(&a_surf_s, &a_surf_c).unwrap();
    assert!(an_inter.is_done());

    let a_nb_lines = an_inter.nb_section_lines();
    assert!(a_nb_lines > 0);

    for i_l in 1..=a_nb_lines {
        let a_nb_pts = an_inter.nb_points_in_line(i_l).unwrap();
        for i_p in 1..=a_nb_pts {
            let [_x, _y, _z, u1, v1, u2, v2, _incidence] =
                an_inter.get_line_point(i_l, i_p).unwrap();

            validate_uv(u1, v1, "S1", i_l, i_p);
            validate_uv(u2, v2, "S2", i_l, i_p);

            // UV1 must not equal UV2 (different surfaces have different parametrization).
            let is_same_uv = (u1 - u2).abs() < precision::p_confusion()
                && (v1 - v2).abs() < precision::p_confusion();
            assert!(
                !is_same_uv,
                "UV1 == UV2 at line={i_l} pt={i_p} (possible SetUV copy/paste bug)"
            );
        }
    }
}

// Basic intersection: two tilted planes should intersect along a line.
#[test]
fn IntPolyh_Intersection__TwoPlanes_ProducesSectionLine() {
    let an_axis1 = ax3((0.0, 0.0, 0.0), (0.0, 0.0, 1.0));
    let an_axis2 = ax3((0.0, 0.0, 0.0), (0.0, 1.0, 1.0));

    let a_plane1: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(an_axis1)).cast();
    let a_surf1 = bounded(a_plane1, (-5., 5., -5., 5.));

    let a_plane2: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(an_axis2)).cast();
    let a_surf2 = bounded(a_plane2, (-5., 5., -5., 5.));

    let an_inter = IntPolyhIntersection::new(&a_surf1, &a_surf2).unwrap();
    assert!(an_inter.is_done());
    assert!(an_inter.nb_section_lines() > 0);
}
