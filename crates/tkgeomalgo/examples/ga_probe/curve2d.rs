//! The 2D curve reader of the Geom2dInt and Geom2dAPI probes (ga_curve2d of
//! tools/oracle/tkgeomalgo_probe_geom2dint.cxx).
use super::common::Reader;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};

fn dir2d(r: &mut Reader<'_>) -> Direction2 {
    let x = r.real();
    Direction2::new(x, r.real()).expect("probe direction")
}
fn ax2d(r: &mut Reader<'_>) -> Axis2d {
    let p = r.pnt2d();
    Axis2d::new(p, dir2d(r))
}

fn bezier(r: &mut Reader<'_>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let rational = r.flag();
    let n = r.integer();
    let mut p = Array1::<Point2>::with_bounds(1, n).unwrap();
    let mut w = Array1::<f64>::with_bounds(1, n).unwrap();
    for i in 1..=n {
        p[i] = r.pnt2d();
        w[i] = r.real();
    }
    let curve = if rational {
        Geom2dBezierCurve::with_weights(&p, &w)?
    } else {
        Geom2dBezierCurve::new(&p)?
    };
    Ok(Handle::new(curve).cast())
}

fn bspline(r: &mut Reader<'_>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (rational, periodic) = (r.flag(), r.flag());
    let (degree, nk) = (r.integer(), r.integer());
    let mut k = Array1::<f64>::with_bounds(1, nk).unwrap();
    let mut m = Array1::<i32>::with_bounds(1, nk).unwrap();
    for i in 1..=nk {
        k[i] = r.real();
        m[i] = r.integer();
    }
    let np = r.integer();
    let mut p = Array1::<Point2>::with_bounds(1, np).unwrap();
    let mut w = Array1::<f64>::with_bounds(1, np).unwrap();
    for i in 1..=np {
        p[i] = r.pnt2d();
        w[i] = r.real();
    }
    let curve = if rational {
        Geom2dBSplineCurve::with_weights(&p, &w, &k, &m, degree, periodic)?
    } else {
        Geom2dBSplineCurve::new(&p, &k, &m, degree, periodic)?
    };
    Ok(Handle::new(curve).cast())
}

fn conic(kind: i32, r: &mut Reader<'_>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let a = ax2d(r);
    let sense = r.flag();
    let x = r.real();
    if kind == 1 {
        return Ok(Handle::new(Geom2dCircle::from_axis(a, x, sense)?).cast());
    }
    if kind == 4 {
        return Ok(Handle::new(Geom2dParabola::from_axis(a, x, sense)?).cast());
    }
    let y = r.real();
    if kind == 2 {
        return Ok(Handle::new(Geom2dEllipse::from_axis(a, x, y, sense)?).cast());
    }
    Ok(Handle::new(Geom2dHyperbola::from_axis(a, x, y, sense)?).cast())
}

/// One curve specification, as ga_curve2d reads it.
pub fn curve2d(r: &mut Reader<'_>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let kind = r.integer();
    match kind {
        0 => {
            let p = r.pnt2d();
            Ok(Handle::new(Geom2dLine::from_point(p, dir2d(r))).cast())
        }
        1..=4 => conic(kind, r),
        5 => bezier(r),
        6 => bspline(r),
        7 => {
            let (u1, u2) = (r.real(), r.real());
            let (sense, adjust) = (r.flag(), r.flag());
            let basis = curve2d(r)?;
            Ok(Handle::new(Geom2dTrimmedCurve::new(&basis, u1, u2, sense, adjust)?).cast())
        }
        8 => {
            let d = r.real();
            let no_check = r.flag();
            let basis = curve2d(r)?;
            Ok(Handle::new(Geom2dOffsetCurve::new(&basis, d, no_check)?).cast())
        }
        _ => Err(Failure::with_type(
            "Standard_ProgramError",
            "unknown curve kind",
            "",
        )),
    }
}
