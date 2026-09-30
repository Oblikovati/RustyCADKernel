//! Bisector probe operations on the algorithmic bisectors, the intersection and the
//! helpers; the Rust twin of the second half of tools/oracle/tktopalgo_probe_bisector.cxx.
use super::bisector::{
    bisector_curve, curve_passage, curves_passage, join, sample, samples, span, vec,
};
use super::common::{Reader, attempt, captured, end, flag, int, pnt2d, real};
use super::curve2d::curve2d;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCartesianPoint, Geom2dCurve, Geom2dGeometry, Geom2dPoint};
use rustycad_tkgeomalgo::int_res2d::Domain;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};
use rustycad_tktopalgo::bisector::{
    Bisec, BisecCC, BisecPC, BisectorCurve, BisectorFunctionH, BisectorFunctionInter,
    BisectorInter, PointOnBis, PolyBis, is_convex,
};

fn point(r: &mut Reader<'_>) -> Handle<dyn Geom2dPoint> {
    Handle::new(Geom2dCartesianPoint::from_pnt2d(r.pnt2d())).cast()
}
pub(super) fn polygon(poly: &PolyBis) {
    int(poly.length());
    for i in 1..=poly.length() {
        let p: PointOnBis = poly.value(i).expect("polygon point");
        real(p.param_on_c1());
        real(p.param_on_c2());
        real(p.param_on_bis());
        real(p.distance());
        pnt2d(p.point());
    }
}
fn cc_values(b: &Handle<BisecCC>, u: f64) {
    for by_int in [false, true] {
        attempt(|| {
            let v = if by_int {
                b.value_by_int(u)?
            } else {
                b.value_and_dist(u)?
            };
            pnt2d(v.point);
            real(v.u1);
            real(v.u2);
            real(v.distance);
            Ok(())
        });
    }
    attempt(|| {
        let (p, v1, v2) = b.d2(u)?;
        pnt2d(p);
        vec(v1);
        vec(v2);
        Ok(())
    });
    attempt(|| {
        vec(b.d3(u)?.3);
        Ok(())
    });
    attempt(|| {
        vec(b.dn(u, 1)?);
        Ok(())
    });
    attempt(|| {
        vec(b.dn(u, 4)?);
        Ok(())
    });
}
fn cc_members(b: &Handle<BisecCC>) -> Result<(), Failure> {
    let (a, e) = (b.first_parameter(), b.last_parameter());
    real(a);
    real(e);
    attempt(|| {
        flag(b.is_closed()?);
        Ok(())
    });
    flag(b.is_periodic());
    int(b.continuity() as i32);
    int(b.interval_continuity() as i32);
    attempt(|| {
        flag(b.is_cn(1)?);
        Ok(())
    });
    attempt(|| {
        real(b.reversed_parameter(a)?);
        Ok(())
    });
    real(b.link_bis_curve(a));
    real(b.link_curve_bis(a));
    let bc: Handle<dyn BisectorCurve> = b.cast();
    samples(&b.cast(), &bc, a, e);
    let (lo, hi) = span(a, e);
    for k in 0..=4 {
        cc_values(b, sample(lo, hi, k));
    }
    cc_values(b, a - 1.);
    cc_values(b, e + 1.);
    attempt(|| {
        let g = b.change_guide()?;
        polygon(&g.polygon());
        real(g.first_parameter());
        real(g.last_parameter());
        Ok(())
    });
    attempt(|| {
        let c: Handle<BisecCC> = b.copy()?.cast();
        real(c.first_parameter());
        polygon(&c.polygon());
        Ok(())
    });
    captured(|out| b.dump(0, 1, out));
    Ok(())
}
/// `c1 c2 side1 side2 ox oy distmax`: Bisector_BisecCC and its members.
pub fn bisec_cc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c1 = curve2d(&mut r)?;
        let c2 = curve2d(&mut r)?;
        let (s1, s2) = (r.real(), r.real());
        let o = r.pnt2d();
        let dmax = r.real();
        let b: Handle<BisecCC> = Handle::new(BisecCC::with_curves(&c1, &c2, s1, s2, o, dmax)?);
        flag(b.is_empty());
        polygon(&b.polygon());
        let bc: Handle<dyn BisectorCurve> = b.cast();
        bisector_curve(&bc);
        if b.nb_intervals() > 0 {
            cc_members(&b)?;
        }
        attempt(|| b.reverse());
        Ok(())
    });
    end();
}
fn pc_members(b: &Handle<BisecPC>) {
    flag(b.is_empty());
    let bc: Handle<dyn BisectorCurve> = b.cast();
    bisector_curve(&bc);
    if b.nb_intervals() == 0 {
        return;
    }
    let (a, e) = (b.first_parameter(), b.last_parameter());
    real(a);
    real(e);
    attempt(|| {
        flag(b.is_closed()?);
        Ok(())
    });
    flag(b.is_periodic());
    int(b.continuity() as i32);
    int(b.interval_continuity() as i32);
    attempt(|| {
        flag(b.is_cn(1)?);
        Ok(())
    });
    attempt(|| {
        real(b.reversed_parameter(a)?);
        Ok(())
    });
    real(b.link_bis_curve(a));
    real(b.link_curve_bis(a));
    samples(&b.cast(), &bc, a, e);
    let (lo, hi) = span(a, e);
    for k in 0..=4 {
        pc_sample(b, sample(lo, hi, k));
    }
    for u in [a - 1., e + 1.] {
        attempt(|| {
            let (p, v) = b.d1(u)?;
            pnt2d(p);
            vec(v);
            Ok(())
        });
    }
    attempt(|| {
        let c: Handle<BisecPC> = b.copy()?.cast();
        real(c.first_parameter());
        real(c.last_parameter());
        Ok(())
    });
    captured(|out| b.dump(0, 1, out));
    attempt(|| b.reverse());
}
fn pc_sample(b: &Handle<BisecPC>, u: f64) {
    attempt(|| {
        real(b.distance(u)?);
        Ok(())
    });
    attempt(|| {
        let (p, v1, v2) = b.d2(u)?;
        pnt2d(p);
        vec(v1);
        vec(v2);
        Ok(())
    });
    attempt(|| {
        let (p, _, _, v3) = b.d3(u)?;
        pnt2d(p);
        vec(v3);
        Ok(())
    });
    attempt(|| {
        vec(b.dn(u, 2)?);
        Ok(())
    });
    attempt(|| {
        vec(b.dn(u, 4)?);
        Ok(())
    });
}
/// `mode c x y side distmax|umin umax`: Bisector_BisecPC and its members.
pub fn bisec_pc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let mode = r.integer();
        let c = curve2d(&mut r)?;
        let p = r.pnt2d();
        let side = r.real();
        let b = if mode == 0 {
            let dmax = r.real();
            BisecPC::with_curve(&c, p, side, dmax)?
        } else {
            let (u1, u2) = (r.real(), r.real());
            BisecPC::with_bounds(&c, p, side, u1, u2)?
        };
        pc_members(&Handle::new(b));
        Ok(())
    });
    end();
}
fn read_bisec(r: &mut Reader<'_>) -> Result<(Bisec, Domain), Failure> {
    let kind = r.integer();
    let automatic = r.flag();
    let mut b = Bisec::new();
    if kind == 0 {
        let c1 = curve2d(r)?;
        let c2 = curve2d(r)?;
        let (sense, jt, tol, oncurve) = (r.real(), r.integer(), r.real(), r.flag());
        let s = curves_passage(r, automatic, &c1, &c2)?;
        b.perform_curves(&c1, &c2, s.p, s.v1, s.v2, sense, join(jt), tol, oncurve)?;
    } else {
        let c = curve2d(r)?;
        let pnt = point(r);
        let (sense, tol, oncurve) = (r.real(), r.real(), r.flag());
        let s = curve_passage(r, automatic, &c, kind == 1)?;
        if kind == 1 {
            b.perform_curve_point(&c, &pnt, s.p, s.v1, s.v2, sense, tol, oncurve)?;
        } else {
            b.perform_point_curve(&pnt, &c, s.p, s.v1, s.v2, sense, tol, oncurve)?;
        }
    }
    let tol = r.real();
    let t = b.value();
    let a = t.first_parameter();
    let mut e = t.last_parameter();
    if e > a + 100. {
        e = a + 100.;
    }
    let d = Domain::bounded(t.value(a)?, a, tol, t.value(e)?, e, tol);
    Ok((b, d))
}
/// `bisec1 bisec2 tolconf tol comun`: Bisector_Inter of two bisectors on their domains.
pub fn bisec_inter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (b1, d1) = read_bisec(&mut r)?;
        let (b2, d2) = read_bisec(&mut r)?;
        let (tc, t, comun) = (r.real(), r.real(), r.flag());
        let mut i = BisectorInter::new();
        i.perform(&b1, &d1, &b2, &d2, tc, t, comun)?;
        flag(i.is_done());
        if i.is_done() {
            int(i.nb_points()?);
            for k in 1..=i.nb_points()? {
                let p = i.point(k)?;
                pnt2d(p.value());
                real(p.param_on_first());
                real(p.param_on_second());
            }
            int(i.nb_segments()?);
            for k in 1..=i.nb_segments()? {
                let s = i.segment(k)?;
                flag(s.has_first_point());
                flag(s.has_last_point());
                if s.has_first_point() {
                    pnt2d(s.first_point()?.value());
                }
            }
        }
        Ok(())
    });
    end();
}
/// `c x y tx ty u`: Bisector_FunctionH at `u`.
pub fn bisec_function_h_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = curve2d(&mut r)?;
        let p = r.pnt2d();
        let t = r.pnt2d();
        let u = r.real();
        let mut h = BisectorFunctionH::new(&c, p, Vector2::from(t.xy()))?;
        let (mut f, mut d) = (0.0, 0.0);
        // A native evaluation failure leaves the member by an exception, before its
        // flag is printed.
        let done = h.value(u, &mut f);
        h.raised()?;
        flag(done);
        real(f);
        let done = h.derivative(u, &mut d);
        h.raised()?;
        flag(done);
        real(d);
        (f, d) = (0.0, 0.0);
        let done = h.values(u, &mut f, &mut d);
        h.raised()?;
        flag(done);
        real(f);
        real(d);
        Ok(())
    });
    end();
}
/// `c x1 y1 x2 y2 side u`: Bisector_FunctionInter of a curve and two curve-point bisectors.
pub fn bisec_function_inter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = curve2d(&mut r)?;
        let (p1, p2) = (r.pnt2d(), r.pnt2d());
        let (side, u) = (r.real(), r.real());
        let (first, last) = (c.first_parameter(), c.last_parameter());
        let b1: Handle<dyn BisectorCurve> =
            Handle::new(BisecPC::with_bounds(&c, p1, side, first, last)?).cast();
        let b2: Handle<dyn BisectorCurve> =
            Handle::new(BisecPC::with_bounds(&c, p2, side, first, last)?).cast();
        let mut fi = BisectorFunctionInter::with_curves(&c, &b1, &b2);
        let (mut f, mut d) = (0.0, 0.0);
        let done = fi.value(u, &mut f);
        fi.raised()?;
        flag(done);
        real(f);
        let done = fi.values(u, &mut f, &mut d);
        fi.raised()?;
        flag(done);
        real(f);
        real(d);
        let mut fe = BisectorFunctionInter::new();
        fe.perform(&c, &b2, &b1);
        d = 0.0;
        let done = fe.derivative(u, &mut d);
        fe.raised()?;
        flag(done);
        real(d);
        Ok(())
    });
    end();
}
/// `n (u1 u2 ub d x y)*n u`: Bisector_PolyBis and Bisector_PointOnBis.
pub fn bisec_poly_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let n = r.integer();
        let mut poly = PolyBis::new();
        flag(poly.is_empty());
        for _ in 0..n {
            let (u1, u2, ub, d) = (r.real(), r.real(), r.real(), r.real());
            let p = r.pnt2d();
            poly.append(PointOnBis::new(u1, u2, ub, d, p))?;
        }
        let u = r.real();
        int(poly.length());
        flag(poly.is_empty());
        real(poly.first().param_on_bis());
        real(poly.last().param_on_bis());
        if n >= 2 {
            int(poly.interval(u)?);
        }
        let mut t = Transform2::default();
        t.set_rotation(Point2::new(1., 2.), 0.3);
        poly.transform(t);
        polygon(&poly);
        let mut q = PointOnBis::default();
        flag(q.is_infinite());
        q.set_infinite(true);
        q.set_param_on_c1(u);
        q.set_param_on_c2(2. * u);
        q.set_param_on_bis(3. * u);
        q.set_distance(4.);
        q.set_point(Point2::new(u, u));
        flag(q.is_infinite());
        real(q.param_on_c2());
        captured(|out| {
            q.dump(out);
            Ok(())
        });
        Ok(())
    });
    end();
}
/// `c sign`: Bisector::IsConvex.
pub fn bisec_is_convex_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = curve2d(&mut r)?;
        let s = r.real();
        flag(is_convex(&c, s)?);
        Ok(())
    });
    end();
}
