// Rust translation of BRepLib::FindValidRange (BRepLib_1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::FindValidRange`: the part of a curve outside the tolerance spheres of its
//! end vertices.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point3;
use std::cmp::Ordering;

/// A vertex at one end of the range: its parameter, point and tolerance.
#[derive(Clone, Copy, Debug)]
pub struct RangeEnd {
    /// The parameter of the vertex on the curve.
    pub parameter: f64,
    /// The point of the vertex.
    pub point: Point3,
    /// The tolerance of the vertex.
    pub tolerance: f64,
}

/// The outputs of `BRepLib::FindValidRange`: whether the range is valid, and
/// `theFirst` and `theLast` when the native call writes them (a failing call may write
/// one or both before it returns false).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ValidRange {
    /// The native return value.
    pub is_valid: bool,
    /// `theFirst`, when written.
    pub first: Option<f64>,
    /// `theLast`, when written.
    pub last: Option<f64>,
}

impl ValidRange {
    /// A failed search with the outputs written so far.
    fn failed(first: Option<f64>, last: Option<f64>) -> Self {
        Self {
            is_valid: false,
            first,
            last,
        }
    }
}

/// `BRepLib::FindValidRange(theCurve, theTolE, theParV1, thePntV1, theTolV1, theParV2,
/// thePntV2, theTolV2, theFirst, theLast)`: the range of `curve` between the vertices
/// that no vertex tolerance sphere covers (an infinite end is kept as it is); invalid
/// when there is none; `brep_lib::find_valid_range(&curve, 1e-7, v1, v2)?`.
pub fn find_valid_range(
    curve: &dyn Adaptor3dCurve,
    tol_e: f64,
    v1: RangeEnd,
    v2: RangeEnd,
) -> Result<ValidRange, Failure> {
    if v2.parameter - v1.parameter < precision::p_confusion() {
        return Ok(ValidRange::failed(None, None));
    }
    let eps: f64 = range_epsilon(curve, tol_e, v1.parameter, v2.parameter)?;
    let span = (v1.parameter, v2.parameter);
    let first: f64 = if precision::is_infinite(v1.parameter) {
        v1.parameter
    } else {
        let Some(first) = find_nearest_valid_point(curve, span, true, v1, eps)? else {
            return Ok(ValidRange::failed(None, None));
        };
        if v2.parameter - first < eps {
            return Ok(ValidRange::failed(Some(first), None));
        }
        first
    };
    let last: f64 = if precision::is_infinite(v2.parameter) {
        v2.parameter
    } else {
        let Some(last) = find_nearest_valid_point(curve, span, false, v2, eps)? else {
            return Ok(ValidRange::failed(Some(first), None));
        };
        if last - v1.parameter < eps {
            return Ok(ValidRange::failed(Some(first), Some(last)));
        }
        last
    };
    // check found parameters: overlapping is not a valid range
    Ok(ValidRange {
        is_valid: first.partial_cmp(&last) != Some(Ordering::Greater),
        first: Some(first),
        last: Some(last),
    })
}

/// The search precision: a tenth of the curve resolution of the edge tolerance, at least
/// the epsilon of the larger finite parameter and Precision::PConfusion().
fn range_epsilon(
    curve: &dyn Adaptor3dCurve,
    tol_e: f64,
    par_v1: f64,
    par_v2: f64,
) -> Result<f64, Failure> {
    let mut max_par: f64 = 0.0;
    if !precision::is_infinite(par_v1) {
        max_par = par_v1.abs();
    }
    if !precision::is_infinite(par_v2) {
        max_par = std_max(max_par, par_v2.abs());
    }
    let resolution: f64 = curve.resolution(tol_e)? * 0.1;
    Ok(std_max(
        std_max(resolution, epsilon(max_par)),
        precision::p_confusion(),
    ))
}

/// `BRepLib::FindValidRange(theEdge, theFirst, theLast)`: the valid range of the 3d curve
/// of the edge, with the vertex tolerances enlarged by Precision::Confusion() (and a
/// missing vertex replaced by the curve end with the edge tolerance); invalid for an edge
/// without 3d curve or range; `brep_lib::find_valid_range_of_edge(&edge)?`.
pub fn find_valid_range_of_edge(edge: &Edge) -> Result<ValidRange, Failure> {
    if BRepTool::curve_with_location(edge)?.curve.is_null() {
        return Ok(ValidRange::failed(None, None));
    }
    let curve = BRepAdaptorCurve::with_edge(edge)?;
    let par_v: [f64; 2] = [curve.first_parameter()?, curve.last_parameter()?];
    if par_v[1] - par_v[0] < precision::p_confusion() {
        return Ok(ValidRange::failed(None, None));
    }
    let (first, last) = topexp::vertices(edge, false);
    let tol_e: f64 = BRepTool::edge_tolerance(edge)?;
    let mut ends = [
        RangeEnd {
            parameter: par_v[0],
            point: Point3::default(),
            tolerance: precision::confusion(),
        },
        RangeEnd {
            parameter: par_v[1],
            point: Point3::default(),
            tolerance: precision::confusion(),
        },
    ];
    for (end, vertex) in ends.iter_mut().zip([&first, &last]) {
        if !vertex.is_null() {
            end.tolerance += BRepTool::vertex_tolerance(vertex)?;
            end.point = BRepTool::pnt(vertex)?;
        } else if !precision::is_infinite(end.parameter) {
            end.tolerance += tol_e;
            end.point = curve.value(end.parameter)?;
        }
    }
    find_valid_range(&curve, tol_e, ends[0], ends[1])
}

/// `findNearestValidPoint`: starting from the chosen end of `span` (inside the sphere of
/// the vertex, else None), the parameter where the curve leaves the sphere, found by
/// steps of the curve resolution then by bisection to `eps`; None when the whole range is
/// inside.
fn find_nearest_valid_point(
    curve: &dyn Adaptor3dCurve,
    span: (f64, f64),
    is_first: bool,
    end: RangeEnd,
    eps: f64,
) -> Result<Option<f64>, Failure> {
    // 1. Check that the needed end is inside the sphere
    let (start_u, end_u) = if is_first { span } else { (span.1, span.0) };
    let sq_tol: f64 = end.tolerance * end.tolerance;
    if curve.value(start_u)?.square_distance(&end.point) > sq_tol {
        // the vertex does not cover the corresponding to this vertex end of the curve
        return Ok(None);
    }
    // 2. Find a nearest point that is outside
    let mut step: f64 = curve.resolution(end.tolerance)? * 1.01;
    if step < eps {
        step = eps;
    }
    let d1_mag: f64 = singularity_threshold(curve)?;
    if !is_first {
        step = -step;
    }
    let walk = SphereWalk {
        curve,
        center: end.point,
        sq_tol,
        end_u,
        is_first,
        d1_mag,
    };
    let Some((u_in, u_out)) = walk.leave_sphere(start_u, step)? else {
        return Ok(None);
    };
    // 3. Precise solution with binary search
    walk.bisect(u_in, u_out, eps).map(Some)
}

/// "aD1Mag": for Bezier and B-spline curves (also as offset basis), the square of a
/// hundredth of the maximal derivative, below which a derivative is a local
/// singularity; 0 otherwise.
fn singularity_threshold(curve: &dyn Adaptor3dCurve) -> Result<f64, Failure> {
    let mut kind: CurveType = curve.get_type()?;
    if kind == CurveType::OffsetCurve {
        let offset = curve.offset_curve()?;
        let basis = offset.get().map(|o| o.basis_curve()).unwrap_or_default();
        kind = GeomAdaptorCurve::with_curve(&basis)?.get_type()?;
    }
    if kind != CurveType::BezierCurve && kind != CurveType::BSplineCurve {
        return Ok(0.0);
    }
    let d1_mag: f64 = 1.0 / curve.resolution(1.0)? * 0.01;
    Ok(d1_mag * d1_mag)
}

/// The walk out of the tolerance sphere of one vertex.
struct SphereWalk<'a> {
    curve: &'a dyn Adaptor3dCurve,
    center: Point3,
    sq_tol: f64,
    end_u: f64,
    is_first: bool,
    d1_mag: f64,
}

impl SphereWalk<'_> {
    /// Whether the curve point at `u` is outside the sphere.
    fn is_out(&self, u: f64) -> Result<bool, Failure> {
        Ok(self.curve.value(u)?.square_distance(&self.center) > self.sq_tol)
    }

    /// Whether `u` is beyond the opposite end of the range.
    fn beyond(&self, u: f64) -> bool {
        if self.is_first {
            u > self.end_u
        } else {
            u < self.end_u
        }
    }

    /// The last parameter inside and the first outside the sphere, stepping from
    /// `start_u`; None when the whole range is inside.
    fn leave_sphere(&self, start_u: f64, step: f64) -> Result<Option<(f64, f64)>, Failure> {
        let (mut u_in, mut u_out) = (start_u, start_u);
        let mut is_out: bool = false;
        while !is_out {
            u_in = u_out;
            u_out += step;
            if self.beyond(u_out) {
                // step is too big and we go out of bounds, check if the opposite bound is outside
                if !self.is_out(self.end_u)? {
                    return Ok(None); // all range is inside sphere
                }
                return Ok(Some((u_in, self.end_u)));
            }
            let Some(out) = self.probe(&mut u_out, step)? else {
                return Ok(None);
            };
            is_out = out;
        }
        Ok(Some((u_in, u_out)))
    }

    /// The state at `u_out`, after doubling steps out of a local singularity (a small
    /// derivative inside the sphere); None when that walk ends inside at the range end.
    fn probe(&self, u_out: &mut f64, step: f64) -> Result<Option<bool>, Failure> {
        if self.d1_mag <= 0. {
            return self.is_out(*u_out).map(Some);
        }
        let mut step_local: f64 = step;
        loop {
            // cycle to go out of local singularity
            let (point, d1) = self.curve.d1(*u_out)?;
            let is_out: bool = point.square_distance(&self.center) > self.sq_tol;
            if is_out || d1.square_magnitude() >= self.d1_mag {
                return Ok(Some(is_out));
            }
            step_local *= 2.;
            *u_out += step_local;
            if (self.is_first && *u_out < self.end_u) || (!self.is_first && *u_out > self.end_u) {
                continue; // still in range
            }
            // went out of range, so check if the end point has out state
            *u_out = self.end_u;
            if !self.is_out(*u_out)? {
                return Ok(None); // all range is inside sphere
            }
            return Ok(Some(true));
        }
    }

    /// The middle of the final bracket after bisecting it below `eps`.
    fn bisect(&self, mut u_in: f64, mut u_out: f64, eps: f64) -> Result<f64, Failure> {
        let mut delta: f64 = (u_out - u_in).abs();
        while delta > eps {
            let mid_u: f64 = (u_in + u_out) * 0.5;
            if self.is_out(mid_u)? {
                u_out = mid_u;
            } else {
                u_in = mid_u;
            }
            delta = (u_out - u_in).abs();
        }
        Ok((u_in + u_out) * 0.5)
    }
}
