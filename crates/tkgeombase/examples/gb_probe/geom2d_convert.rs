//! The Geom2dConvert probe; the counterpart of
//! tools/oracle/tkgeombase_probe_geom2dconvert.cxx (same request layout, member order
//! and token stream).
// The printers run inside `attempt`, whose closure returns `Ok(printer(...))`.
#![allow(clippy::unit_arg)]
use super::common::{Reader, attempt, circ2d, end, flag, int, lin2d, pnt2d, real, xy};
use super::lprop::build;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dBoundedCurve, Geom2dCircle, Geom2dCurve,
    Geom2dLine, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::geom2d_convert::{
    self as g2c, ApproxArcsSegments, ApproxCurve, BSplineCurveKnotSplitting,
    BSplineCurveToBezierCurve, CompCurveToBSplineCurve, ConcatOutputs, HArrayOfBSplineCurves,
    PPoint,
};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape;

const PARAMETERISATIONS: [ParameterisationType; 8] = [
    ParameterisationType::TgtThetaOver2,
    ParameterisationType::TgtThetaOver2_1,
    ParameterisationType::TgtThetaOver2_2,
    ParameterisationType::TgtThetaOver2_3,
    ParameterisationType::TgtThetaOver2_4,
    ParameterisationType::QuasiAngular,
    ParameterisationType::RationalC1,
    ParameterisationType::Polynomial,
];
const SHAPES: [Shape; 7] = [
    Shape::C0,
    Shape::G1,
    Shape::C1,
    Shape::G2,
    Shape::C2,
    Shape::C3,
    Shape::CN,
];

fn parameterisation(r: &mut Reader<'_>) -> ParameterisationType {
    PARAMETERISATIONS[r.integer() as usize]
}

/// gb_g2c_bspline.
fn bspline(c: &Handle<Geom2dBSplineCurve>) {
    let Some(c) = c.get() else {
        return int(-1);
    };
    int(c.degree());
    flag(c.is_periodic());
    flag(c.is_rational());
    int(c.nb_poles());
    for i in 1..=c.nb_poles() {
        pnt2d(c.pole(i).unwrap());
        real(c.weight(i).unwrap());
    }
    int(c.nb_knots());
    for i in 1..=c.nb_knots() {
        real(c.knot(i).unwrap());
        int(c.multiplicity(i).unwrap());
    }
}

/// gb_g2c_bezier.
fn bezier(c: &Handle<Geom2dBezierCurve>) {
    let Some(c) = c.get() else {
        return int(-1);
    };
    int(c.degree());
    flag(c.is_rational());
    for i in 1..=c.nb_poles() {
        pnt2d(c.pole(i).unwrap());
        real(c.weight(i).unwrap());
    }
}

/// gb_g2c_element.
fn element(c: &Handle<dyn Geom2dCurve>) {
    if c.is_null() {
        return int(-1);
    }
    let trimmed: Handle<Geom2dTrimmedCurve> = c.cast();
    let Some(t) = trimmed.get() else {
        return int(0);
    };
    let basis: Handle<dyn Geom2dCurve> = t.basis_curve();
    if let Some(l) = basis.cast::<Geom2dLine>().get() {
        int(1);
        lin2d(l.lin2d());
    } else if let Some(ci) = basis.cast::<Geom2dCircle>().get() {
        int(2);
        circ2d(ci.circ2d().unwrap());
    } else {
        return int(0);
    }
    real(t.first_parameter());
    real(t.last_parameter());
    pnt2d(t.start_point().unwrap());
    pnt2d(t.end_point().unwrap());
}

fn read_bspline(r: &mut Reader<'_>) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    Ok(build::curve2d(r)?.cast())
}

/// g2c_to_bspline.
pub fn to_bspline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let p = parameterisation(&mut r);
    attempt(|| {
        let c = build::curve2d(&mut r)?;
        Ok(bspline(&g2c::curve_to_bspline_curve(&c, p)?))
    });
    end();
}

/// g2c_split.
pub fn split_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let (a, b, tol, same) = (r.real(), r.real(), r.real(), r.flag());
    attempt(|| {
        let c = read_bspline(&mut r)?;
        if mode == 0 {
            return Ok(bspline(&g2c::split_bspline_curve_knots(
                &c, a as i32, b as i32, same,
            )?));
        }
        Ok(bspline(&g2c::split_bspline_curve(&c, a, b, tol, same)?))
    });
    end();
}

/// g2c_knot_split.
pub fn knot_split_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let range = r.integer();
    attempt(|| {
        let s = BSplineCurveKnotSplitting::new(&read_bspline(&mut r)?, range)?;
        let n = s.nb_splits();
        int(n);
        for i in 0..=n + 1 {
            attempt(|| Ok(int(s.split_value(i)?)));
        }
        let mut v: Array1<'static, i32> = Array1::with_bounds(1, n).unwrap();
        s.splitting(&mut v)?;
        for i in 1..=n {
            int(v[i]);
        }
        Ok(())
    });
    end();
}

/// g2c_bezier.
pub fn bezier_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let (u1, u2, tol) = (r.real(), r.real(), r.real());
    attempt(|| {
        let c = read_bspline(&mut r)?;
        let s = if mode == 0 {
            BSplineCurveToBezierCurve::new(&c)?
        } else {
            BSplineCurveToBezierCurve::with_range(&c, u1, u2, tol)?
        };
        let n = s.nb_arcs()?;
        int(n);
        let mut k: Array1<'static, f64> = Array1::with_bounds(1, n + 1).unwrap();
        s.knots(&mut k)?;
        for i in 1..=n + 1 {
            real(k[i]);
        }
        for i in 0..=n + 1 {
            attempt(|| Ok(bezier(&s.arc(i)?)));
        }
        let mut arcs: Array1<'static, Handle<Geom2dBezierCurve>> =
            Array1::with_bounds(1, n).unwrap();
        s.arcs(&mut arcs)?;
        for i in 1..=n {
            bezier(&arcs[i]);
        }
        Ok(())
    });
    end();
}

/// g2c_comp.
pub fn comp_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let p = parameterisation(&mut r);
    let (with_first, n) = (r.flag(), r.integer());
    attempt(|| {
        let mut comp: Option<CompCurveToBSplineCurve> = None;
        for i in 0..n {
            let (tol, after) = (r.real(), r.flag());
            let c: Handle<dyn Geom2dBoundedCurve> = build::curve2d(&mut r)?.cast();
            if i == 0 && with_first {
                comp = Some(CompCurveToBSplineCurve::with_curve(&c, p)?);
                continue;
            }
            let comp = comp.get_or_insert_with(|| CompCurveToBSplineCurve::new(p));
            attempt(|| Ok(flag(comp.add(&c, tol, after)?)));
        }
        let comp = comp
            .as_mut()
            .expect("the probe corpus adds at least one curve");
        bspline(&comp.bspline_curve());
        comp.clear();
        Ok(flag(comp.bspline_curve().is_null()))
    });
    end();
}

/// g2c_concat.
pub fn concat_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let mut closed = r.flag();
    let (ctol, ang) = (r.real(), r.real());
    let nt = r.integer();
    let mut tols: Array1<'static, f64> = Array1::with_bounds(0, nt - 1).unwrap();
    for i in 0..nt {
        tols[i] = r.real();
    }
    let nc = r.integer();
    let mut curves: Array1<'static, Handle<Geom2dBSplineCurve>> =
        Array1::with_bounds(0, nc - 1).unwrap();
    for i in 0..nc {
        curves[i] = read_bspline(&mut r).unwrap();
    }
    let mut idx: Handle<HArray1<i32>> = Handle::default();
    let mut res: HArrayOfBSplineCurves = Handle::default();
    attempt(|| match kind {
        0 => g2c::concat_g1(&mut curves, &tols, &mut res, &mut closed, ctol),
        1 => g2c::concat_c1(&mut curves, &tols, &mut idx, &mut res, &mut closed, ctol),
        _ => {
            let outputs = ConcatOutputs {
                indices: &mut idx,
                concatenated: &mut res,
            };
            g2c::concat_c1_with_angular(&mut curves, &tols, outputs, &mut closed, ctol, ang)
        }
    });
    flag(closed);
    match idx.get() {
        None => int(-1),
        Some(t) => {
            let t = t.array1();
            int(t.lower());
            int(t.upper());
            for i in t.lower()..=t.upper() {
                int(t[i]);
            }
        }
    }
    curve_table(&res);
    for i in 0..nc {
        bspline(&curves[i]);
    }
    end();
}

// The bounds and curves of a B-spline table, -1 when null.
fn curve_table(table: &HArrayOfBSplineCurves) {
    let Some(t) = table.get() else {
        return int(-1);
    };
    let t = t.array1();
    int(t.lower());
    int(t.upper());
    for i in t.lower()..=t.upper() {
        bspline(&t[i]);
    }
}

/// g2c_c0c1.
pub fn c0c1_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let (tol, ang) = (r.real(), r.real());
    attempt(|| {
        let mut c = read_bspline(&mut r)?;
        if mode == 0 {
            g2c::c0_bspline_to_c1_bspline_curve(&mut c, tol)?;
            return Ok(bspline(&c));
        }
        let mut t: HArrayOfBSplineCurves = Handle::default();
        attempt(|| {
            if mode == 1 {
                g2c::c0_bspline_to_array_of_c1_bspline_curve(&c, &mut t, tol)
            } else {
                g2c::c0_bspline_to_array_of_c1_bspline_curve_with_angular(&c, &mut t, ang, tol)
            }
        });
        Ok(curve_table(&t))
    });
    end();
}

/// g2c_approx.
pub fn approx_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (via_adaptor, ranged) = (r.flag(), r.flag());
    let (u1, u2, tol) = (r.real(), r.real(), r.real());
    let order = SHAPES[r.integer() as usize];
    let (seg, deg) = (r.integer(), r.integer());
    attempt(|| {
        let c = build::curve2d(&mut r)?;
        let a = if via_adaptor {
            let adaptor = if ranged {
                Geom2dAdaptorCurve::with_range(&c, u1, u2)?
            } else {
                Geom2dAdaptorCurve::with_curve(&c)?
            };
            let h: Handle<dyn Adaptor2dCurve2d> = Handle::new(adaptor).cast();
            ApproxCurve::from_adaptor(&h, tol, order, seg, deg)?
        } else {
            ApproxCurve::from_curve(&c, tol, order, seg, deg)?
        };
        flag(a.is_done());
        flag(a.has_result());
        real(a.max_error());
        Ok(bspline(&a.curve()))
    });
    end();
}

/// g2c_arcs.
pub fn arcs_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let ranged = r.flag();
    let (u1, u2, tol, ang) = (r.real(), r.real(), r.real(), r.real());
    attempt(|| {
        let c = build::curve2d(&mut r)?;
        let a = if ranged {
            Geom2dAdaptorCurve::with_range(&c, u1, u2)?
        } else {
            Geom2dAdaptorCurve::with_curve(&c)?
        };
        let s = ApproxArcsSegments::new(&a, tol, ang)?;
        let seq = s.get_result();
        int(seq.length());
        for curve in seq.iter() {
            element(curve);
        }
        Ok(())
    });
    end();
}

fn ppoint(p: &PPoint) {
    real(p.parameter());
    xy(p.point());
    xy(p.d1());
}

/// g2c_ppoint.
pub fn ppoint_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (u1, u2) = (r.real(), r.real());
    let (v_xy, d) = (r.xy(), r.xy());
    attempt(|| {
        let c = build::curve2d(&mut r)?;
        let a = Geom2dAdaptorCurve::with_curve(&c)?;
        let mut p1 = PPoint::from_adaptor(u1, &a)?;
        let p2 = PPoint::from_adaptor(u2, &a)?;
        let (e, v) = (PPoint::default(), PPoint::new(u2, v_xy, d));
        for p in [&p1, &p2, &e, &v] {
            ppoint(p);
        }
        real(p1.dist(&p2));
        real(p1.dist(&e));
        #[allow(clippy::eq_op)]
        for value in [p1 == p2, p1 != p2, p2 == v, p2 != v, e == e, e != e] {
            flag(value);
        }
        p1.set_d1(d);
        Ok(ppoint(&p1))
    });
    end();
}
