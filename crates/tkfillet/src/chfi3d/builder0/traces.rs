// Rust translation of ChFi3d_IntTraces with recadre (ChFi3d_Builder_0.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The meeting of the traces of two fillets on their common face.
use crate::chfi_ds::{SurfDataHandle, lock};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_res2d::IntersectionPoint;
use rustycad_tkmath::gp::Point2;

/// `recadre(p, ref, sens, first, last)`: the parameter moved by one period toward
/// `sens` when it comes nearer to the reference.
fn recadre(p: f64, reference: f64, sens: i32, first: f64, last: f64) -> f64 {
    let pp = p + if sens > 0 { first - last } else { last - first };
    if (pp - reference).abs() < (p - reference).abs() {
        pp
    } else {
        p
    }
}

/// One trace of `ChFi3d_IntTraces`: the pcurve on the face, its periodicity and range
/// (of the basis of a periodic trimmed pcurve), and its adaptor (the whole periodic
/// basis, else the range of the interference enlarged when asked).
struct Trace {
    original: Handle<dyn Geom2dCurve>,
    periodic: bool,
    first: f64,
    last: f64,
    adaptor: Geom2dAdaptorCurve,
}

/// The trace of the interference `jf` of the fillet; `None` when it is too short or has
/// no pcurve on the face.
fn trace(fd: &SurfDataHandle, jf: i32, enlarge: bool) -> Result<Option<Trace>, Failure> {
    let fd = lock(fd);
    let fi = fd.interference(jf);
    let (first, last) = (fi.first_parameter(), fi.last_parameter());
    if last - first < precision::p_confusion() {
        return Ok(None);
    }
    // std::min(0.1, 5%): 0.1 when the share is NaN.
    let share = 0.05 * (last - first);
    let delta = if !enlarge {
        0.0
    } else if share < 0.1 {
        share
    } else {
        0.1
    };
    let original = fi.pcurve_on_face().clone();
    let Some(curve) = original.get() else {
        return Ok(None);
    };
    let periodic = curve.is_periodic();
    let mut pcf = original.clone();
    let adaptor = if periodic {
        if let Some(basis) = original
            .cast::<Geom2dTrimmedCurve>()
            .get()
            .map(|t| t.basis_curve())
        {
            pcf = basis;
        }
        Geom2dAdaptorCurve::with_curve(&pcf)?
    } else {
        Geom2dAdaptorCurve::with_range(&original, first - delta, last + delta)?
    };
    let pcf_curve = pcf.get().ok_or_else(|| {
        Failure::with_message("Standard_NullObject", "ChFi3d_IntTraces: null basis")
    })?;
    Ok(Some(Trace {
        original: original.clone(),
        periodic,
        first: pcf_curve.first_parameter(),
        last: pcf_curve.last_parameter(),
        adaptor,
    }))
}

/// The references, senses and the 2d reference point of the choice among the points.
struct Choice {
    pref: [f64; 2],
    sens: [i32; 2],
    ref_p2d: Point2,
    check_2d: bool,
}

/// The current solution: the parameters on both traces and the 2d point.
struct Solution {
    p: [f64; 2],
    p2d: Point2,
}

impl Solution {
    fn nearer_2d(&self, choice: &Choice, int2d: &IntersectionPoint) -> bool {
        choice.check_2d
            && choice.ref_p2d.distance(int2d.value()) < choice.ref_p2d.distance(self.p2d)
    }
}

/// The candidate `int2d` replacing the solution along a periodic trace `k` (0 or 1):
/// when it is nearer to its reference after recadre, or nearer to the 2d reference.
fn periodic_candidate(
    sol: &mut Solution,
    int2d: &IntersectionPoint,
    k: usize,
    traces: &[Trace; 2],
    choice: &Choice,
) {
    let params = [int2d.param_on_first(), int2d.param_on_second()];
    let other = 1 - k;
    let pk = recadre(
        params[k],
        choice.pref[k],
        choice.sens[k],
        traces[k].first,
        traces[k].last,
    );
    if (pk - choice.pref[k]).abs() < (sol.p[k] - choice.pref[k]).abs() {
        sol.p[k] = pk;
        sol.p[other] = params[other];
        sol.p2d = int2d.value();
    } else if sol.nearer_2d(choice, int2d) {
        let mut po = params[other];
        if traces[other].periodic {
            po = recadre(
                po,
                choice.pref[other],
                choice.sens[other],
                traces[other].first,
                traces[other].last,
            );
        }
        sol.p[k] = pk;
        sol.p[other] = po;
        sol.p2d = int2d.value();
    }
}

/// The candidate `int2d` on non-periodic traces: to the left of both modulo the senses,
/// nearer to both references, or nearer to the 2d reference.
fn plain_candidate(sol: &mut Solution, int2d: &IntersectionPoint, choice: &Choice) {
    let (q1, q2) = (int2d.param_on_first(), int2d.param_on_second());
    let left = (q1 - sol.p[0]) * f64::from(choice.sens[0]) < 0.0
        && (q2 - sol.p[1]) * f64::from(choice.sens[1]) < 0.0;
    let nearer = (q1 - choice.pref[0]).abs() < (sol.p[0] - choice.pref[0]).abs()
        && (q2 - choice.pref[1]).abs() < (sol.p[1] - choice.pref[1]).abs();
    if left || nearer || sol.nearer_2d(choice, int2d) {
        sol.p = [q1, q2];
        sol.p2d = int2d.value();
    }
}

/// The chosen point among the points of the intersection (periodic traces: nearest to
/// the reference; otherwise the leftmost modulo the senses).
fn choose(
    intersection: &GInter,
    traces: &[Trace; 2],
    choice: &Choice,
) -> Result<[f64; 2], Failure> {
    let int2d = intersection.point(1)?;
    let mut sol = Solution {
        p: [int2d.param_on_first(), int2d.param_on_second()],
        p2d: int2d.value(),
    };
    for (k, trace) in traces.iter().enumerate() {
        if trace.periodic {
            sol.p[k] = recadre(
                sol.p[k],
                choice.pref[k],
                choice.sens[k],
                trace.first,
                trace.last,
            );
        }
    }
    for i in 2..=intersection.nb_points()? {
        let int2d = intersection.point(i)?;
        if traces[0].periodic {
            periodic_candidate(&mut sol, &int2d, 0, traces, choice);
        } else if traces[1].periodic {
            periodic_candidate(&mut sol, &int2d, 1, traces, choice);
        } else {
            plain_candidate(&mut sol, &int2d, choice);
        }
    }
    Ok(sol.p)
}

/// One fillet trace of `ChFi3d_IntTraces`: the fillet, its interference on the face,
/// the reference parameter and the sense.
pub type TraceOf<'a> = (&'a SurfDataHandle, i32, f64, i32);

/// `ChFi3d_IntTraces(fd1, pref1, p1, jf1, sens1, fd2, pref2, p2, jf2, sens2, RefP2d,
/// Check2dDistance, enlarge)`: `(p1, p2)`, the parameters on the traces of the two
/// fillets on their common face (`jf` the interference, `pref` the reference, `sens` the
/// side) where they meet, the pcurves enlarged when asked and periodic ones taken
/// whole; `None` when the traces do not meet (the caller's values stay);
/// `int_traces((&fd1, 1, 0.0, 1), (&fd2, 2, 0.0, -1), (p2d, false), true)?`.
pub fn int_traces(
    (fd1, jf1, pref1, sens1): TraceOf<'_>,
    (fd2, jf2, pref2, sens2): TraceOf<'_>,
    (ref_p2d, check_2d): (Point2, bool),
    enlarge: bool,
) -> Result<Option<(f64, f64)>, Failure> {
    let Some(t1) = trace(fd1, jf1, enlarge)? else {
        return Ok(None);
    };
    let Some(t2) = trace(fd2, jf2, enlarge)? else {
        return Ok(None);
    };
    let tol = precision::p_intersection();
    let mut intersection = GInter::new();
    if t1.original == t2.original {
        intersection.perform_self(&t1.adaptor, tol, tol)?;
    } else {
        intersection.perform_between(&t1.adaptor, &t2.adaptor, tol, tol)?;
    }
    if !intersection.is_done() || intersection.is_empty()? || intersection.nb_points()? < 1 {
        return Ok(None);
    }
    let choice = Choice {
        pref: [pref1, pref2],
        sens: [sens1, sens2],
        ref_p2d,
        check_2d,
    };
    let [p1, p2] = choose(&intersection, &[t1, t2], &choice)?;
    Ok(Some((p1, p2)))
}
