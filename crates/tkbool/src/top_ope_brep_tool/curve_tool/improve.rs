// Rust translation of the CurveImprovement block of TopOpeBRepTool_CurveTool::MakeCurves
// (ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_CurveTool.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The "improvement" of intersection curves of more than 100 poles: the poles too
//! close to the previous kept one (or aligned with it and the next) are dropped and
//! the input curves replaced by degree-1 B-splines of the others.
use super::array_failure;
use super::make_curves::SectionCurves;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkgeombase::gce::{MakeLin, MakeLin2d, Root};
use rustycad_tkmath::gp::{Point2, Point3};

/// The poles of the three curves (the p-curves only when computed).
pub(super) struct Poles {
    pub c3d: Vec<Point3>,
    pub pc1: Option<Vec<Point2>>,
    pub pc2: Option<Vec<Point2>>,
}

/// The square tolerances of the pole filter: 3d, 2d, and the deflections.
struct Tolerances {
    tol: f64,
    tl2d: f64,
    def: f64,
    def2d: f64,
}

/// The last kept poles.
#[derive(Clone, Copy)]
struct Kept {
    p: Point3,
    p1: Option<Point2>,
    p2: Option<Point2>,
}

impl Kept {
    /// The poles at `ip`.
    fn at(poles: &Poles, ip: usize) -> Self {
        Self {
            p: poles.c3d[ip],
            p1: poles.pc1.as_ref().map(|v| v[ip]),
            p2: poles.pc2.as_ref().map(|v| v[ip]),
        }
    }
}

/// The square distance between two optional 2d points (0 without p-curve).
fn sq2d(a: Option<Point2>, b: Option<Point2>) -> f64 {
    match (a, b) {
        (Some(a), Some(b)) => a.square_distance(b),
        _ => 0.,
    }
}

/// The square distance of a 2d pole to the line from the kept pole to the next (0
/// without p-curve).
fn sq_line_2d(
    kept: Option<Point2>,
    next: Option<Point2>,
    pole: Option<Point2>,
) -> Result<f64, Failure> {
    match (kept, next, pole) {
        (Some(k), Some(n), Some(p)) => {
            Ok(MakeLin2d::from_points(k, n)?.value()?.square_distance(p))
        }
        _ => Ok(0.),
    }
}

/// Whether the pole `ip` is kept after the kept pole, the next pole being close: kept
/// when it is off the line through them; `None` when the line cannot be made (the
/// next pole is then dropped and this one kept).
fn kept_by_line(
    poles: &Poles,
    ip: usize,
    kept: &Kept,
    t: &Tolerances,
) -> Result<Option<bool>, Failure> {
    let mk_l = MakeLin::from_points(kept.p, poles.c3d[ip + 1])?;
    if !mk_l.is_done() {
        return Ok(None);
    }
    let d: f64 = mk_l.value()?.square_distance(poles.c3d[ip]);
    let next = Kept::at(poles, ip + 1);
    let here = Kept::at(poles, ip);
    let d1 = sq_line_2d(kept.p1, next.p1, here.p1)?;
    let d2 = sq_line_2d(kept.p2, next.p2, here.p2)?;
    Ok(Some(d > t.def || d1 > t.def2d || d2 > t.def2d))
}

/// The validity of every pole and the count of the kept ones.
fn filter(poles: &Poles, t: &Tolerances) -> Result<(Vec<bool>, i32), Failure> {
    let nbpol = poles.c3d.len();
    let mut is_valid: Vec<bool> = vec![true; nbpol];
    let mut nb_pol: i32 = 1;
    let mut kept = Kept::at(poles, 0);
    for ip in 1..nbpol {
        if !is_valid[ip] {
            continue;
        }
        let here = Kept::at(poles, ip);
        let d = kept.p.square_distance(&here.p);
        if !(d > t.tol || sq2d(kept.p1, here.p1) > t.tl2d || sq2d(kept.p2, here.p2) > t.tl2d) {
            is_valid[ip] = false;
            continue;
        }
        let dd: f64 = if ip + 1 < nbpol {
            kept.p.distance(&poles.c3d[ip + 1])
        } else {
            0.
        };
        let keep: bool = if ip + 1 < nbpol && dd < 10. * t.tol {
            match kept_by_line(poles, ip, &kept, t)? {
                Some(keep) => keep,
                None => {
                    is_valid[ip + 1] = false;
                    true
                }
            }
        } else {
            true
        };
        if keep {
            nb_pol += 1;
            kept = here;
        } else {
            is_valid[ip] = false;
        }
    }
    if nb_pol < 2 {
        is_valid[nbpol - 1] = true;
        nb_pol += 1;
    }
    Ok((is_valid, nb_pol))
}

/// The degree-1 B-spline of the kept poles (the last pole always at the end), knots at
/// the pole counts.
fn kept_curves<P: Copy + Default>(
    values: &[P],
    is_valid: &[bool],
    nb_pol: i32,
) -> Result<Array1<'static, P>, Failure> {
    let mut out: Array1<'static, P> = Array1::with_bounds(1, nb_pol).map_err(array_failure)?;
    let mut count: i32 = 0;
    for (ip, v) in values.iter().enumerate() {
        if is_valid[ip] {
            count += 1;
            out[count] = *v;
        }
    }
    out[nb_pol] = values[values.len() - 1];
    Ok(out)
}

/// The improved curves (`#define CurveImprovement`): for more than 100 poles, the
/// curves rebuilt on the kept poles, replacing the caller's handles (natively a
/// const_cast of the arguments); the new last index when they are rebuilt.
pub(super) fn improve(
    poles: &Poles,
    (tol3d, tol2d): (f64, f64),
    curves: &mut SectionCurves,
) -> Result<Option<i32>, Failure> {
    let nbpol = poles.c3d.len();
    if nbpol <= 100 {
        return Ok(None);
    }
    // The tolerances on square distances.
    let tol: f64 = 1.0e-10_f64.max(100. * tol3d * tol3d);
    let t = Tolerances {
        tol,
        tl2d: tol * (tol2d * tol2d) / (tol3d * tol3d),
        def: tol,
        def2d: tol2d,
    };
    let (is_valid, nb_pol) = filter(poles, &t)?;
    if nb_pol as usize >= nbpol {
        return Ok(None);
    }
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, nb_pol).map_err(array_failure)?;
    let mut mults: Array1<'static, i32> = Array1::with_bounds(1, nb_pol).map_err(array_failure)?;
    let mut count: i32 = 0;
    for valid in &is_valid {
        if *valid {
            count += 1;
            knots[count] = f64::from(count);
        }
    }
    for i in 1..=nb_pol {
        mults[i] = if i == 1 || i == nb_pol { 2 } else { 1 };
    }
    let polc3d = kept_curves(&poles.c3d, &is_valid, nb_pol)?;
    curves.c3d = Handle::new(GeomBSplineCurve::new(&polc3d, &knots, &mults, 1, false)?).cast();
    if let Some(pc1) = &poles.pc1 {
        curves.pc1 = Handle::new(Geom2dBSplineCurve::new(
            &kept_curves(pc1, &is_valid, nb_pol)?,
            &knots,
            &mults,
            1,
            false,
        )?)
        .cast();
    }
    if let Some(pc2) = &poles.pc2 {
        curves.pc2 = Handle::new(Geom2dBSplineCurve::new(
            &kept_curves(pc2, &is_valid, nb_pol)?,
            &knots,
            &mults,
            1,
            false,
        )?)
        .cast();
    }
    Ok(Some(nb_pol))
}
