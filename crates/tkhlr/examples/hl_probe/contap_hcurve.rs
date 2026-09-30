//! Contap's curve-tool surface and its distinct adaptive sample-count formula.
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, show};
use super::curve2d::curve2d;
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::int_start::TheArc;
use rustycad_tkhlr::contap::ContapHCurve2dTool;
use rustycad_tkmath::{geom_abs::Shape, gp::Vector2};
/// Read the bounded or natural arc used by the native tool; `arc(reader)`.
pub fn arc(r: &mut Reader) -> Result<TheArc, Failure> {
    let c = curve2d(r)?;
    if !r.flag() {
        return Ok(Handle::new(Geom2dAdaptorCurve::with_curve(&c)?).cast());
    }
    let a = r.real();
    Ok(Handle::new(Geom2dAdaptorCurve::with_range(&c, a, r.real())?).cast())
}

fn v2d(v: Vector2) {
    real(v.x());
    real(v.y());
}

type C = ContapHCurve2dTool;

fn shape(k: i32) -> Shape {
    [Shape::C0, Shape::G1, Shape::C1, Shape::G2][k as usize]
}

fn hc2d_intervals(a: &TheArc, k: i32) -> Result<(), Failure> {
    let n = C::nb_intervals(a, shape(k))?;
    int(n);
    let mut t: Array1<'static, f64> =
        Array1::with_bounds(1, n + 1).map_err(|e| e.into_failure("NCollection_Array1"))?;
    C::intervals(a, &mut t, shape(k))?;
    (1..=n + 1).for_each(|i| real(t[i]));
    Ok(())
}

fn hc2d_derivatives(a: &TheArc, u: f64) {
    show(C::value(a, u), pnt2d);
    show(C::d0(a, u), pnt2d);
    show(C::d1(a, u), |(p, v)| {
        pnt2d(p);
        v2d(v);
    });
    show(C::d2(a, u), |(p, v, w)| {
        pnt2d(p);
        v2d(v);
        v2d(w);
    });
    show(C::d3(a, u), |(p, v, w, x)| {
        pnt2d(p);
        v2d(v);
        v2d(w);
        v2d(x);
    });
    (1..=3).for_each(|n| show(C::dn(a, u, n), v2d));
}

fn hc2d_conics(a: &TheArc) {
    show(C::line(a), |x| pnt2d(x.location()));
    show(C::circle(a), |x| real(x.radius()));
    show(C::ellipse(a), |x| real(x.minor_radius()));
    show(C::hyperbola(a), |x| real(x.major_radius()));
    show(C::parabola(a), |x| real(x.focal()));
    show(C::bezier(a), |x| {
        flag(x.is_null());
        if let Some(b) = x.get() {
            int(b.nb_poles());
        }
    });
    show(C::bspline(a), |x| {
        flag(x.is_null());
        if let Some(b) = x.get() {
            int(b.nb_knots());
        }
    });
}

/// All original Contap curve-tool calls; `hc2d_case(numbers)`.
pub fn hc2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = arc(&mut r)?;
        let (u, u0, u1) = (r.real(), r.real(), r.real());
        real(C::first_parameter(&a)?);
        real(C::last_parameter(&a)?);
        int(C::continuity(&a)? as i32);
        (0..=3).for_each(|k| attempt(|| hc2d_intervals(&a, k)));
        show(C::is_closed(&a), flag);
        show(C::is_periodic(&a), flag);
        show(C::period(&a), real);
        hc2d_derivatives(&a, u);
        show(C::resolution(&a, 1.0e-3), real);
        int(C::get_type(&a)? as i32);
        hc2d_conics(&a);
        show(C::nb_samples(&a, u0, u1), int);
        Ok(())
    });
    end();
}
