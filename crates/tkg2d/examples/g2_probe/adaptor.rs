//! Adaptor2d and Geom2dAdaptor adaptors: construction, summary and MakeCurve; the
//! counterpart of tools/oracle/tkg2d_probe_adaptor.cxx (a2_adaptor, a2_summary...).
use super::common::{Reader, attempt, ax, ax22, curve, flag, int, pnt, real, show, summary, vec};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::{
    Adaptor2dCurve2d, Adaptor2dCurve2dBase, Adaptor2dLine2d, Adaptor2dOffsetCurve,
};
use rustycad_tkg2d::geom2d_adaptor::{Geom2dAdaptorCurve, make_curve};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Line2;

fn line_adaptor(kind: i32, r: &mut Reader) -> Handle<dyn Adaptor2dCurve2d> {
    let (p, d) = (r.pnt(), r.dir());
    if kind == 0 {
        let u1 = r.real();
        return Handle::new(Adaptor2dLine2d::new(p, d, u1, r.real())).cast();
    }
    let line = Adaptor2dLine2d::default();
    if kind == 1 {
        line.load(Line2::new(p, d));
    } else {
        let u1 = r.real();
        line.load_bounded(Line2::new(p, d), u1, r.real());
    }
    Handle::new(line).cast()
}

/// One adaptor specification, as a2_adaptor reads it.
pub fn adaptor(r: &mut Reader) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
    let kind = r.integer();
    Ok(match kind {
        0..=2 => line_adaptor(kind, r),
        3 => Handle::new(Geom2dAdaptorCurve::with_curve(&curve(r)?)?).cast(),
        4 => {
            let (u1, u2) = (r.real(), r.real());
            Handle::new(Geom2dAdaptorCurve::with_range(&curve(r)?, u1, u2)?).cast()
        }
        5 => Handle::new(Adaptor2dOffsetCurve::with_curve(&adaptor(r)?)).cast(),
        6 => {
            let d = r.real();
            Handle::new(Adaptor2dOffsetCurve::with_offset(&adaptor(r)?, d)?).cast()
        }
        7 => {
            let (d, w1, w2) = (r.real(), r.real(), r.real());
            Handle::new(Adaptor2dOffsetCurve::with_bounds(&adaptor(r)?, d, w1, w2)).cast()
        }
        8 => Handle::new(Adaptor2dCurve2dBase::default()).cast(),
        _ => panic!("unknown adaptor kind {kind}"),
    })
}

/// Reads an adaptor, reporting a construction failure and ending the line (a2_built).
pub fn built(r: &mut Reader) -> Option<Handle<dyn Adaptor2dCurve2d>> {
    match adaptor(r) {
        Ok(adaptor) => Some(adaptor),
        Err(error) => {
            println!("{}", error.exception_type());
            None
        }
    }
}

const SHAPES: [Shape; 7] = [
    Shape::C0,
    Shape::G1,
    Shape::C1,
    Shape::G2,
    Shape::C2,
    Shape::C3,
    Shape::CN,
];

/// NbIntervals and Intervals for every continuity (a2_intervals).
pub fn intervals(a: &dyn Adaptor2dCurve2d) {
    for shape in SHAPES {
        let n = match a.nb_intervals(shape) {
            Ok(n) => n,
            Err(error) => {
                print!("{} ", error.exception_type());
                continue;
            }
        };
        int(n);
        let mut t = Array1::<f64>::with_bounds(1, n + 1).unwrap();
        attempt(|| {
            a.intervals(&mut t, shape)?;
            int(t.lower());
            int(t.upper());
            t.iter().for_each(|v| real(*v));
            Ok(())
        });
    }
}

fn null_or_poles(result: Result<Option<i32>, Failure>) {
    show(result, |poles| {
        flag(poles.is_none());
        if let Some(count) = poles {
            int(count);
        }
    });
}

fn conic_kinds(a: &dyn Adaptor2dCurve2d) {
    show(a.line(), |l| ax(l.position()));
    show(a.circle(), |c| {
        ax22(c.position());
        real(c.radius());
    });
    show(a.ellipse(), |e| {
        ax22(e.axis());
        real(e.major_radius());
        real(e.minor_radius());
    });
    show(a.hyperbola(), |h| {
        ax22(h.axis());
        real(h.major_radius());
        real(h.minor_radius());
    });
    show(a.parabola(), |p| {
        ax22(p.axis());
        real(p.focal());
    });
}

/// The kind queries (a2_kinds).
pub fn kinds(a: &dyn Adaptor2dCurve2d) {
    show(a.get_type(), |t| int(t as i32));
    conic_kinds(a);
    show(a.degree(), int);
    show(a.is_rational(), flag);
    show(a.nb_poles(), int);
    show(a.nb_knots(), int);
    show(a.nb_samples(), int);
    null_or_poles(a.bezier().map(|b| b.get().map(|b| b.nb_poles())));
    null_or_poles(a.bspline().map(|b| b.get().map(|b| b.nb_poles())));
}

fn eval_members(a: &dyn Adaptor2dCurve2d, u: f64) {
    show(a.eval_d0(u), pnt);
    show(a.eval_d1(u), |e| {
        pnt(e.point);
        vec(e.d1);
    });
    show(a.eval_d2(u), |e| {
        pnt(e.point);
        [e.d1, e.d2].into_iter().for_each(vec);
    });
    show(a.eval_d3(u), |e| {
        pnt(e.point);
        [e.d1, e.d2, e.d3].into_iter().for_each(vec);
    });
    for n in 0..=5 {
        show(a.eval_dn(u, n), vec);
    }
}

/// The evaluators at one parameter (a2_evaluations).
pub fn evaluations(a: &dyn Adaptor2dCurve2d, u: f64) {
    eval_members(a, u);
    show(a.value(u), pnt);
    show(a.d0(u), pnt);
    show(a.d1(u), |(p, v1)| {
        pnt(p);
        vec(v1);
    });
    show(a.d2(u), |(p, v1, v2)| {
        pnt(p);
        [v1, v2].into_iter().for_each(vec);
    });
    show(a.d3(u), |(p, v1, v2, v3)| {
        pnt(p);
        [v1, v2, v3].into_iter().for_each(vec);
    });
    show(a.dn(u, 2), vec);
}

/// The adaptor summary (a2_summary).
pub fn adaptor_summary(a: &Handle<dyn Adaptor2dCurve2d>, params: &[f64]) {
    print!("{} ", a.dynamic_type().unwrap().name());
    show(a.first_parameter(), real);
    show(a.last_parameter(), real);
    show(a.continuity(), |c| int(c as i32));
    intervals(&**a);
    show(a.is_closed(), flag);
    show(a.is_periodic(), flag);
    show(a.period(), real);
    for r in [1e-7, 1e-3, 0.5, 10.0] {
        show(a.resolution(r), real);
    }
    kinds(&**a);
    for &u in params {
        evaluations(&**a, u);
    }
}

/// Geom2dAdaptor::MakeCurve and the summary of its result (a2_make_curve).
pub fn make_curve_summary(a: &dyn Adaptor2dCurve2d, params: &[f64]) {
    show(make_curve(a), |c| {
        flag(c.is_null());
        if let Some(built) = c.get() {
            print!("{} ", c.dynamic_type().unwrap().name());
            summary(built, params);
        }
    });
}
