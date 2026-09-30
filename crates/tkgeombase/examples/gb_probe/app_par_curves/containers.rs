//! The Rust twin of tools/oracle/tkgeombase_probe_appparcurves.cxx.
use super::super::common::{self as p, Reader, attempt, end};
use super::super::fem_tool::{read_vector, vector};
use super::{curve, dump, ivector, matrix, point};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_par_curves::{
    self as apc, MultiBSpCurve, MultiCurve, MultiCurveEval, MultiPoint,
};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::math::{IntegerVector, Matrix, Vector};

pub fn bernstein_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let nbpoles = r.integer();
    let u = read_vector(&mut r);
    let ddalen = r.integer();
    attempt(|| {
        let new = || Matrix::filled(u.lower(), u.upper(), 1, nbpoles, -7.).unwrap();
        let (mut a, mut a2, mut da) = (new(), new(), new());
        apc::bernstein_matrix(nbpoles, &u, &mut a)?;
        matrix(&a);
        apc::bernstein(nbpoles, &u, &mut a2, &mut da)?;
        matrix(&a2);
        matrix(&da);
        for i in u.lower()..=u.upper() {
            let mut dda = Vector::filled(1, ddalen, -7.)?;
            apc::second_derivative_bernstein(u[i], &mut dda)?;
            vector(&dda);
        }
        Ok(())
    });
    end();
}

pub fn spline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nbpoles, deg) = (r.integer(), r.integer());
    let (u, flat) = (read_vector(&mut r), read_vector(&mut r));
    attempt(|| {
        let mut a = Matrix::filled(u.lower(), u.upper(), 1, nbpoles, -7.).unwrap();
        let mut da = a.clone();
        let mut index = IntegerVector::filled(u.lower(), u.upper(), -7)?;
        apc::spline_function(nbpoles, deg, &u, &flat, &mut a, &mut da, &mut index)?;
        matrix(&a);
        matrix(&da);
        ivector(&index);
        Ok(())
    });
    end();
}

pub fn read_point(r: &mut Reader<'_>) -> Result<MultiPoint, Failure> {
    let (kind, nb3d, nb2d) = (r.integer(), r.integer(), r.integer());
    let p3: Vec<Point3> = (0..nb3d).map(|_| r.pnt()).collect();
    let p2: Vec<Point2> = (0..nb2d).map(|_| r.pnt2d()).collect();
    if kind == 0 {
        let m = MultiPoint::with_counts(nb3d, nb2d)?;
        for (point, i) in p3.iter().zip(1..) {
            m.set_point(i, *point)?;
        }
        for (point, i) in p2.iter().zip(nb3d + 1..) {
            m.set_point2d(i, *point)?;
        }
        return Ok(m);
    }
    let (a3, a2) = (Array1::from_shared(&p3, 1), Array1::from_shared(&p2, 1));
    Ok(match kind {
        1 => MultiPoint::from_points(&a3),
        2 => MultiPoint::from_points2d(&a2),
        _ => MultiPoint::from_points_and_2d(&a3, &a2),
    })
}

fn point_command(r: &mut Reader<'_>, m: &MultiPoint) -> Result<(), Failure> {
    match r.integer() {
        1 => {
            let i = r.integer();
            m.set_point(i, r.pnt())?;
        }
        2 => p::pnt(m.point(r.integer())?),
        3 => {
            let i = r.integer();
            m.set_point2d(i, r.pnt2d())?;
        }
        4 => p::pnt2d(m.point2d(r.integer())?),
        5 => p::int(m.dimension(r.integer())?),
        6 => {
            let i = r.integer();
            let c = [r.real(), r.real(), r.real(), r.real(), r.real(), r.real()];
            m.transform(i, c)?;
        }
        7 => {
            let i = r.integer();
            let c = [r.real(), r.real(), r.real(), r.real()];
            m.transform2d(i, c)?;
        }
        8 => {
            let mut text = String::new();
            m.dump(&mut text).unwrap();
            dump(&text);
        }
        _ => {
            let copy = m.clone();
            let i = r.integer();
            copy.set_point(i, r.pnt())?;
            point(m)?;
        }
    }
    Ok(())
}

pub fn multipoint_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let m = read_point(&mut r)?;
        point(&m)?;
        for _ in 0..r.integer() {
            attempt(|| point_command(&mut r, &m));
        }
        Ok(())
    });
    end();
}

/// A curve the probe commands act on: the virtual members through MultiCurveEval.
pub trait ProbeCurve: MultiCurveEval {
    fn base(&self) -> &MultiCurve;
    fn base_mut(&mut self) -> &mut MultiCurve;
}
impl ProbeCurve for MultiCurve {
    fn base(&self) -> &MultiCurve {
        self
    }
    fn base_mut(&mut self) -> &mut MultiCurve {
        self
    }
}
impl ProbeCurve for MultiBSpCurve {
    fn base(&self) -> &MultiCurve {
        self
    }
    fn base_mut(&mut self) -> &mut MultiCurve {
        self
    }
}

fn evaluate(r: &mut Reader<'_>, c: &dyn ProbeCurve, code: i32) -> Result<(), Failure> {
    let (k, u) = (r.integer(), r.real());
    if c.base().dimension(k)? == 3 {
        let (pt, v1, v2) = match code {
            1 => (c.value_3d(k, u)?, Default::default(), Default::default()),
            2 => {
                let (pt, v1) = c.d1_3d(k, u)?;
                (pt, v1, Default::default())
            }
            _ => c.d2_3d(k, u)?,
        };
        p::pnt(pt);
        p::vec(v1);
        p::vec(v2);
        return Ok(());
    }
    let (pt, v1, v2) = match code {
        1 => (c.value_2d(k, u)?, Default::default(), Default::default()),
        2 => {
            let (pt, v1) = c.d1_2d(k, u)?;
            (pt, v1, Default::default())
        }
        _ => c.d2_2d(k, u)?,
    };
    p::pnt2d(pt);
    p::vec2d(v1);
    p::vec2d(v2);
    Ok(())
}

fn curve_command(r: &mut Reader<'_>, c: &mut dyn ProbeCurve) -> Result<(), Failure> {
    match r.integer() {
        code @ 1..=3 => evaluate(r, c, code)?,
        4 => p::int(c.base().dimension(r.integer())?),
        5 => {
            let (k, dim) = (r.integer(), r.integer());
            let n = c.nb_poles();
            if dim == 3 {
                let mut t: Array1<'static, Point3> = Array1::with_bounds(1, n).unwrap();
                c.base().curve_3d(k, &mut t)?;
                t.iter().for_each(|q| p::pnt(*q));
            } else {
                let mut t: Array1<'static, Point2> = Array1::with_bounds(1, n).unwrap();
                c.base().curve_2d(k, &mut t)?;
                t.iter().for_each(|q| p::pnt2d(*q));
            }
        }
        6 => {
            let (k, n, dim) = (r.integer(), r.integer(), r.integer());
            if dim == 3 {
                p::pnt(c.base().pole(k, n)?);
            } else {
                p::pnt2d(c.base().pole2d(k, n)?);
            }
        }
        7 => {
            let k = r.integer();
            let v = [r.real(), r.real(), r.real(), r.real(), r.real(), r.real()];
            c.base().transform(k, v)?;
        }
        8 => {
            let k = r.integer();
            let v = [r.real(), r.real(), r.real(), r.real()];
            c.base().transform2d(k, v)?;
        }
        9 => {
            let mut text = String::new();
            c.dump(&mut text).map_err(|_| Failure::new("dump"))?;
            dump(&text);
        }
        10 => p::int(c.degree()?),
        11 => {
            let i = r.integer();
            let m = read_point(r)?;
            c.base().set_value(i, &m)?;
        }
        _ => c.base_mut().set_nb_poles(r.integer())?,
    }
    Ok(())
}

fn curve_commands(r: &mut Reader<'_>, c: &mut dyn ProbeCurve) {
    for _ in 0..r.integer() {
        attempt(|| curve_command(r, c));
    }
}

pub fn read_points(r: &mut Reader<'_>) -> Result<Array1<'static, MultiPoint>, Failure> {
    let n = r.integer();
    let points: Vec<MultiPoint> = (0..n).map(|_| read_point(r)).collect::<Result<_, _>>()?;
    let mut array: Array1<'static, MultiPoint> = Array1::with_bounds(1, n).unwrap();
    array.iter_mut().zip(points).for_each(|(slot, m)| *slot = m);
    Ok(array)
}

pub fn multicurve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    attempt(|| {
        if kind == 0 {
            let mut c = MultiCurve::with_nb_poles(r.integer())?;
            p::int(c.nb_poles());
            p::int(c.nb_curves()?);
            curve_commands(&mut r, &mut c);
            return Ok(());
        }
        let poles = read_points(&mut r)?;
        if kind == 1 {
            let mut c = MultiCurve::from_multi_points(&poles);
            curve(&c)?;
            curve_commands(&mut r, &mut c);
            return Ok(());
        }
        let knots_vector = read_vector(&mut r);
        let knot_values: Vec<f64> = (knots_vector.lower()..=knots_vector.upper())
            .map(|i| knots_vector[i])
            .collect();
        let knots = Array1::from_shared(&knot_values, knots_vector.lower());
        let mults: Vec<i32> = (0..r.integer()).map(|_| r.integer()).collect();
        let mults = Array1::from_shared(&mults, 1);
        let mut c = if kind == 2 {
            MultiBSpCurve::from_multi_points(&poles, &knots, &mults)
        } else {
            MultiBSpCurve::from_multi_curve(&MultiCurve::from_multi_points(&poles), &knots, &mults)
        };
        curve(&c)?;
        p::int(MultiCurveEval::degree(&c)?);
        let k = c.knots()?;
        p::int(k.lower());
        p::int(k.upper());
        k.iter().for_each(|v| p::real(*v));
        c.multiplicities()?.iter().for_each(|m| p::int(*m));
        if kind == 3 {
            c.set_knots(&knots);
            c.set_multiplicities(&mults);
            p::int(MultiCurveEval::degree(&c)?);
        }
        curve_commands(&mut r, &mut c);
        Ok(())
    });
    end();
}
