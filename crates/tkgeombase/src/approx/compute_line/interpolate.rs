// Rust translation of ModelingData/TKGeomBase/Approx/Approx_ComputeLine.gxx (ComputeCurve)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_ComputeLine::ComputeCurve`: the interpolation of a piece that cannot
//! receive more points — a straight segment for two points, else a Bezier curve
//! with the extremity tangents.
use super::{ComputeLine, owned_array};
use crate::app_par_curves::arrays::scratch;
use crate::app_par_curves::{Constraint, LeastSquare, LineTool, MultiCurve, MultiPoint};
use crate::approx::tangency::{LineEnd, search_lambda, tangency_vector};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::Vector;

/// The extremity points and tangents of a piece.
struct Ends {
    p1: Array1<'static, Point3>,
    p2: Array1<'static, Point3>,
    p12d: Array1<'static, Point2>,
    p22d: Array1<'static, Point2>,
    v1: Array1<'static, Vector3>,
    v2: Array1<'static, Vector3>,
    v12d: Array1<'static, Vector2>,
    v22d: Array1<'static, Vector2>,
    tangent1: bool,
    tangent2: bool,
}

// The extremities through the overloads ComputeCurve picks (the 3D ones when the
// line has no 2D point, even without 3D points).
fn read_ends<L: LineTool>(line: &L, [first, last]: [i32; 2]) -> Result<Ends, Failure> {
    let (nb3d, nb2d) = (line.nb_p3d()?, line.nb_p2d()?);
    let mut e = Ends {
        p1: scratch(nb3d)?,
        p2: scratch(nb3d)?,
        p12d: scratch(nb2d)?,
        p22d: scratch(nb2d)?,
        v1: scratch(nb3d)?,
        v2: scratch(nb3d)?,
        v12d: scratch(nb2d)?,
        v22d: scratch(nb2d)?,
        tangent1: false,
        tangent2: false,
    };
    if nb3d != 0 && nb2d != 0 {
        line.value(first, &mut e.p1, &mut e.p12d)?;
        line.value(last, &mut e.p2, &mut e.p22d)?;
        e.tangent1 = line.tangency(first, &mut e.v1, &mut e.v12d)?;
        e.tangent2 = line.tangency(last, &mut e.v2, &mut e.v22d)?;
    } else if nb2d != 0 {
        line.value_2d(first, &mut e.p12d)?;
        line.value_2d(last, &mut e.p22d)?;
        e.tangent1 = line.tangency_2d(first, &mut e.v12d)?;
        e.tangent2 = line.tangency_2d(last, &mut e.v22d)?;
    } else {
        line.value_3d(first, &mut e.p1)?;
        line.value_3d(last, &mut e.p2)?;
        e.tangent1 = line.tangency_3d(first, &mut e.v1)?;
        e.tangent2 = line.tangency_3d(last, &mut e.v2)?;
    }
    Ok(e)
}

// The alignment test of the two-point case: it only feeds a debug message
// natively, but gp_Vec::IsParallel raises for a null vector, and each loop stops
// at its first non-parallel tangent.
fn check_parallel(ends: &Ends, [nb3d, nb2d]: [i32; 2], tangents: [bool; 2]) -> Result<(), Failure> {
    let angular: f64 = precision::angular();
    for (tangent, (v3, v2)) in tangents
        .into_iter()
        .zip([(&ends.v1, &ends.v12d), (&ends.v2, &ends.v22d)])
    {
        if !tangent {
            continue;
        }
        for i in 1..=nb3d {
            if !Vector3::between(ends.p1[i], ends.p2[i]).is_parallel(v3[i], angular)? {
                break;
            }
        }
        for i in 1..=nb2d {
            if !Vector2::between(ends.p12d[i], ends.p22d[i]).is_parallel(v2[i], angular)? {
                break;
            }
        }
    }
    Ok(())
}

// Pole `i` of the straight segment of degree `deg`: `P1 + (i - 1) (P2 - P1) / deg`.
fn segment_pole3(p1: Point3, p2: Point3, i: i32, deg: i32) -> Point3 {
    let (k, d): (f64, f64) = (f64::from(i - 1), f64::from(deg));
    Point3::new(
        p1.x() + k * (p2.x() - p1.x()) / d,
        p1.y() + k * (p2.y() - p1.y()) / d,
        p1.z() + k * (p2.z() - p1.z()) / d,
    )
}

fn segment_pole2(p1: Point2, p2: Point2, i: i32, deg: i32) -> Point2 {
    let (k, d): (f64, f64) = (f64::from(i - 1), f64::from(deg));
    Point2::new(
        p1.x() + k * (p2.x() - p1.x()) / d,
        p1.y() + k * (p2.y() - p1.y()) / d,
    )
}

// The MultiPoint of the given points through the constructor the line's
// dimensions select (none without points: the pole stays default).
fn multi_point(
    [nb3d, nb2d]: [i32; 2],
    p3: &Array1<'_, Point3>,
    p2: &Array1<'_, Point2>,
) -> Option<MultiPoint> {
    if nb3d != 0 && nb2d != 0 {
        return Some(MultiPoint::from_points_and_2d(p3, p2));
    }
    if nb3d != 0 {
        return Some(MultiPoint::from_points(p3));
    }
    (nb2d != 0).then(|| MultiPoint::from_points2d(p2))
}

fn segment_curve(ends: &Ends, dims: [i32; 2], deg: i32) -> Result<MultiCurve, Failure> {
    let curve: MultiCurve = MultiCurve::with_nb_poles(deg + 1)?;
    let set = |i: i32, pole: Option<MultiPoint>| match pole {
        Some(pole) => curve.set_value(i, &pole),
        None => Ok(()),
    };
    set(1, multi_point(dims, &ends.p1, &ends.p12d))?;
    set(deg + 1, multi_point(dims, &ends.p2, &ends.p22d))?;
    let (mut p3, mut p2) = (scratch::<Point3>(dims[0])?, scratch::<Point2>(dims[1])?);
    for i in 2..=deg {
        for j in 1..=dims[0] {
            p3[j] = segment_pole3(ends.p1[j], ends.p2[j], i, deg);
        }
        for j in 1..=dims[1] {
            p2[j] = segment_pole2(ends.p12d[j], ends.p22d[j], i, deg);
        }
        set(i, multi_point(dims, &p3, &p2))?;
    }
    Ok(curve)
}

impl ComputeLine {
    /// `ComputeCurve(Line, firstpt, lastpt)`: two points give the straight segment of
    /// degree `degreemin` (errors Precision::Confusion() and PConfusion()); more
    /// points the least-squares curve of degree `nbp + 1` with the extremity
    /// tangents of the line (or of a parabola through three points) and the chord
    /// lambdas. Stores the piece and returns true when the fit is done.
    pub(super) fn compute_curve<L: LineTool + Clone>(
        &mut self,
        line: &L,
        first: i32,
        last: i32,
    ) -> Result<bool, Failure> {
        let nbp: i32 = last - first + 1;
        let mut para: Vector<'static> = Vector::with_bounds(first, last)?;
        self.compute_parameters(line, first, last, &mut para)?;
        let dims: [i32; 2] = [line.nb_p3d()?, line.nb_p2d()?];
        let ends: Ends = read_ends(line, [first, last])?;
        if nbp == 2 {
            check_parallel(&ends, dims, [ends.tangent1, ends.tangent2])?;
            let curve: MultiCurve = segment_curve(&ends, dims, self.options.degree_min)?;
            self.tolreached = true;
            let tolerances = [precision::confusion(), precision::p_confusion()];
            self.store(curve, owned_array(&para)?, tolerances);
            return Ok(true);
        }
        self.tangent_curve(line, [first, last], &para, dims)
    }

    fn tangent_curve<L: LineTool + Clone>(
        &mut self,
        line: &L,
        [first, last]: [i32; 2],
        para: &Vector<'static>,
        [nb3d, nb2d]: [i32; 2],
    ) -> Result<bool, Failure> {
        let deg: i32 = last - first + 2;
        let size: i32 = nb3d * 3 + nb2d * 2;
        let (mut v1, mut v2) = (Vector::with_bounds(1, size)?, Vector::with_bounds(1, size)?);
        let params =
            |l: &L, f: i32, t: i32, p: &mut Vector<'static>| self.compute_parameters(l, f, t, p);
        tangency_vector(line, first, LineEnd::First, &mut v1, &params)?;
        let lambda1: f64 = search_lambda(line, para, &v1, [first, first + 1])?;
        tangency_vector(line, last, LineEnd::Last, &mut v2, &params)?;
        let lambda2: f64 = search_lambda(line, para, &v2, [last - 1, last])?;
        let cons: Constraint = Constraint::TangencyPoint;
        let mut lsq = LeastSquare::new(line, first, last, cons, cons, para, deg + 1)?;
        let (l1, l2) = (lambda1 / f64::from(deg), lambda2 / f64::from(deg));
        lsq.perform_tangents(para, &v1, &v2, l1, l2)?;
        let done: bool = lsq.is_done();
        let curve: MultiCurve = lsq.bezier_value()?;
        if !done {
            return Ok(false);
        }
        let (_, tol3d, tol2d) = lsq.error()?;
        self.tolreached = true;
        self.store(curve, owned_array(para)?, [tol3d, tol2d]);
        Ok(true)
    }
}
