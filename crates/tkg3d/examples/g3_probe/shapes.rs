//! The curve and surface specifications of tools/oracle/tkg3d_probe_common.cxx
//! (g3_curve and g3_surface), built with the Rust classes.
use super::print::Reader;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomLine, GeomSurface,
};
use rustycad_tkmath::gp::Point3;

/// One curve specification, as g3_curve reads it.
pub fn curve(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let kind = r.integer();
    match kind {
        0 => {
            let p = r.pnt();
            Ok(Handle::new(GeomLine::from_point(p, r.dir())).cast())
        }
        1 => {
            let frame = r.ax2();
            Ok(Handle::new(GeomCircle::new(frame, r.real())?).cast())
        }
        5 => bezier_curve(r),
        6 => bspline_curve(r),
        _ => panic!("curve kind {kind} has no Rust builder in the probe"),
    }
}

/// One surface specification, as g3_surface reads it.
pub fn surface(r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    let kind = r.integer();
    panic!("surface kind {kind} has no Rust builder in the probe")
}

// Poles `(x y z w)*n` of the spline specifications.
fn weighted_poles(r: &mut Reader, n: i32) -> (Array1<'static, Point3>, Array1<'static, f64>) {
    let mut p = Array1::<Point3>::with_bounds(1, n).unwrap();
    let mut w = Array1::<f64>::with_bounds(1, n).unwrap();
    for i in 1..=n {
        p[i] = r.pnt();
        w[i] = r.real();
    }
    (p, w)
}

// Kind 5: bezier rational n (x y z w)*n.
fn bezier_curve(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let rational = r.flag();
    let n = r.integer();
    let (p, w) = weighted_poles(r, n);
    let curve = if rational {
        GeomBezierCurve::with_weights(&p, &w)?
    } else {
        GeomBezierCurve::new(&p)?
    };
    Ok(Handle::new(curve).cast())
}

// Kind 6: bspline rational periodic degree nk (k m)*nk np (x y z w)*np.
fn bspline_curve(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let (rational, periodic) = (r.flag(), r.flag());
    let (degree, nk) = (r.integer(), r.integer());
    let mut k = Array1::<f64>::with_bounds(1, nk).unwrap();
    let mut m = Array1::<i32>::with_bounds(1, nk).unwrap();
    for i in 1..=nk {
        k[i] = r.real();
        m[i] = r.integer();
    }
    let np = r.integer();
    let (p, w) = weighted_poles(r, np);
    let curve = if rational {
        GeomBSplineCurve::with_weights(&p, &w, &k, &m, degree, periodic)?
    } else {
        GeomBSplineCurve::new(&p, &k, &m, degree, periodic)?
    };
    Ok(Handle::new(curve).cast())
}
