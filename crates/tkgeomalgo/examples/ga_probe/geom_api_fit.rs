//! GeomAPI probe operations over the plane conversions, the curve-surface intersection,
//! the interpolation and the approximation, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geomapi_fit.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, show};
use super::curve2d::curve2d;
use super::ics_geometry::{geom_curve, geom_surface};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::geom_api::{self, IntCS, Interpolate, PointsToBSpline};
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Plane3, Point3, Vector3};

/// `ga_gapi_sample`: uniform on a finite range, -1..1 otherwise.
fn sample(f: f64, l: f64, k: i32) -> f64 {
    if !precision::is_infinite(f) && !precision::is_infinite(l) {
        return f + (l - f) * f64::from(k) / 4.0;
    }
    -1.0 + 0.5 * f64::from(k)
}

/// `ga_gapi_curve_out`: null flag, type, bounds, periodicity and five points.
fn curve_out(c: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
    flag(c.is_null());
    if c.is_null() {
        return Ok(());
    }
    int(GeomAdaptorCurve::with_curve(c)?.get_type()? as i32);
    let (f, l) = (c.first_parameter(), c.last_parameter());
    real(f);
    real(l);
    flag(c.is_periodic());
    for k in 0..=4 {
        show(c.value(sample(f, l, k)), pnt);
    }
    Ok(())
}

/// `ga_gapi_curve2d_out`: null flag, type, bounds, periodicity and five points.
fn curve2d_out(c: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
    flag(c.is_null());
    if c.is_null() {
        return Ok(());
    }
    int(Geom2dAdaptorCurve::with_curve(c)?.get_type()? as i32);
    let (f, l) = (c.first_parameter(), c.last_parameter());
    real(f);
    real(l);
    flag(c.is_periodic());
    for k in 0..=4 {
        show(c.value(sample(f, l, k)), pnt2d);
    }
    Ok(())
}

fn plane(r: &mut Reader<'_>) -> Plane3 {
    let p = r.pnt();
    Plane3::new(p, r.dir()).expect("probe plane")
}

/// `curve px py pz dx dy dz`: GeomAPI::To2d.
pub fn to2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = geom_curve(&mut r);
        let p = plane(&mut r);
        curve2d_out(&geom_api::to_2d(&c, &p)?)
    });
    end();
}

/// `curve2d px py pz dx dy dz`: GeomAPI::To3d.
pub fn to3d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = curve2d(&mut r)?;
        let p = plane(&mut r);
        curve_out(&geom_api::to_3d(&c, &p)?)
    });
    end();
}

/// `curve surface mode`: GeomAPI_IntCS.
pub fn intcs_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (c, s, mode) = (geom_curve(&mut r), geom_surface(&mut r), r.integer());
        let mut e = if mode == 0 {
            IntCS::with_curve(&c, &s)?
        } else {
            IntCS::new()
        };
        if mode == 1 {
            e.perform(&c, &s)?;
        }
        flag(e.is_done());
        let n = e.nb_points().unwrap_or(0);
        show(e.nb_points(), int);
        for i in 0..=n + 1 {
            show(e.point(i), pnt);
            show(e.parameters(i), |(u, v, w)| {
                real(u);
                real(v);
                real(w);
            });
        }
        let m = e.nb_segments().unwrap_or(0);
        show(e.nb_segments(), int);
        for i in 0..=m + 1 {
            attempt(|| curve_out(&e.segment(i)?));
            show(e.segment_parameters(i), |(a, b, d, f)| {
                real(a);
                real(b);
                real(d);
                real(f);
            });
        }
        Ok(())
    });
    end();
}

/// `ga_gapi_bspline_out`: degree, periodic, rational, poles (weights), knots and mults.
fn bspline_out(c: &Handle<GeomBSplineCurve>) -> Result<(), Failure> {
    flag(c.is_null());
    let Some(c) = c.get() else { return Ok(()) };
    int(c.degree());
    flag(c.is_periodic());
    flag(c.is_rational());
    int(c.nb_poles());
    for i in 1..=c.nb_poles() {
        pnt(c.pole(i)?);
        if c.is_rational() {
            real(c.weight(i)?);
        }
    }
    int(c.nb_knots());
    for i in 1..=c.nb_knots() {
        real(c.knot(i)?);
        int(c.multiplicity(i)?);
    }
    Ok(())
}

fn vec3(r: &mut Reader<'_>) -> Vector3 {
    let (x, y) = (r.real(), r.real());
    Vector3::new(x, y, r.real())
}

/// `periodic tol n points hasparams [params] mode ...`: GeomAPI_Interpolate.
pub fn interpolate_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (periodic, tol, n) = (r.flag(), r.real(), r.integer());
    let mut points: Array1<'static, Point3> = Array1::with_bounds(1, n).unwrap();
    for i in 1..=n {
        points[i] = r.pnt();
    }
    let pts = Handle::new(HArray1::from_owned(points));
    let params: Option<Handle<HArray1<f64>>> = r.flag().then(|| {
        let np = r.integer();
        let mut values: Array1<'static, f64> = Array1::with_bounds(1, np).unwrap();
        for i in 1..=np {
            values[i] = r.real();
        }
        Handle::new(HArray1::from_owned(values))
    });
    let mode = r.integer();
    attempt(|| {
        let mut it = match &params {
            Some(prm) => Interpolate::with_parameters(&pts, prm, periodic, tol)?,
            None => Interpolate::new(&pts, periodic, tol)?,
        };
        let flags = load_tangents(&mut r, &mut it, mode, n)?;
        it.perform()?;
        flag(it.is_done());
        attempt(|| bspline_out(&it.curve()?));
        bspline_out(&it.as_curve())?;
        if let Some(flags) = flags {
            for i in 1..=n {
                flag(flags.get().unwrap().array1()[i]);
            }
        }
        Ok(())
    });
    end();
}

fn load_tangents(
    r: &mut Reader<'_>,
    it: &mut Interpolate,
    mode: i32,
    n: i32,
) -> Result<Option<Handle<HArray1<bool>>>, Failure> {
    if mode == 1 {
        let (t0, t1) = (vec3(r), vec3(r));
        let scale = r.flag();
        it.load_ends(t0, t1, scale)?;
        return Ok(None);
    }
    if mode != 2 {
        return Ok(None);
    }
    let mut tangents: Array1<'static, Vector3> = Array1::with_bounds(1, n).unwrap();
    let mut flags: Array1<'static, bool> = Array1::with_bounds(1, n).unwrap();
    for i in 1..=n {
        tangents[i] = vec3(r);
        flags[i] = r.flag();
    }
    let flags = Handle::new(HArray1::from_owned(flags));
    let scale = r.flag();
    it.load(&tangents, &flags, scale)?;
    Ok(Some(flags))
}

fn shape(s: i32) -> Shape {
    [
        Shape::C0,
        Shape::G1,
        Shape::C1,
        Shape::G2,
        Shape::C2,
        Shape::C3,
        Shape::CN,
    ][s as usize]
}

fn par_type(p: i32) -> ParametrizationType {
    [
        ParametrizationType::ChordLength,
        ParametrizationType::Centripetal,
        ParametrizationType::IsoParametric,
    ][p as usize]
}

// The Init of `base` (0, 1, 2 or 4) on `f`, or its constructor when `build`; a failed
// constructor keeps the previous object, as the native `reset` of a throwing `new`.
fn fit(
    r: &mut Reader<'_>,
    f: &mut PointsToBSpline,
    (base, build): (i32, bool),
    (pts, prm): (&Array1<'_, Point3>, &Array1<'_, f64>),
) -> Result<(), Failure> {
    match base {
        0 => {
            let (a, b, s, t) = (r.integer(), r.integer(), shape(r.integer()), r.real());
            if !build {
                return f.init(pts, a, b, s, t);
            }
            *f = PointsToBSpline::with_points(pts, a, b, s, t)?;
        }
        1 => {
            let (pt, a, b) = (par_type(r.integer()), r.integer(), r.integer());
            let (s, t) = (shape(r.integer()), r.real());
            if !build {
                return f.init_parametrization(pts, pt, a, b, s, t);
            }
            *f = PointsToBSpline::with_parametrization(pts, pt, a, b, s, t)?;
        }
        2 => {
            let (a, b, s, t) = (r.integer(), r.integer(), shape(r.integer()), r.real());
            if !build {
                return f.init_parameters(pts, prm, a, b, s, t);
            }
            *f = PointsToBSpline::with_parameters(pts, prm, a, b, s, t)?;
        }
        _ => {
            let w = (r.real(), r.real(), r.real());
            let (b, s, t) = (r.integer(), shape(r.integer()), r.real());
            if !build {
                return f.init_weights(pts, w, b, s, t);
            }
            *f = PointsToBSpline::with_weights(pts, w, b, s, t)?;
        }
    }
    Ok(())
}

/// `mode n points [np params] args`: GeomAPI_PointsToBSpline.
pub fn points_to_bspline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, n) = (r.integer(), r.integer());
    let mut pts: Array1<'static, Point3> = Array1::with_bounds(1, n).unwrap();
    for i in 1..=n {
        pts[i] = r.pnt();
    }
    let base: i32 = match mode {
        8 => 4,
        m if m >= 5 => m - 5,
        m => m,
    };
    let np: i32 = if base == 2 { r.integer() } else { 1 };
    let mut prm: Array1<'static, f64> = Array1::with_bounds(1, np).unwrap();
    if base == 2 {
        for i in 1..=np {
            prm[i] = r.real();
        }
    }
    let mut f = PointsToBSpline::new();
    attempt(|| fit(&mut r, &mut f, (base, mode >= 5), (&pts, &prm)));
    flag(f.is_done());
    attempt(|| bspline_out(&f.curve()?));
    let _ = bspline_out(&f.as_curve());
    end();
}
