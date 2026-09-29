//! BinTools probe cases, part 2: the Rust twin of
//! tools/oracle/tkbrep_probe_bintools_geometry.cxx (the geometry list).
use super::brep::{at, point, point2d};
use super::common::{Reader, word};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomCircle,
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomOffsetSurface, GeomParabola, GeomPlane, GeomRectangularTrimmedSurface,
    GeomSphericalSurface, GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution,
    GeomToroidalSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Axis22d, Direction2, Direction3, Point2, Point3};

/// `BtGeometry`.
#[derive(Default)]
pub struct BtGeometry {
    pub curves: Vec<Handle<dyn GeomCurve>>,
    pub surfaces: Vec<Handle<dyn GeomSurface>>,
    pub pcurves: Vec<Handle<dyn Geom2dCurve>>,
}

fn direction(r: &mut Reader<'_>) -> Result<Direction3, Failure> {
    let (x, y) = (r.real(), r.real());
    Direction3::new(x, y, r.real())
}
fn ax2(r: &mut Reader<'_>) -> Result<Axis2, Failure> {
    let p = point(r);
    let n = direction(r)?;
    Axis2::with_x_direction(p, n, direction(r)?)
}
fn ax3(r: &mut Reader<'_>) -> Result<Axis3, Failure> {
    let p = point(r);
    let n = direction(r)?;
    let mut a = Axis3::new(p, n, direction(r)?)?;
    if r.flag() {
        a.y_reverse();
    }
    Ok(a)
}
fn ax22d(r: &mut Reader<'_>) -> Result<Axis22d, Failure> {
    let p = point2d(r);
    let x = r.real();
    let d = Direction2::new(x, r.real())?;
    Axis22d::new(p, d, r.flag())
}
fn up<T: Transient + RuntimeTyped, U: ?Sized + RuntimeTyped>(value: T) -> Handle<U> {
    Handle::new(value).into_cast()
}

/// `bt_poles`: `n` poles, each followed by its weight when rational.
fn poles<P: Copy + Default>(
    r: &mut Reader<'_>,
    rational: bool,
    n: i32,
    read: fn(&mut Reader<'_>) -> P,
) -> (Vec<P>, Vec<f64>) {
    let (mut p, mut w) = (Vec::new(), Vec::new());
    for _ in 0..n {
        p.push(read(r));
        w.push(if rational { r.real() } else { 1.0 });
    }
    (p, w)
}
fn knots(r: &mut Reader<'_>) -> (Vec<f64>, Vec<i32>) {
    let n = r.integer();
    let (mut k, mut m) = (Vec::new(), Vec::new());
    for _ in 0..n {
        k.push(r.real());
        m.push(r.integer());
    }
    (k, m)
}
fn a1<T: Clone>(v: &[T]) -> Array1<'_, T> {
    Array1::from_shared(v, 1)
}

fn spline(r: &mut Reader<'_>, kind: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
    let rational = r.flag();
    if kind == 5 {
        let n = r.integer();
        let (p, w) = poles(r, rational, n, point);
        return Ok(if rational {
            up(GeomBezierCurve::with_weights(&a1(&p), &a1(&w))?)
        } else {
            up(GeomBezierCurve::new(&a1(&p))?)
        });
    }
    let (periodic, degree, n) = (r.flag(), r.integer(), r.integer());
    let (p, w) = poles(r, rational, n, point);
    let (k, m) = knots(r);
    Ok(if rational {
        up(GeomBSplineCurve::with_weights(
            &a1(&p),
            &a1(&w),
            &a1(&k),
            &a1(&m),
            degree,
            periodic,
        )?)
    } else {
        up(GeomBSplineCurve::new(
            &a1(&p),
            &a1(&k),
            &a1(&m),
            degree,
            periodic,
        )?)
    })
}

fn curve(g: &BtGeometry, r: &mut Reader<'_>, kind: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(match kind {
        0 => {
            let p = point(r);
            up(GeomLine::from_point(p, direction(r)?))
        }
        1 => {
            let a = ax2(r)?;
            up(GeomCircle::new(a, r.real())?)
        }
        2 => {
            let a = ax2(r)?;
            let r1 = r.real();
            up(GeomEllipse::new(a, r1, r.real())?)
        }
        3 => {
            let a = ax2(r)?;
            up(GeomParabola::new(a, r.real())?)
        }
        4 => {
            let a = ax2(r)?;
            let r1 = r.real();
            up(GeomHyperbola::new(a, r1, r.real())?)
        }
        5 | 6 => spline(r, kind)?,
        7 => {
            let (i, u1) = (r.integer(), r.real());
            up(GeomTrimmedCurve::new(
                &at(&g.curves, i),
                u1,
                r.real(),
                true,
                true,
            )?)
        }
        _ => {
            let (i, d) = (r.integer(), r.real());
            up(GeomOffsetCurve::new(
                &at(&g.curves, i),
                d,
                direction(r)?,
                false,
            )?)
        }
    })
}

fn a2(
    v: &[Point3],
    w: &[f64],
    nu: i32,
    nv: i32,
) -> Result<(Array2<'static, Point3>, Array2<'static, f64>), Failure> {
    let wrap = |e: rustycad_tkernel::ncollection::ArrayError| e.into_failure("NCollection_Array2");
    let mut p = Array2::with_bounds(1, nu, 1, nv).map_err(wrap)?;
    let mut ws = Array2::with_bounds(1, nu, 1, nv).map_err(wrap)?;
    for i in 1..=nu {
        for j in 1..=nv {
            let k = ((i - 1) * nv + j - 1) as usize;
            p.set_value(i, j, v[k]).map_err(wrap)?;
            ws.set_value(i, j, w[k]).map_err(wrap)?;
        }
    }
    Ok((p, ws))
}

fn spline_surface(r: &mut Reader<'_>, kind: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
    let rational = r.flag();
    let (mut uper, mut vper, mut udeg, mut vdeg) = (false, false, 0, 0);
    if kind == 18 {
        (uper, vper, udeg, vdeg) = (r.flag(), r.flag(), r.integer(), r.integer());
    }
    let (nu, nv) = (r.integer(), r.integer());
    let (v, w) = poles(r, rational, nu * nv, point);
    let (p, ws) = a2(&v, &w, nu, nv)?;
    if kind == 17 {
        return Ok(if rational {
            up(GeomBezierSurface::with_weights(&p, &ws)?)
        } else {
            up(GeomBezierSurface::new(&p)?)
        });
    }
    let (uk, um) = knots(r);
    let (vk, vm) = knots(r);
    let (u, vv) = ((&a1(&uk), &a1(&um)), (&a1(&vk), &a1(&vm)));
    Ok(if rational {
        up(GeomBSplineSurface::with_weights(
            &p,
            &ws,
            u,
            vv,
            (udeg, vdeg),
            (uper, vper),
        )?)
    } else {
        up(GeomBSplineSurface::new(
            &p,
            u,
            vv,
            (udeg, vdeg),
            (uper, vper),
        )?)
    })
}

fn surface(
    g: &BtGeometry,
    r: &mut Reader<'_>,
    kind: i32,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(match kind {
        10 => up(GeomPlane::new(ax3(r)?)),
        11 => {
            let a = ax3(r)?;
            up(GeomCylindricalSurface::new(a, r.real())?)
        }
        12 => {
            let a = ax3(r)?;
            let rad = r.real();
            up(GeomConicalSurface::new(a, r.real(), rad)?)
        }
        13 => {
            let a = ax3(r)?;
            up(GeomSphericalSurface::new(a, r.real())?)
        }
        14 => {
            let a = ax3(r)?;
            let r1 = r.real();
            up(GeomToroidalSurface::new(a, r1, r.real())?)
        }
        15 => {
            let i = r.integer();
            up(GeomSurfaceOfLinearExtrusion::new(
                &at(&g.curves, i),
                direction(r)?,
            )?)
        }
        16 => {
            let i = r.integer();
            let p = point(r);
            up(GeomSurfaceOfRevolution::new(
                &at(&g.curves, i),
                Axis1::new(p, direction(r)?),
            )?)
        }
        17 | 18 => spline_surface(r, kind)?,
        19 => {
            let (i, u1, u2, v1) = (r.integer(), r.real(), r.real(), r.real());
            up(GeomRectangularTrimmedSurface::new(
                &at(&g.surfaces, i),
                (u1, u2),
                (v1, r.real()),
                (true, true),
            )?)
        }
        _ => {
            let i = r.integer();
            up(GeomOffsetSurface::new(
                &at(&g.surfaces, i),
                r.real(),
                false,
            )?)
        }
    })
}

fn spline2d(r: &mut Reader<'_>, kind: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let rational = r.flag();
    if kind == 35 {
        let n = r.integer();
        let (p, w) = poles(r, rational, n, point2d);
        return Ok(if rational {
            up(Geom2dBezierCurve::with_weights(&a1(&p), &a1(&w))?)
        } else {
            up(Geom2dBezierCurve::new(&a1(&p))?)
        });
    }
    let (periodic, degree, n) = (r.flag(), r.integer(), r.integer());
    let (p, w) = poles(r, rational, n, point2d);
    let (k, m) = knots(r);
    Ok(if rational {
        up(Geom2dBSplineCurve::with_weights(
            &a1(&p),
            &a1(&w),
            &a1(&k),
            &a1(&m),
            degree,
            periodic,
        )?)
    } else {
        up(Geom2dBSplineCurve::new(
            &a1(&p),
            &a1(&k),
            &a1(&m),
            degree,
            periodic,
        )?)
    })
}

fn curve2d(
    g: &BtGeometry,
    r: &mut Reader<'_>,
    kind: i32,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(match kind {
        30 => {
            let p: Point2 = point2d(r);
            let x = r.real();
            up(Geom2dLine::from_point(p, Direction2::new(x, r.real())?))
        }
        31 => {
            let a = ax22d(r)?;
            up(Geom2dCircle::new(a, r.real())?)
        }
        32 => {
            let a = ax22d(r)?;
            let r1 = r.real();
            up(Geom2dEllipse::new(a, r1, r.real())?)
        }
        33 => {
            let a = ax22d(r)?;
            up(Geom2dParabola::new(a, r.real())?)
        }
        34 => {
            let a = ax22d(r)?;
            let r1 = r.real();
            up(Geom2dHyperbola::new(a, r1, r.real())?)
        }
        35 | 36 => spline2d(r, kind)?,
        37 => {
            let (i, u1) = (r.integer(), r.real());
            up(Geom2dTrimmedCurve::new(
                &at(&g.pcurves, i),
                u1,
                r.real(),
                true,
                true,
            )?)
        }
        _ => {
            let i = r.integer();
            up(Geom2dOffsetCurve::new(&at(&g.pcurves, i), r.real(), false)?)
        }
    })
}

/// `bt_read_geometry`.
fn read_geometry(g: &mut BtGeometry, r: &mut Reader<'_>) {
    let kind = r.integer();
    if kind < 10 {
        let made = curve(g, r, kind);
        push(&mut g.curves, made);
    } else if kind < 30 {
        let made = surface(g, r, kind);
        push(&mut g.surfaces, made);
    } else {
        let made = curve2d(g, r, kind);
        push(&mut g.pcurves, made);
    }
}
fn push<T: ?Sized + RuntimeTyped>(list: &mut Vec<Handle<T>>, made: Result<Handle<T>, Failure>) {
    match made {
        Ok(h) => {
            word("-");
            list.push(h);
        }
        Err(e) => {
            word(e.exception_type());
            list.push(Handle::default());
        }
    }
}

/// `bt_geometry`.
pub fn geometry(r: &mut Reader<'_>) -> BtGeometry {
    let mut g = BtGeometry::default();
    for _ in 0..r.integer() {
        read_geometry(&mut g, r);
    }
    g
}
