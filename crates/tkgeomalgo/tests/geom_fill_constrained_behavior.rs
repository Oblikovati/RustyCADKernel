//! Behavior of the GeomFill constrained filling (`GeomFill_Boundary` and its
//! boundaries, `GeomFill_CoonsAlgPatch`, `GeomFill_TgtOnCoons`, `GeomFill_CornerState`,
//! `GeomFill_ConstrainedFilling`) beyond the numeric probe.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dLine;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillBoundWithSurf, GeomFillBoundary, GeomFillConstrainedFilling, GeomFillCoonsAlgPatch,
    GeomFillCornerState, GeomFillDegeneratedBound, GeomFillSimpleBound, GeomFillTgtField,
    GeomFillTgtOnCoons,
};
use rustycad_tkmath::gp::{Axis1, Direction2, Direction3, Point2, Point3};

type Bound = Handle<dyn GeomFillBoundary>;

/// A straight boundary from `a` to `b` on [0, 1].
fn segment(a: Point3, b: Point3) -> Bound {
    let d = Direction3::new(b.x() - a.x(), b.y() - a.y(), b.z() - a.z()).unwrap();
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(a, d))).cast();
    let length = a.distance(&b);
    let curve: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(&line, 0., length).unwrap()).cast();
    Handle::new(GeomFillSimpleBound::new(&curve, 1e-4, 1e-2)).cast()
}

/// A boundary on the plane z = 0 along the 2d segment `p + t d`, t in [0, length].
fn on_plane(p: (f64, f64), d: (f64, f64), length: f64) -> Bound {
    let plane: Handle<dyn GeomSurface> = Handle::new(
        GeomPlane::from_point(
            Point3::new(0., 0., 0.),
            Direction3::new(0., 0., 1.).unwrap(),
        )
        .unwrap(),
    )
    .cast();
    let surface: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&plane).unwrap()).cast();
    let line = Handle::new(Geom2dLine::from_point(
        Point2::new(p.0, p.1),
        Direction2::new(d.0, d.1).unwrap(),
    ))
    .cast();
    let curve: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(&line, 0., length).unwrap()).cast();
    let cos = Adaptor3dCurveOnSurface::with_curve(&curve, &surface).unwrap();
    Handle::new(GeomFillBoundWithSurf::new(&cos, 1e-4, 1e-2)).cast()
}

fn square() -> [Bound; 4] {
    [
        on_plane((0., 0.), (1., 0.), 2.),
        on_plane((2., 0.), (0., 1.), 2.),
        on_plane((2., 2.), (-1., 0.), 2.),
        on_plane((0., 2.), (0., -1.), 2.),
    ]
}

#[test]
fn corner_state_kills_only_constrained_corners() {
    let mut corner = GeomFillCornerState::new();
    assert_eq!(corner.gap(), f64::MAX);
    corner.do_kill(0.5);
    assert_eq!(corner.is_to_kill(), (false, 0.5));
    corner.constraint();
    assert_eq!(corner.is_to_kill(), (true, 0.5));
}

#[test]
fn simple_bound_has_no_normals() -> Result<(), Failure> {
    let bound = segment(Point3::new(0., 0., 0.), Point3::new(3., 4., 0.));
    assert!(!bound.has_normals());
    assert_eq!(
        bound.norm(0.5).unwrap_err().exception_type(),
        "Standard_Failure"
    );
    assert_eq!(bound.bounds()?, (0., 5.));
    bound.reparametrize(0., 1., false, false, 1., 1., true)?;
    let (first, last) = bound.points()?;
    assert!(first.distance(&Point3::new(3., 4., 0.)) < 1e-6);
    assert!(last.distance(&Point3::new(0., 0., 0.)) < 1e-6);
    Ok(())
}

#[test]
fn degenerated_bound_keeps_its_point() -> Result<(), Failure> {
    let point = Point3::new(1., 2., 3.);
    let bound = GeomFillDegeneratedBound::new(point, 0., 1., 1e-4, 10.);
    assert!(bound.is_degenerated());
    bound.reparametrize(2., 5., true, true, 1., 1., true)?;
    assert_eq!(bound.bounds()?, (2., 5.));
    assert_eq!(bound.d1(3.)?, (point, Default::default()));
    bound.set_tol3d(0.5);
    assert_eq!(bound.tol3d(), 0.5);
    Ok(())
}

#[test]
fn bound_with_surf_normals_are_the_plane_normal() -> Result<(), Failure> {
    let bound = on_plane((0., 0.), (1., 0.), 2.);
    assert!(bound.has_normals());
    let n = bound.norm(0.7)?;
    assert!((n.z() - 1.).abs() < 1e-12);
    let (_, dn) = bound.d1_norm(0.7)?;
    assert!(dn.magnitude() < 1e-12);
    Ok(())
}

#[test]
fn coons_patch_bounds_are_checked() -> Result<(), Failure> {
    let [b1, b2, b3, b4] = square();
    let patch = Handle::new(GeomFillCoonsAlgPatch::new([&b1, &b2, &b3, &b4])?);
    assert_eq!(
        patch.bound(4).err().map(|e| e.exception_type()),
        Some("Standard_OutOfRange")
    );
    // The mean of B4 at the start of B2 and B1 at its start: B4 runs the other way.
    assert!(patch.corner(0)?.distance(&Point3::new(0., 1., 0.)) < 1e-12);
    let field = GeomFillTgtOnCoons::new(&patch, 0);
    assert!(!field.is_scalable());
    let (t, _) = field.d1_with_value(1.)?;
    assert!(t.z().abs() < 1e-12);
    Ok(())
}

#[test]
fn constrained_filling_of_a_planar_square_is_planar() -> Result<(), Failure> {
    let mut filling = GeomFillConstrainedFilling::new(8, 3);
    assert_eq!(
        filling.re_build().unwrap_err().exception_type(),
        "Standard_Failure"
    );
    assert!(filling.surface().is_null());
    let [b1, b2, b3, b4] = square();
    filling.init_four([&b3, &b1, &b4, &b2], false)?;
    let surface = filling.surface();
    for (u, v) in [(0.1, 0.2), (0.5, 0.5), (0.9, 0.7)] {
        assert!(surface.eval_d0(u, v)?.z().abs() < 1e-7);
    }
    assert_eq!(filling.boundary(0)?.object_address(), b3.object_address());
    let mut values = [0.; 12];
    assert_eq!(filling.eval(0.5, 0, &mut values)?, 0);
    let mut short = [0.; 2];
    assert_eq!(
        filling
            .eval(0.5, 0, &mut short)
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
    Ok(())
}

#[test]
fn constrained_filling_checks_write_their_report() -> Result<(), Failure> {
    let mut filling = GeomFillConstrainedFilling::new(8, 3);
    let [b1, b2, b3, _] = square();
    filling.init_three([&b1, &b2, &b3], false)?;
    let mut text = String::new();
    filling.check_approx(0, &mut text)?;
    filling.check_result(1, &mut text)?;
    filling.check_tgte_field(0, &mut text)?;
    assert!(text.starts_with("Controle approx/contrainte sur bord 0 : \nDistance max : "));
    assert!(text.contains("KAlgo angle max sur bord 0 : "));
    Ok(())
}

#[test]
fn set_domain_changes_the_tangent_blending() -> Result<(), Failure> {
    let mut filling = GeomFillConstrainedFilling::new(8, 3);
    let [b1, b2, b3, b4] = square();
    filling.init_four([&b1, &b2, &b3, &b4], true)?;
    filling.set_domain(0.5, &b1.cast::<GeomFillBoundWithSurf>())?;
    filling.re_build()?;
    assert!(!filling.surface().is_null());
    Ok(())
}
