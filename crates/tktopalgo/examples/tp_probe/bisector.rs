//! Bisector probe operations, the Rust twin of tools/oracle/tktopalgo_probe_bisector.cxx.
//! The curves are read by crates/tkgeomalgo/examples/ga_probe/curve2d.rs (ga_curve2d).
use super::common::{Reader, attempt, captured, end, flag, int, pnt2d, real, text, xy};
use super::curve2d::curve2d;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dCartesianPoint, Geom2dCurve, Geom2dGeometry, Geom2dPoint, Geom2dTrimmedCurve,
};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tktopalgo::bisector::{Bisec, BisecAna, BisectorCurve};

pub(super) fn vec(v: Vector2) {
    xy(v.xy());
}
pub(super) fn join(k: i32) -> JoinType {
    match k {
        0 => JoinType::Arc,
        1 => JoinType::Tangent,
        _ => JoinType::Intersection,
    }
}
fn type_name<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(h: &Handle<T>) {
    text(h.dynamic_type().map_or("", |t| t.name()).as_bytes());
}
/// `tp_span`: an infinite bound is replaced 10 away from the other.
pub(super) fn span(mut a: f64, mut b: f64) -> (f64, f64) {
    if b > 1e10 {
        b = a + 10.;
    }
    if b < -1e10 {
        b = a - 10.;
    }
    if a < -1e10 {
        a = b - 10.;
    }
    if a > 1e10 {
        a = b + 10.;
    }
    (a, b)
}
pub(super) fn sample(a: f64, b: f64, k: i32) -> f64 {
    a + (b - a) * (f64::from(k) * 0.25)
}
fn point(r: &mut Reader<'_>) -> Handle<dyn Geom2dPoint> {
    Handle::new(Geom2dCartesianPoint::from_pnt2d(r.pnt2d())).cast()
}
pub(super) fn bisector_curve(bc: &Handle<dyn BisectorCurve>) {
    int(bc.nb_intervals());
    for i in 1..=bc.nb_intervals() {
        attempt(|| {
            real(bc.interval_first(i)?);
            real(bc.interval_last(i)?);
            Ok(())
        });
    }
    flag(bc.is_extend_at_start());
    flag(bc.is_extend_at_end());
}
pub(super) fn samples(c: &Handle<dyn Geom2dCurve>, bc: &Handle<dyn BisectorCurve>, a: f64, b: f64) {
    let (a, b) = span(a, b);
    for k in 0..=4 {
        let u = sample(a, b, k);
        attempt(|| {
            let p = c.value(u)?;
            pnt2d(p);
            if bc.is_some() {
                attempt(|| {
                    real(bc.parameter(p)?);
                    Ok(())
                });
            }
            Ok(())
        });
        attempt(|| {
            let (p, v) = c.d1(u)?;
            pnt2d(p);
            vec(v);
            Ok(())
        });
    }
}
pub(super) fn trimmed(t: &Handle<Geom2dTrimmedCurve>) {
    if t.is_null() {
        int(0);
        return;
    }
    int(1);
    let basis = t.basis_curve();
    type_name(&basis);
    real(t.first_parameter());
    real(t.last_parameter());
    let bc: Handle<dyn BisectorCurve> = basis.cast();
    bisector_curve(&bc);
    if let Some(ana) = basis.cast::<BisecAna>().get() {
        type_name(&ana.geom2d_curve());
    }
    samples(&t.cast(), &bc, t.first_parameter(), t.last_parameter());
}
pub(super) struct Passage {
    pub p: Point2,
    pub v1: Vector2,
    pub v2: Vector2,
}
fn read_passage(r: &mut Reader<'_>) -> Passage {
    let p = r.pnt2d();
    let a = r.pnt2d();
    let b = r.pnt2d();
    Passage {
        p,
        v1: Vector2::from(a.xy()),
        v2: Vector2::from(b.xy()),
    }
}
pub(super) fn curves_passage(
    r: &mut Reader<'_>,
    automatic: bool,
    c1: &Handle<dyn Geom2dCurve>,
    c2: &Handle<dyn Geom2dCurve>,
) -> Result<Passage, Failure> {
    if !automatic {
        return Ok(read_passage(r));
    }
    let p = c2.value(c2.first_parameter())?;
    let v1 = c1.dn(c1.last_parameter(), 1)?;
    let v2 = c2.dn(c2.first_parameter(), 1)?.reversed();
    Ok(Passage { p, v1, v2 })
}
pub(super) fn curve_passage(
    r: &mut Reader<'_>,
    automatic: bool,
    c: &Handle<dyn Geom2dCurve>,
    at_end: bool,
) -> Result<Passage, Failure> {
    if !automatic {
        return Ok(read_passage(r));
    }
    let u = if at_end {
        c.last_parameter()
    } else {
        c.first_parameter()
    };
    let p = c.value(u)?;
    let v1 = c.dn(u, 1)?;
    Ok(Passage {
        p,
        v1,
        v2: v1.reversed(),
    })
}
/// `auto c1 c2 sense join tol oncurve [p v1 v2]`: Bisector_Bisec::Perform of two curves.
pub fn bisec_curves_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let automatic = r.flag();
        let c1 = curve2d(&mut r)?;
        let c2 = curve2d(&mut r)?;
        let (sense, jt, tol, oncurve) = (r.real(), r.integer(), r.real(), r.flag());
        let s = curves_passage(&mut r, automatic, &c1, &c2)?;
        let mut b = Bisec::new();
        b.perform_curves(&c1, &c2, s.p, s.v1, s.v2, sense, join(jt), tol, oncurve)?;
        trimmed(&b.value());
        Ok(())
    });
    end();
}
/// `which auto c x y sense tol oncurve [p v1 v2]`: Bisector_Bisec::Perform of a curve and a point.
pub fn bisec_curve_point_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let which = r.integer();
        let automatic = r.flag();
        let c = curve2d(&mut r)?;
        let pnt = point(&mut r);
        let (sense, tol, oncurve) = (r.real(), r.real(), r.flag());
        let s = curve_passage(&mut r, automatic, &c, which == 0)?;
        let mut b = Bisec::new();
        if which == 0 {
            b.perform_curve_point(&c, &pnt, s.p, s.v1, s.v2, sense, tol, oncurve)?;
        } else {
            b.perform_point_curve(&pnt, &c, s.p, s.v1, s.v2, sense, tol, oncurve)?;
        }
        trimmed(&b.value());
        Ok(())
    });
    end();
}
/// `x1 y1 x2 y2 p v1 v2 sense tol oncurve`: Bisector_Bisec::Perform of two points.
pub fn bisec_points_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let p1 = point(&mut r);
        let p2 = point(&mut r);
        let s = read_passage(&mut r);
        let (sense, tol, oncurve) = (r.real(), r.real(), r.flag());
        let mut b = Bisec::new();
        b.perform_points(&p1, &p2, s.p, s.v1, s.v2, sense, tol, oncurve)?;
        trimmed(&b.value());
        Ok(())
    });
    end();
}
fn ana_members(a: &Handle<BisecAna>) {
    attempt(|| {
        type_name(&a.geom2d_curve());
        Ok(())
    });
    real(a.parameter_of_start_point());
    real(a.parameter_of_end_point());
    real(a.first_parameter());
    real(a.last_parameter());
    let bc: Handle<dyn BisectorCurve> = a.cast();
    bisector_curve(&bc);
    attempt(|| {
        real(a.interval_first(2)?);
        Ok(())
    });
    attempt(|| {
        flag(a.is_closed()?);
        Ok(())
    });
    flag(a.is_periodic());
    int(a.continuity() as i32);
    attempt(|| {
        flag(a.is_cn(2)?);
        Ok(())
    });
    attempt(|| {
        real(a.reversed_parameter(0.5)?);
        Ok(())
    });
    samples(&a.cast(), &bc, a.first_parameter(), a.last_parameter());
    attempt(|| {
        let c: Handle<BisecAna> = a.copy()?.cast();
        real(c.first_parameter());
        real(c.last_parameter());
        Ok(())
    });
    captured(|out| {
        a.dump(0, 2, out);
        Ok(())
    });
    attempt(|| {
        a.reverse()?;
        real(a.first_parameter());
        real(a.last_parameter());
        Ok(())
    });
}
/// `which ...`: Bisector_BisecAna::Perform and its members.
pub fn bisec_ana_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let which = r.integer();
        let a: Handle<BisecAna> = Handle::new(BisecAna::new());
        if which == 0 {
            let automatic = r.flag();
            let c1 = curve2d(&mut r)?;
            let c2 = curve2d(&mut r)?;
            let (sense, jt, tol, oncurve) = (r.real(), r.integer(), r.real(), r.flag());
            let s = curves_passage(&mut r, automatic, &c1, &c2)?;
            a.perform_curves(&c1, &c2, s.p, s.v1, s.v2, sense, join(jt), tol, oncurve)?;
        } else if which == 3 {
            let p1 = point(&mut r);
            let p2 = point(&mut r);
            let s = read_passage(&mut r);
            let (sense, tol, oncurve) = (r.real(), r.real(), r.flag());
            a.perform_points(&p1, &p2, s.p, s.v1, s.v2, sense, tol, oncurve)?;
        } else {
            let automatic = r.flag();
            let c = curve2d(&mut r)?;
            let pnt = point(&mut r);
            let (sense, tol, oncurve) = (r.real(), r.real(), r.flag());
            let s = curve_passage(&mut r, automatic, &c, which == 1)?;
            if which == 1 {
                a.perform_curve_point(&c, &pnt, s.p, s.v1, s.v2, sense, tol, oncurve)?;
            } else {
                a.perform_point_curve(&pnt, &c, s.p, s.v1, s.v2, sense, tol, oncurve)?;
            }
        }
        ana_members(&a);
        Ok(())
    });
    end();
}
