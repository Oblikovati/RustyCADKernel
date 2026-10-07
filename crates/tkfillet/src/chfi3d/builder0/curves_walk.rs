// Rust translation of the walking part of ChFi3d_ComputeCurves (ChFi3d_Builder_0.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The last resort of `ChFi3d_ComputeCurves`: walk the intersection from a point between
//! the known ends, cut the line at the points nearest to them, insert the ends when the
//! line stops short, and approximate the line by B-splines.
use super::curves::{CurveEnds, IntersectionCurves, middle};
use super::stripes::eval_tol_reached;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkgeomalgo::approx_int::ApproxIntRange;
use rustycad_tkgeomalgo::geom_int::GeomIntWLApprox;
use rustycad_tkgeomalgo::int_patch::IntPatchWLine;
use rustycad_tkgeomalgo::int_surf::IntSurfLineOn2S;
use rustycad_tkgeomalgo::int_walk::IntWalkPWalking;
use rustycad_tkgeombase::app_par_curves::MultiCurveEval;
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};

fn array_error(e: impl std::fmt::Debug) -> Failure {
    Failure::with_message(
        "Standard_RangeError",
        &format!("ChFi3d_ComputeCurves: {e:?}"),
    )
}

/// The four parameters of a 1-based array.
fn four(values: [f64; 4]) -> Result<Array1<'static, f64>, Failure> {
    let mut depart = Array1::with_bounds(1, 4).map_err(array_error)?;
    for (k, value) in (1..=4).zip(values) {
        *depart.change_value(k).map_err(array_error)? = value;
    }
    Ok(depart)
}

/// The start of the walk: the first solution from the points at 0.2 to 0.7 of the way
/// back from the end.
fn first_point(
    walk: &mut IntWalkPWalking,
    ends: &CurveEnds<'_>,
) -> Result<Option<[f64; 4]>, Failure> {
    for ipdep in 2..=7 {
        let alpha = 0.1 * f64::from(ipdep);
        let unmoinsalpha = 1.0 - alpha;
        let depart: [f64; 4] =
            std::array::from_fn(|k| alpha * ends.pardeb[k] + unmoinsalpha * ends.parfin[k]);
        if let Some(pintdep) = walk.perform_first_point(&four(depart)?)? {
            let (u1, v1, u2, v2) = pintdep.parameters();
            return Ok(Some([u1, v1, u2, v2]));
        }
    }
    Ok(None)
}

/// A known end of the line: the middle of its points on both surfaces, its tolerance,
/// whether the end is good (its points within `tol3d`).
struct KnownEnd {
    pnt: Point3,
    tol: f64,
    good: bool,
}

fn known_end(ends: &CurveEnds<'_>, params: [f64; 4], tol3d: f64) -> Result<KnownEnd, Failure> {
    let (co1, co2) = (
        ends.s1.value(params[0], params[1])?,
        ends.s2.value(params[2], params[3])?,
    );
    let d = co1.distance(&co2);
    let tol = if d < tol3d { tol3d } else { d };
    Ok(KnownEnd {
        pnt: middle(co1, co2),
        tol,
        good: tol == tol3d,
    })
}

/// The indices of the points of the line nearest to both ends (the last of the ties
/// for the start, the first for the end) and their distances.
fn nearest(
    l2s: &IntSurfLineOn2S,
    pntd: Point3,
    pntf: Point3,
) -> Result<(i32, f64, i32, f64), Failure> {
    let (mut ddeb, mut dfin) = (precision::infinite(), precision::infinite());
    let (mut indd, mut indf) = (0, 0);
    for i in 1..=l2s.nb_points() {
        let p = l2s.value(i)?.value();
        let dd = p.distance(&pntd);
        if dd <= ddeb {
            (ddeb, indd) = (dd, i);
        }
        let dd = p.distance(&pntf);
        if dd < dfin {
            (dfin, indf) = (dd, i);
        }
    }
    Ok((indd, ddeb, indf, dfin))
}

/// The line oriented from the start to the end and cut to the points nearest to them;
/// its number of points.
fn cut_line(l2s: &IntSurfLineOn2S, (mut indd, mut indf): (i32, i32)) -> Result<i32, Failure> {
    let mut nbp = l2s.nb_points();
    if indd > indf {
        l2s.reverse();
        indd = nbp - indd + 1;
        indf = nbp - indf + 1;
    }
    for _ in 1..indd {
        l2s.remove_point(1)?;
        nbp -= 1;
        indf -= 1;
    }
    for _ in (indf + 1)..=nbp {
        l2s.remove_point(indf + 1)?;
    }
    Ok(indf)
}

/// The known end put on the line at `index` (1 or the last point; `neighbour` is the
/// next point inward): replacing the end point, replacing it after dropping it when the
/// known end lies past it, or inserted before it when the end point is far.
fn put_end(
    l2s: &IntSurfLineOn2S,
    (index, neighbour): (i32, i32),
    (pnt, params): (Point3, [f64; 4]),
    nbp: &mut i32,
    at_start: bool,
) -> Result<(), Failure> {
    let (mut p1, p2) = (l2s.value(index)?, l2s.value(neighbour)?);
    let v1 = Vector3::between(pnt, p1.value());
    let v2 = Vector3::between(p1.value(), p2.value());
    let v3 = Vector3::between(pnt, p2.value());
    p1.set_value_all(pnt, params[0], params[1], params[2], params[3]);
    if v1.dot(v3) < 0.0 {
        let mut index = index;
        if v3.magnitude() < 0.2 * v2.magnitude() {
            l2s.remove_point(index)?;
            *nbp -= 1;
            if !at_start {
                index = *nbp;
            }
        }
        return l2s.set_value(index, &p1);
    }
    if v1.magnitude() > 0.2 * v2.magnitude() {
        if at_start {
            l2s.insert_before(1, &p1)?
        } else {
            l2s.add(&p1)
        }
        *nbp += 1;
        return Ok(());
    }
    l2s.set_value(index, &p1)
}

/// The B-spline curves of the approximation, their ends reset on the known ends when
/// the line stopped short of them.
fn approximated(
    ends: &CurveEnds<'_>,
    approx: &GeomIntWLApprox,
    (deb, ddeb): (&KnownEnd, f64),
    (fin, dfin): (&KnownEnd, f64),
    out: &mut IntersectionCurves,
) -> Result<(), Failure> {
    let mbs = approx.value(1);
    let nbpol = mbs.nb_poles();
    let mut pol3d: Array1<'static, Point3> = Array1::with_bounds(1, nbpol).map_err(array_error)?;
    mbs.curve_3d(1, &mut pol3d)?;
    let mut pol2d1: Array1<'static, Point2> = Array1::with_bounds(1, nbpol).map_err(array_error)?;
    mbs.curve_2d(2, &mut pol2d1)?;
    let mut pol2d2: Array1<'static, Point2> = Array1::with_bounds(1, nbpol).map_err(array_error)?;
    mbs.curve_2d(3, &mut pol2d2)?;
    if ddeb >= deb.tol {
        *pol3d.change_value(1).map_err(array_error)? = deb.pnt;
        *pol2d1.change_value(1).map_err(array_error)? = Point2::new(ends.pardeb[0], ends.pardeb[1]);
        *pol2d2.change_value(1).map_err(array_error)? = Point2::new(ends.pardeb[2], ends.pardeb[3]);
    }
    if dfin >= fin.tol {
        *pol3d.change_value(nbpol).map_err(array_error)? = fin.pnt;
        *pol2d1.change_value(nbpol).map_err(array_error)? =
            Point2::new(ends.parfin[0], ends.parfin[1]);
        *pol2d2.change_value(nbpol).map_err(array_error)? =
            Point2::new(ends.parfin[2], ends.parfin[3]);
    }
    let (knots, mults, deg) = (mbs.knots()?, mbs.multiplicities()?, mbs.degree()?);
    out.c3d = Handle::new(GeomBSplineCurve::new(&pol3d, &knots, &mults, deg, false)?).cast();
    out.pc1 = Handle::new(Geom2dBSplineCurve::new(
        &pol2d1, &knots, &mults, deg, false,
    )?)
    .cast();
    out.pc2 = Handle::new(Geom2dBSplineCurve::new(
        &pol2d2, &knots, &mults, deg, false,
    )?)
    .cast();
    let tolreached = eval_tol_reached((ends.s1, &out.pc1), (ends.s2, &out.pc2), &out.c3d)?;
    let tolreached = if tolreached < ddeb { ddeb } else { tolreached };
    out.tolreached = if tolreached < dfin { dfin } else { tolreached };
    Ok(())
}

/// The walked line of more than 30 points cut, completed with the known ends and
/// approximated; `false` when it reduces to one point or the approximation fails.
fn approximate_line(
    ends: &CurveEnds<'_>,
    l2s: &Handle<IntSurfLineOn2S>,
    (tol3d, tol2d): (f64, f64),
    out: &mut IntersectionCurves,
) -> Result<bool, Failure> {
    let line = l2s.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ChFi3d_ComputeCurves: null walked line",
        )
    })?;
    let deb = known_end(ends, ends.pardeb, tol3d)?;
    let fin = known_end(ends, ends.parfin, tol3d)?;
    let (indd, mut ddeb, indf, mut dfin) = nearest(line, deb.pnt, fin.pnt)?;
    let mut nbp = cut_line(line, (indd, indf))?;
    if nbp == 1 {
        return Ok(false);
    }
    if ddeb >= tol3d && deb.good {
        put_end(line, (1, 2), (deb.pnt, ends.pardeb), &mut nbp, true)?;
        ddeb = 0.0;
    }
    if dfin >= tol3d && fin.good {
        put_end(
            line,
            (nbp, nbp - 1),
            (fin.pnt, ends.parfin),
            &mut nbp,
            false,
        )?;
        dfin = 0.0;
    }
    let wl = Handle::new(IntPatchWLine::new_undecided(l2s, false));
    let mut approx = GeomIntWLApprox::new();
    approx.set_parameters(
        1.0e-7,
        tol2d,
        4,
        8,
        0,
        30,
        true,
        ParametrizationType::ChordLength,
    );
    let range = ApproxIntRange {
        approx_xyz: true,
        approx_u1v1: true,
        approx_u2v2: true,
        indicemin: 1,
        indicemax: nbp,
    };
    approx.perform_on_surfaces(ends.s1, ends.s2, &wl, range)?;
    if !approx.is_done() {
        return Ok(false);
    }
    approximated(ends, &approx, (&deb, ddeb), (&fin, dfin), out)?;
    Ok(true)
}

/// The walking of `ChFi3d_ComputeCurves`, halving the step while the line has at most
/// 30 points; whether it succeeded.
pub(super) fn walked_curves(
    ends: &CurveEnds<'_>,
    tols: (f64, f64),
    out: &mut IntersectionCurves,
) -> Result<bool, Failure> {
    let tol3d = tols.0;
    let mut step = 0.1;
    loop {
        let fleche = 1.0e-3 * ends.pdeb.distance(&ends.pfin);
        let mut walk = IntWalkPWalking::new(ends.s1, ends.s2, tol3d, tol3d, fleche, step)?;
        let Some(depart) = first_point(&mut walk, ends)? else {
            return Ok(false);
        };
        walk.perform(&four(depart)?)?;
        if !walk.is_done() {
            return Ok(false);
        }
        if walk.nb_points()? > 30 {
            return approximate_line(ends, walk.line()?, tols, out);
        }
        step *= 0.5;
        if step <= 0.0001 {
            return Ok(false);
        }
    }
}
