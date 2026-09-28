//! The GCPnts distributions and DistFunction requests of
//! tools/oracle/tkgeombase_probe_gcpnts.cxx.
// The printers run inside `attempt`, whose closure returns `Ok(printer(...))`.
#![allow(clippy::unit_arg)]
use super::gcp_case;
use crate::gb_probe::common::{Reader, attempt, end, flag, int, pnt, real, show};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::gcpnts::{
    QuasiUniformAbscissa, QuasiUniformDeflection, TCurveTypes, TDistFunction, TDistFunctionMV,
    TangentialCriteria, TangentialDeflection, UniformAbscissa, UniformDeflection,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{Function, MultipleVarFunction, Vector};

fn uabs_dump(u: &UniformAbscissa, extra: i32) {
    flag(u.is_done());
    attempt(|| {
        let n: i32 = u.nb_points()?;
        int(n);
        show(u.abscissa(), real);
        for i in 0..=n + extra {
            show(u.parameter(i), real);
        }
        Ok(())
    });
}

// One gcp_uabs run: kinds 0-3 Initialize `u`, 4-7 construct a new object.
fn uabs_step<A: TCurveTypes>(a: &A, u: &mut UniformAbscissa, r: &mut Reader) {
    let (kind, v, u1, u2, tol, extra) = (
        r.integer(),
        r.real(),
        r.real(),
        r.real(),
        r.real(),
        r.integer(),
    );
    let nb: i32 = v as i32;
    if kind < 4 {
        attempt(|| match kind {
            0 => u.initialize_with_abscissa(a, v, tol),
            1 => u.initialize_abscissa_range(a, v, u1, u2, tol),
            2 => u.initialize_with_nb_points(a, nb, tol),
            _ => u.initialize_nb_points_range(a, nb, u1, u2, tol),
        });
        uabs_dump(u, extra);
        return;
    }
    attempt(|| {
        let c = match kind {
            4 => UniformAbscissa::with_abscissa(a, v, tol)?,
            5 => UniformAbscissa::with_abscissa_range(a, v, u1, u2, tol)?,
            6 => UniformAbscissa::with_nb_points(a, nb, tol)?,
            _ => UniformAbscissa::with_nb_points_range(a, nb, u1, u2, tol)?,
        };
        Ok(uabs_dump(&c, 0))
    });
}

fn uabs_run<A: TCurveTypes>(a: &A, r: &mut Reader) {
    let mut u = UniformAbscissa::new();
    uabs_dump(&u, 0);
    for _ in 0..r.integer() {
        uabs_step(a, &mut u, r);
    }
}

/// gcp_uabs.
pub fn uabs_case(values: &[f64]) {
    gcp_case!(values, uabs_run);
}

fn quabs_dump(q: &QuasiUniformAbscissa) {
    flag(q.is_done());
    attempt(|| {
        let n: i32 = q.nb_points()?;
        int(n);
        for i in 0..=n {
            show(q.parameter(i), real);
        }
        Ok(())
    });
}

fn quabs_run<A: TCurveTypes>(a: &A, r: &mut Reader) {
    let mut q = QuasiUniformAbscissa::new();
    quabs_dump(&q);
    for _ in 0..r.integer() {
        let (kind, nb, u1, u2) = (r.integer(), r.integer(), r.real(), r.real());
        if kind < 2 {
            attempt(|| match kind {
                0 => q.initialize(a, nb),
                _ => q.initialize_range(a, nb, u1, u2),
            });
            quabs_dump(&q);
            continue;
        }
        attempt(|| {
            let c = match kind {
                2 => QuasiUniformAbscissa::with_nb_points(a, nb)?,
                _ => QuasiUniformAbscissa::with_range(a, nb, u1, u2)?,
            };
            Ok(quabs_dump(&c))
        });
    }
}

/// gcp_quabs.
pub fn quabs_case(values: &[f64]) {
    gcp_case!(values, quabs_run);
}

/// The members gb_gcp_defl_dump reads from both deflection distributions.
trait DeflectionDump {
    fn done(&self) -> bool;
    fn deflection_value(&self) -> Result<f64, Failure>;
    fn count(&self) -> Result<i32, Failure>;
    fn param(&self, index: i32) -> Result<f64, Failure>;
    fn point(&self, index: i32) -> Result<Point3, Failure>;
}

macro_rules! deflection_dump {
    ($($algo:ty),*) => {$(
        impl DeflectionDump for $algo {
            fn done(&self) -> bool {
                self.is_done()
            }
            fn deflection_value(&self) -> Result<f64, Failure> {
                self.deflection()
            }
            fn count(&self) -> Result<i32, Failure> {
                self.nb_points()
            }
            fn param(&self, index: i32) -> Result<f64, Failure> {
                self.parameter(index)
            }
            fn point(&self, index: i32) -> Result<Point3, Failure> {
                self.value(index)
            }
        }
    )*};
}
deflection_dump!(UniformDeflection, QuasiUniformDeflection);

fn defl_dump(d: &impl DeflectionDump) {
    flag(d.done());
    show(d.deflection_value(), real);
    show(d.point(1), pnt);
    attempt(|| {
        let n: i32 = d.count()?;
        int(n);
        for i in 0..=n {
            show(d.param(i), real);
            show(d.point(i), pnt);
        }
        Ok(())
    });
}

fn udefl_run<A: TCurveTypes>(a: &A, r: &mut Reader) {
    let mut d = UniformDeflection::new();
    defl_dump(&d);
    for _ in 0..r.integer() {
        let (kind, f, ctl, u1, u2) = (r.integer(), r.real(), r.flag(), r.real(), r.real());
        if kind < 2 {
            attempt(|| match kind {
                0 => d.initialize(a, f, ctl),
                _ => d.initialize_range(a, f, u1, u2, ctl),
            });
            defl_dump(&d);
            continue;
        }
        attempt(|| {
            let c = match kind {
                2 => UniformDeflection::with_curve(a, f, ctl)?,
                _ => UniformDeflection::with_range(a, f, u1, u2, ctl)?,
            };
            Ok(defl_dump(&c))
        });
    }
}

/// gcp_udefl.
pub fn udefl_case(values: &[f64]) {
    gcp_case!(values, udefl_run);
}

// `GeomAbs_Shape(int)`.
fn shape(value: i32) -> Shape {
    match value {
        0 => Shape::C0,
        1 => Shape::G1,
        2 => Shape::C1,
        3 => Shape::G2,
        4 => Shape::C2,
        5 => Shape::C3,
        _ => Shape::CN,
    }
}

fn qudefl_run<A: TCurveTypes>(a: &A, r: &mut Reader) {
    let mut d = QuasiUniformDeflection::new();
    defl_dump(&d);
    for _ in 0..r.integer() {
        let (kind, f, s, u1, u2) = (
            r.integer(),
            r.real(),
            shape(r.integer()),
            r.real(),
            r.real(),
        );
        if kind < 2 {
            attempt(|| match kind {
                0 => d.initialize(a, f, s),
                _ => d.initialize_range(a, f, u1, u2, s),
            });
            defl_dump(&d);
            continue;
        }
        attempt(|| {
            let c = match kind {
                2 => QuasiUniformDeflection::with_curve(a, f, s)?,
                _ => QuasiUniformDeflection::with_range(a, f, u1, u2, s)?,
            };
            Ok(defl_dump(&c))
        });
    }
}

/// gcp_qudefl.
pub fn qudefl_case(values: &[f64]) {
    gcp_case!(values, qudefl_run);
}

fn tdefl_dump(t: &TangentialDeflection) {
    let n: i32 = t.nb_points();
    int(n);
    for i in 0..=n + 1 {
        show(t.parameter(i), real);
        show(t.value(i), pnt);
    }
}

// One gcp_tdefl run: kinds 0-1 Initialize `t`, 2-3 construct a new object.
fn tdefl_step<A: TCurveTypes>(a: &A, t: &mut TangentialDeflection, r: &mut Reader) {
    let (kind, angular, curvature) = (r.integer(), r.real(), r.real());
    let (points, u_tol, min_len, u1, u2) = (r.integer(), r.real(), r.real(), r.real(), r.real());
    let criteria = TangentialCriteria {
        angular_deflection: angular,
        curvature_deflection: curvature,
        minimum_of_points: points,
        u_tol,
        min_len,
    };
    if kind < 2 {
        attempt(|| match kind {
            0 => t.initialize(a, criteria),
            _ => t.initialize_range(a, u1, u2, criteria),
        });
        tdefl_dump(t);
        return;
    }
    attempt(|| {
        let c = match kind {
            2 => TangentialDeflection::with_curve(a, criteria)?,
            _ => TangentialDeflection::with_range(a, u1, u2, criteria)?,
        };
        Ok(tdefl_dump(&c))
    });
}

fn tdefl_run<A: TCurveTypes>(a: &A, r: &mut Reader) {
    let mut t = TangentialDeflection::new();
    tdefl_dump(&t);
    for _ in 0..r.integer() {
        tdefl_step(a, &mut t, r);
    }
    for _ in 0..r.integer() {
        let (p, u, replace) = (r.pnt(), r.real(), r.flag());
        show(t.add_point(p, u, replace), int);
    }
    tdefl_dump(&t);
}

/// gcp_tdefl.
pub fn tdefl_case(values: &[f64]) {
    gcp_case!(values, tdefl_run);
}

/// gcp_arc_step.
pub fn arc_step_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (rad, lin, ang, ml) = (r.real(), r.real(), r.real(), r.real());
    show(
        TangentialDeflection::arc_angular_step(rad, lin, ang, ml),
        real,
    );
    end();
}

fn print_value(ok: bool, v: f64) {
    flag(ok);
    if ok {
        real(v);
    }
}

// The kept curve failure is raised right after each `Value`, where the native
// exception would leave the call.
fn dist_body<A: TCurveTypes>(a: &A, (u1, u2): (f64, f64), xs: &[f64]) -> Result<(), Failure> {
    let mut f = TDistFunction::new(a, u1, u2)?;
    let mut v: f64 = 0.0;
    for &x in xs {
        let ok: bool = f.value(x, &mut v);
        f.take_failure()?;
        print_value(ok, v);
    }
    int(TDistFunctionMV::new(&mut f).variables());
    let mut point: Vector<'static> = Vector::with_bounds(1, 1)?;
    for &x in xs {
        point[1] = x;
        let ok: bool = TDistFunctionMV::new(&mut f).value(&point, &mut v);
        f.take_failure()?;
        print_value(ok, v);
    }
    Ok(())
}

fn dist_run<A: TCurveTypes>(a: &A, r: &mut Reader) {
    let (u1, u2, n) = (r.real(), r.real(), r.integer());
    let xs: Vec<f64> = (0..n).map(|_| r.real()).collect();
    attempt(|| dist_body(a, (u1, u2), &xs));
}

/// gcp_dist.
pub fn dist_case(values: &[f64]) {
    gcp_case!(values, dist_run);
}
