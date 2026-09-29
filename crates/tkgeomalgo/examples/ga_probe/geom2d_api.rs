//! Geom2dAPI probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geom2dapi.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, show};
use super::curve2d::curve2d;
use super::int_imp_par_gen::vec2d;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkgeomalgo::geom2d_api::{
    ExtremaCurveCurve, Interpolate, PointsToBSpline, ProjectPointOnCurve,
};
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Vector2};

/// `ga_bspline_out`: degree, periodic, rational, poles (weights), knots and mults.
fn bspline_out(c: &Handle<Geom2dBSplineCurve>) -> Result<(), Failure> {
    flag(c.is_null());
    let Some(c) = c.get() else { return Ok(()) };
    int(c.degree());
    flag(c.is_periodic());
    flag(c.is_rational());
    int(c.nb_poles());
    for i in 1..=c.nb_poles() {
        pnt2d(c.pole(i)?);
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

/// `curve1 curve2 u1min u1max u2min u2max`: every member of ExtremaCurveCurve.
pub fn extrema_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (c1, c2) = (curve2d(&mut r)?, curve2d(&mut r)?);
        let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
        let e = ExtremaCurveCurve::new(&c1, &c2, a, b, c, d)?;
        let n = e.nb_extrema();
        int(n);
        show(e.as_int(), int);
        for i in 0..=n + 1 {
            show(e.points(i), |(p, q)| {
                pnt2d(p);
                pnt2d(q);
            });
            show(e.parameters(i), |(u, v)| {
                real(u);
                real(v);
            });
            show(e.distance(i), real);
        }
        show(e.nearest_points(), |(p, q)| {
            pnt2d(p);
            pnt2d(q);
        });
        show(e.lower_distance_parameters(), |(u, v)| {
            real(u);
            real(v);
        });
        show(e.lower_distance(), real);
        show(e.as_real(), real);
        let x = e.extrema()?;
        flag(x.is_done());
        show(x.is_parallel(), flag);
        Ok(())
    });
    end();
}

/// `curve x y ranged umin usup`: every member of ProjectPointOnCurve.
pub fn project_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = curve2d(&mut r)?;
        let p = r.pnt2d();
        let (ranged, u0, u1) = (r.flag(), r.real(), r.real());
        let mut e = ProjectPointOnCurve::new();
        int(e.nb_points());
        if ranged {
            e.init_range(p, &c, u0, u1)?;
        } else {
            e.init(p, &c)?;
        }
        let n = e.nb_points();
        int(n);
        int(e.as_int());
        for i in 0..=n + 1 {
            show(e.point(i), pnt2d);
            show(e.parameter(i), real);
            show(e.parameter(i), real);
            show(e.distance(i), real);
        }
        show(e.nearest_point(), pnt2d);
        show(e.as_point(), pnt2d);
        show(e.lower_distance_parameter(), real);
        show(e.lower_distance(), real);
        show(e.as_real(), real);
        flag(e.extrema()?.is_done());
        Ok(())
    });
    end();
}

/// `periodic tol n points hasparams [params] mode ...`: Interpolate.
pub fn interpolate_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (periodic, tol, n) = (r.flag(), r.real(), r.integer());
    let mut points: Array1<'static, Point2> = Array1::with_bounds(1, n).unwrap();
    for i in 1..=n {
        points[i] = r.pnt2d();
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
        let (t0, t1) = (vec2d(r), vec2d(r));
        let scale = r.flag();
        it.load_ends(t0, t1, scale)?;
        return Ok(None);
    }
    if mode != 2 {
        return Ok(None);
    }
    let mut tangents: Array1<'static, Vector2> = Array1::with_bounds(1, n).unwrap();
    let mut flags: Array1<'static, bool> = Array1::with_bounds(1, n).unwrap();
    for i in 1..=n {
        tangents[i] = vec2d(r);
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

/// `mode n values [params] args`: PointsToBSpline.
pub fn points_to_bspline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, n) = (r.integer(), r.integer());
    let mut pts: Array1<'static, Point2> = Array1::with_bounds(1, n).unwrap();
    let mut ys: Array1<'static, f64> = Array1::with_bounds(1, n).unwrap();
    let mut prm: Array1<'static, f64> = Array1::with_bounds(1, n).unwrap();
    for i in 1..=n {
        if mode == 3 {
            ys[i] = r.real();
        } else {
            pts[i] = r.pnt2d();
        }
    }
    if mode == 2 {
        for i in 1..=n {
            prm[i] = r.real();
        }
    }
    let mut f = PointsToBSpline::new();
    attempt(|| match mode {
        0 => {
            let (a, b, s, t) = (r.integer(), r.integer(), r.integer(), r.real());
            f.init(&pts, a, b, shape(s), t)
        }
        1 => {
            let (pt, a, b, s, t) = (r.integer(), r.integer(), r.integer(), r.integer(), r.real());
            f.init_parametrization(&pts, par_type(pt), a, b, shape(s), t)
        }
        2 => {
            let (a, b, s, t) = (r.integer(), r.integer(), r.integer(), r.real());
            f.init_parameters(&pts, &prm, a, b, shape(s), t)
        }
        3 => {
            let (x0, dx) = (r.real(), r.real());
            let (a, b, s, t) = (r.integer(), r.integer(), r.integer(), r.real());
            f.init_y_values(&ys, x0, dx, a, b, shape(s), t)
        }
        _ => {
            let (w1, w2, w3) = (r.real(), r.real(), r.real());
            let (b, s, t) = (r.integer(), r.integer(), r.real());
            f.init_weights(&pts, (w1, w2, w3), b, shape(s), t)
        }
    });
    flag(f.is_done());
    attempt(|| bspline_out(&f.curve()?));
    end();
}
