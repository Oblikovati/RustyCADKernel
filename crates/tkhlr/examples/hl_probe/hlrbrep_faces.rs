//! HLRBRep face fixtures, the Rust twin of the surface part of
//! tools/oracle/tkhlr_probe_hlrbrep_fixtures.cxx.
use super::common::{Reader, int, pnt, real};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomCircle, GeomConicalSurface, GeomCurve,
    GeomCylindricalSurface, GeomLine, GeomPlane, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomToroidalSurface,
};
use rustycad_tkgeomalgo::int_curve_surface::IntersectionPoint;
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

fn surface_poles(r: &mut Reader) -> Array2<'static, Point3> {
    let (nu, nv) = (r.integer(), r.integer());
    let mut poles = Array2::with_bounds(1, nu, 1, nv).expect("probe poles");
    for i in 1..=nu {
        for j in 1..=nv {
            poles
                .set_value(i, j, r.pnt())
                .expect("probe pole inside bounds");
        }
    }
    poles
}

fn clamped(n: i32) -> (Array1<'static, f64>, Array1<'static, i32>) {
    let mut k = Array1::with_bounds(1, n - 1).expect("probe knots");
    let mut m = Array1::with_bounds(1, n - 1).expect("probe knots");
    for i in 1..=n - 1 {
        k[i] = f64::from(i - 1);
        m[i] = if i == 1 || i == n - 1 { 3 } else { 1 };
    }
    (k, m)
}

fn bspline_surface(r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    let poles = surface_poles(r);
    let (nu, nv) = (poles.upper_row(), poles.upper_col());
    let ((uk, um), (vk, vm)) = (clamped(nu), clamped(nv));
    let s = GeomBSplineSurface::new(&poles, (&uk, &um), (&vk, &vm), (2, 2), (false, false))?;
    Ok(Handle::new(s).cast())
}

fn revolution(r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (p, d) = (r.pnt(), r.dir());
    let (l, a) = (r.pnt(), r.dir());
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(p, d)).cast();
    Ok(Handle::new(GeomSurfaceOfRevolution::new(&line, Axis1::new(l, a))?).cast())
}

/// `sk ...`: see `hb_geom_surface`.
pub fn geom_surface(r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    let kind = r.integer();
    match kind {
        5 => return Ok(Handle::new(GeomBezierSurface::new(&surface_poles(r))?).cast()),
        6 => return bspline_surface(r),
        7 => return revolution(r),
        _ => {}
    }
    let (l, n, x) = (r.pnt(), r.dir(), r.dir());
    let frame = Axis3::new(l, n, x)?;
    Ok(match kind {
        0 => Handle::new(GeomPlane::new(frame)).cast(),
        1 => Handle::new(GeomCylindricalSurface::new(frame, r.real())?).cast(),
        2 => {
            let a = r.real();
            Handle::new(GeomConicalSurface::new(frame, a, r.real())?).cast()
        }
        3 => Handle::new(GeomSphericalSurface::new(frame, r.real())?).cast(),
        4 => {
            let a = r.real();
            Handle::new(GeomToroidalSurface::new(frame, a, r.real())?).cast()
        }
        _ => {
            let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(
                Axis2::with_x_direction(l, n, x)?,
                r.real(),
            )?)
            .cast();
            Handle::new(GeomSurfaceOfLinearExtrusion::new(&circle, r.dir())?).cast()
        }
    })
}

/// `surface umin umax vmin vmax`: see `hb_face`.
pub fn face(r: &mut Reader) -> Result<Face, Failure> {
    let s = geom_surface(r)?;
    let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
    BRepBuilderAPIMakeFace::from_surface_bounds(&s, (u1, u2, v1, v2), 1e-7)?.face()
}

/// `hb_ics_point`.
pub fn ics_point(p: &IntersectionPoint) {
    pnt(p.pnt());
    real(p.u());
    real(p.v());
    real(p.w());
    int(p.transition() as i32);
}
