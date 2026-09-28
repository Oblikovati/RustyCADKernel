//! Counterparts of the D0..DN and flat-knot Eval cases of tkmath_bspl_eval_probe.cxx.
use super::curves::Curve;
use super::{In, fail, o, ob, op, ov, report};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::bsplclib::{self, CurvePoint, CurveSpan};

fn points_eval<P: CurvePoint>(span: &CurveSpan<'_, '_>, c: &Curve, poles: &Array1<'_, P>, n: i32) {
    report(bsplclib::d0(span, poles, c.w()), |p| op(&p));
    report(bsplclib::d1(span, poles, c.w()), |(p, v1)| {
        op(&p);
        ov::<P>(&v1);
    });
    report(bsplclib::d2(span, poles, c.w()), |(p, v1, v2)| {
        op(&p);
        ov::<P>(&v1);
        ov::<P>(&v2);
    });
    report(bsplclib::d3(span, poles, c.w()), |(p, v1, v2, v3)| {
        op(&p);
        ov::<P>(&v1);
        ov::<P>(&v2);
        ov::<P>(&v3);
    });
    report(bsplclib::dn(span, n, poles, c.w()), |v| ov::<P>(&v));
}

// eval: curve flat u index n
pub fn eval_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (flat, u, index, n) = (f.b(), f.r(), f.i(), f.i());
    let span = CurveSpan {
        u,
        index,
        degree: c.deg,
        periodic: c.per,
        knots: &c.k,
        mults: if flat { None } else { Some(&c.m) },
    };
    match c.kind {
        2 => points_eval(&span, &c, &c.p2, n),
        3 => points_eval(&span, &c, &c.p3, n),
        _ => scalar_eval(&span, &c, n),
    }
    println!();
}

fn scalar_eval(span: &CurveSpan<'_, '_>, c: &Curve, n: i32) {
    report(bsplclib::d0_scalar(span, &c.f, c.w()), o);
    report(bsplclib::d1_scalar(span, &c.f, c.w()), |(p, v1)| {
        ob(&[p, v1])
    });
    report(bsplclib::d2_scalar(span, &c.f, c.w()), |(p, v1, v2)| {
        ob(&[p, v1, v2])
    });
    report(bsplclib::d3_scalar(span, &c.f, c.w()), |(p, v1, v2, v3)| {
        ob(&[p, v1, v2, v3])
    });
    report(bsplclib::dn_scalar(span, n, &c.f, c.w()), o);
}

/// The poles of a curve as contiguous reals.
pub fn flat_of(c: &Curve) -> Vec<f64> {
    match c.kind {
        1 => c.f.as_slice().to_vec(),
        2 => coordinates(&c.p2),
        _ => coordinates(&c.p3),
    }
}

fn coordinates<P: CurvePoint>(points: &Array1<'_, P>) -> Vec<f64> {
    points
        .as_slice()
        .iter()
        .flat_map(|point| (0..P::DIMENSION).map(|k| point.pole_coordinate(k)))
        .collect()
}

/// The working dimension: `dim` for flat curves, 2 or 3 for points.
pub fn dimension(c: &Curve) -> i32 {
    if c.kind == 1 { c.dim } else { c.kind }
}

// flat_eval: curve(knots = flat knots) param request em0 em1 homogeneous
pub fn flat_eval_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (u, req, em, hom) = (f.r(), f.i(), [f.i(), f.i()], f.b());
    let dim: i32 = dimension(&c);
    let flat: Vec<f64> = flat_of(&c);
    let mut res: Vec<f64> = vec![91.0; ((req + 1) * dim) as usize];
    let mut wres: Vec<f64> = vec![91.0; ((req + 1) * dim + 1) as usize];
    report(
        bsplclib::eval_extrapolated(u, c.per, req, &em, c.deg, &c.k, dim, &flat, &mut res),
        |()| ob(&res),
    );
    if c.rat {
        res.fill(91.0);
        let result = bsplclib::eval_extrapolated_rational(
            u,
            c.per,
            req,
            &em,
            c.deg,
            &c.k,
            dim,
            &flat,
            c.w.as_slice(),
            &mut res,
            &mut wres,
        );
        report(result, |()| {
            ob(&res);
            ob(&wres);
        });
    }
    if c.kind > 1 && c.rat {
        match c.kind {
            2 => point_eval(&c, &c.p2, u, hom, &em),
            _ => point_eval(&c, &c.p3, u, hom, &em),
        }
    }
    println!();
}

fn point_eval<P: CurvePoint>(c: &Curve, poles: &Array1<'_, P>, u: f64, hom: bool, em: &[i32; 2]) {
    let mut point: P = P::point_from(&[91.0, 91.0, 91.0]);
    let mut weight: f64 = 91.0;
    let result = bsplclib::eval_extrapolated_point(
        u,
        c.per,
        hom,
        em,
        c.deg,
        &c.k,
        poles,
        &c.w,
        &mut point,
        &mut weight,
    );
    match result {
        Ok(()) => {
            op(&point);
            o(weight);
        }
        Err(failure) => fail(&failure),
    }
}
