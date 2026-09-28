//! The CPnts and GCPnts probe; the counterpart of tools/oracle/tkgeombase_probe_gcpnts.cxx
//! (same request layout, member order and token stream).
// The printers run inside `attempt`, whose closure returns `Ok(printer(...))`.
#![allow(clippy::unit_arg)]
mod distributions;

pub use distributions::{
    arc_step_case, dist_case, quabs_case, qudefl_case, tdefl_case, uabs_case, udefl_case,
};

use super::common::{Reader, attempt, flag, int, pnt, real, show};
use rustycad_tkgeombase::cpnts::{self, PntsCurve};
use rustycad_tkgeombase::gcpnts::{AbscissaPoint, TCurveTypes};

/// gb_gcp_curve: build the adaptor (dimension, range flag and bounds, curve spec) and
/// call `$run(&adaptor, &mut reader)`; a construction failure prints its type.
macro_rules! gcp_case {
    ($values:expr, $run:path) => {{
        use crate::gb_probe::lprop::build;
        use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
        use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
        let mut r = crate::gb_probe::common::Reader::new($values);
        let (dim, ranged) = (r.integer(), r.flag());
        let (u1, u2) = if ranged {
            (r.real(), r.real())
        } else {
            (0.0, 0.0)
        };
        crate::gb_probe::common::attempt(|| {
            if dim == 2 {
                let c = build::curve2d(&mut r)?;
                let a = if ranged {
                    Geom2dAdaptorCurve::with_range(&c, u1, u2)?
                } else {
                    Geom2dAdaptorCurve::with_curve(&c)?
                };
                $run(&a, &mut r);
                return Ok(());
            }
            let c = build::curve3d(&mut r)?;
            let a = if ranged {
                GeomAdaptorCurve::with_range(&c, u1, u2)?
            } else {
                GeomAdaptorCurve::with_curve(&c)?
            };
            $run(&a, &mut r);
            Ok(())
        });
        crate::gb_probe::common::end();
    }};
}
pub(crate) use gcp_case;

fn length_run<A: TCurveTypes>(a: &A, r: &mut Reader) {
    let (u1, u2, tol) = (r.real(), r.real(), r.real());
    let c: PntsCurve = a.pnts_curve();
    show(cpnts::AbscissaPoint::length(c), real);
    show(cpnts::AbscissaPoint::length_with_tolerance(c, tol), real);
    show(cpnts::AbscissaPoint::length_between(c, u1, u2), real);
    show(
        cpnts::AbscissaPoint::length_between_with_tolerance(c, u1, u2, tol),
        real,
    );
    show(AbscissaPoint::length(a), real);
    show(AbscissaPoint::length_with_tolerance(a, tol), real);
    show(AbscissaPoint::length_between(a, u1, u2), real);
    show(
        AbscissaPoint::length_between_with_tolerance(a, u1, u2, tol),
        real,
    );
}

/// gcp_length.
pub fn length_case(values: &[f64]) {
    gcp_case!(values, length_run);
}

fn cpnts_point(p: &cpnts::AbscissaPoint) {
    flag(p.is_done());
    show(p.parameter(), real);
}

fn cpnts_abscissa_run<A: TCurveTypes>(a: &A, r: &mut Reader) {
    let (ab, u0, ui, res) = (r.real(), r.real(), r.real(), r.real());
    let (u1, u2, tol) = (r.real(), r.real(), r.real());
    let c: PntsCurve = a.pnts_curve();
    attempt(|| {
        Ok(cpnts_point(&cpnts::AbscissaPoint::with_resolution(
            c, ab, u0, res,
        )?))
    });
    attempt(|| {
        Ok(cpnts_point(&cpnts::AbscissaPoint::with_start(
            c, ab, u0, ui, res,
        )?))
    });
    let mut p = cpnts::AbscissaPoint::new();
    cpnts_point(&p);
    attempt(|| {
        p.init(c)?;
        p.perform(ab, u0, res)?;
        Ok(cpnts_point(&p))
    });
    attempt(|| {
        p.init_with_tolerance(c, tol)?;
        p.adv_perform(ab, u0, ui, res)?;
        Ok(cpnts_point(&p))
    });
    attempt(|| {
        p.init_between(c, u1, u2)?;
        p.perform_from(ab, u0, ui, res)?;
        Ok(cpnts_point(&p))
    });
    attempt(|| {
        p.init_between_with_tolerance(c, u1, u2, tol)?;
        p.adv_perform(ab, u0, ui, res)?;
        Ok(cpnts_point(&p))
    });
    p.set_parameter(u1);
    cpnts_point(&p);
}

/// cpnts_abscissa.
pub fn cpnts_abscissa_case(values: &[f64]) {
    gcp_case!(values, cpnts_abscissa_run);
}

fn cpnts_iterate(it: &mut cpnts::UniformDeflection, cap: i32) {
    flag(it.is_all_done());
    let mut n: i32 = 0;
    attempt(|| {
        while it.more()? && n < cap {
            real(it.value()?);
            pnt(it.point()?);
            n += 1;
            it.next()?;
        }
        Ok(())
    });
    int(n);
}

fn cpnts_udefl_run<A: TCurveTypes>(a: &A, r: &mut Reader) {
    let (d, res, ctl, u1, u2, cap) = (
        r.real(),
        r.real(),
        r.flag(),
        r.real(),
        r.real(),
        r.integer(),
    );
    let c: PntsCurve = a.pnts_curve();
    attempt(|| {
        Ok(cpnts_iterate(
            &mut cpnts::UniformDeflection::with_curve(c, d, res, ctl)?,
            cap,
        ))
    });
    attempt(|| {
        let mut it = cpnts::UniformDeflection::with_range(c, d, u1, u2, res, ctl)?;
        Ok(cpnts_iterate(&mut it, cap))
    });
    let mut it = cpnts::UniformDeflection::new();
    flag(it.is_all_done());
    show(it.more(), flag);
    show(it.value(), real);
    attempt(|| it.next());
    attempt(|| {
        it.initialize_range(c, d, u1, u2, res, ctl)?;
        Ok(cpnts_iterate(&mut it, cap))
    });
    attempt(|| {
        it.initialize(c, d, res, ctl)?;
        Ok(cpnts_iterate(&mut it, cap))
    });
}

/// cpnts_udefl.
pub fn cpnts_udefl_case(values: &[f64]) {
    gcp_case!(values, cpnts_udefl_run);
}

fn gcp_point(p: &AbscissaPoint) {
    flag(p.is_done());
    show(p.parameter(), real);
}

fn gcp_abscissa_run<A: TCurveTypes>(a: &A, r: &mut Reader) {
    let (ab, u0, ui, tol) = (r.real(), r.real(), r.real(), r.real());
    gcp_point(&AbscissaPoint::new());
    attempt(|| Ok(gcp_point(&AbscissaPoint::from_abscissa(a, ab, u0)?)));
    attempt(|| Ok(gcp_point(&AbscissaPoint::with_tolerance(tol, a, ab, u0)?)));
    attempt(|| Ok(gcp_point(&AbscissaPoint::from_start(a, ab, u0, ui)?)));
    attempt(|| {
        Ok(gcp_point(&AbscissaPoint::from_start_with_tolerance(
            a, ab, u0, ui, tol,
        )?))
    });
}

/// gcp_abscissa.
pub fn gcp_abscissa_case(values: &[f64]) {
    gcp_case!(values, gcp_abscissa_run);
}
