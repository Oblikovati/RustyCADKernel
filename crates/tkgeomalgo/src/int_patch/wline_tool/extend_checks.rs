// Rust translation of the static helpers MinMax, IsIntersectionPoint, IsOutOfDomain and
// CheckArgumentsToExtend of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_WLineTool.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The checks of `ExtendTwoWLines`: whether two walking lines can be extended to a
//! common point, the middle of their ends, and how they connect there.
use super::IntPatchWLineTool;
use crate::int_patch::IntPatchSpecialPoints;
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::bnd::{BoundingBox2, IntersectStatus, Range1};
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};

/// `IntPatchWT_WLsConnectionType`: how two walking lines connect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ConnectionType {
    /// `IntPatchWT_NotConnected`.
    NotConnected,
    /// `IntPatchWT_Singular`: the common point is on a boundary of a periodic surface.
    Singular,
    /// `IntPatchWT_Common`: the lines share their end point.
    Common,
    /// `IntPatchWT_ReqExtend`: the lines must be extended to the common point.
    ReqExtend,
}

/// The inputs of the extension shared by every pair of lines: the surfaces, the
/// tolerance, the periods and the boxes of the parameters.
pub(super) struct ExtendInputs<'a> {
    pub(super) s1: &'a Adaptor3dSurfaceHandle,
    pub(super) s2: &'a Adaptor3dSurfaceHandle,
    pub(super) toler3d: f64,
    pub(super) periods: [f64; 4],
    pub(super) box1: &'a BoundingBox2,
    pub(super) box2: &'a BoundingBox2,
}

/// `MinMax`: orders the two parameters.
fn min_max(par_min: &mut f64, par_max: &mut f64) {
    if *par_min > *par_max {
        std::mem::swap(par_min, par_max);
    }
}

/// The parameters of `p` on a quadric (`ElSLib::Parameters`); None for another type.
fn quadric_parameters(
    s: &Adaptor3dSurfaceHandle,
    p: Point3,
) -> Result<Option<(f64, f64)>, Failure> {
    Ok(Some(match s.get_type()? {
        SurfaceType::Plane => elslib::parameters(&s.plane()?, p),
        SurfaceType::Cylinder => elslib::parameters(&s.cylinder()?, p),
        SurfaceType::Sphere => elslib::parameters(&s.sphere()?, p),
        SurfaceType::Cone => elslib::parameters(&s.cone()?, p),
        SurfaceType::Torus => elslib::parameters(&s.torus()?, p),
        _ => return Ok(None),
    }))
}

/// `IsIntersectionPoint`: whether `pmid` is an intersection point of the quadrics
/// within the tolerance; its parameters on both, brought near those of `ref_pt`, are
/// written into `new_pt`.
fn is_intersection_point(
    pmid: Point3,
    inputs: &ExtendInputs<'_>,
    ref_pt: &IntSurfPntOn2S,
    tol: f64,
    new_pt: &mut IntSurfPntOn2S,
) -> Result<bool, Failure> {
    let Some((u1, v1)) = quadric_parameters(inputs.s1, pmid)? else {
        return Ok(false);
    };
    let Some((u2, v2)) = quadric_parameters(inputs.s2, pmid)? else {
        return Ok(false);
    };
    new_pt.set_value_all(pmid, u1, v1, u2, v2);
    IntPatchSpecialPoints::adjust_point_and_vertex(ref_pt, &inputs.periods, new_pt, None);
    let p1: Point3 = inputs.s1.value(u1, v1)?;
    let p2: Point3 = inputs.s2.value(u2, v2)?;
    Ok(p1.square_distance(&p2) <= tol * tol)
}

/// `IsOutOfDomain`: whether the parameters of the point, brought into the periods from
/// the lower corners, are out of the boxes of the surfaces.
pub(super) fn is_out_of_domain(
    box1: &BoundingBox2,
    box2: &BoundingBox2,
    pon2s: &IntSurfPntOn2S,
    periods: &[f64; 4],
) -> Result<bool, Failure> {
    let (u1, v1, u2, v2) = pon2s.parameters();
    let [u1min, v1min, _, _] = box1.get()?;
    let [u2min, v2min, _, _] = box2.get()?;
    let u1 = elclib::in_period(u1, u1min, u1min + periods[0]);
    let v1 = elclib::in_period(v1, v1min, v1min + periods[1]);
    let u2 = elclib::in_period(u2, u2min, u2min + periods[2]);
    let v2 = elclib::in_period(v2, v2min, v2min + periods[3]);
    Ok(box1.is_out_point(Point2::new(u1, v1)) || box2.is_out_point(Point2::new(u2, v2)))
}

/// The tangent angles of the extension: none over `myMaxConcatAngle`; returns the
/// connection when the ends are within the tolerance, `None` when not connected.
fn angles_connection(vecs: [Vector3; 3], toler3d: f64) -> Result<Option<ConnectionType>, Failure> {
    let [vec1, vec2, vec3] = vecs;
    let max: f64 = IntPatchWLineTool::MY_MAX_CONCAT_ANGLE;
    if vec3.square_magnitude() <= toler3d * toler3d {
        if vec1.angle(vec2)? > max {
            return Ok(None);
        }
        return Ok(Some(ConnectionType::Common));
    }
    if vec1.angle(vec2)? > max || vec1.angle(vec3)? > max || vec2.angle(vec3)? > max {
        return Ok(None);
    }
    Ok(Some(ConnectionType::NotConnected))
}

/// Whether extending both lines to the new parameter `new_par` in a periodic direction
/// crosses the lower corner `lbc`, once the parameters of the ends are brought around it
/// (`Some(true)` when they meet on the boundary, `None` when they cross it).
fn crosses_corner(par_wl: (f64, f64), new_par: f64, lbc: f64, period: f64) -> Option<bool> {
    let mut range = Range1::default();
    range.add(par_wl.0);
    range.add(par_wl.1);
    if range.is_intersected(lbc, period) == IntersectStatus::Out {
        return Some(false);
    }
    // Check, if we intersect surface boundary when we will extend Wline1 or Wline2 to the
    // new point: bring the new parameter between the ends first.
    let (mut par1, mut par2) = par_wl;
    min_max(&mut par1, &mut par2);
    if new_par > par2 {
        let par: f64 = par1 + period * ((new_par - par1) / period).ceil();
        (par1, par2) = (par2, par);
    } else if new_par < par1 {
        let par: f64 = par2 - period * ((par2 - new_par) / period).ceil();
        (par2, par1) = (par1, par);
    }
    let (mut r1, mut r2) = (Range1::default(), Range1::default());
    r1.add(par1);
    r1.add(new_par);
    r2.add(new_par);
    r2.add(par2);
    if r1.is_intersected(lbc, period) != IntersectStatus::Out
        || r2.is_intersected(lbc, period) != IntersectStatus::Out
    {
        return None;
    }
    Some(true)
}

/// `CheckArgumentsToExtend`: whether the lines of end points `pt_wl1` and `pt_wl2` (with
/// the tangents `vecs` of the lines and the vector between the ends) can be extended to
/// the middle of their ends, which is written into `new_point`.
pub(super) fn check_arguments_to_extend(
    inputs: &ExtendInputs<'_>,
    (pt_wl1, pt_wl2): (&IntSurfPntOn2S, &IntSurfPntOn2S),
    new_point: &mut IntSurfPntOn2S,
    vecs: [Vector3; 3],
) -> Result<ConnectionType, Failure> {
    let Some(ret_val) = angles_connection(vecs, inputs.toler3d)? else {
        return Ok(ConnectionType::NotConnected);
    };
    let pmid = Point3::from(0.5 * (pt_wl1.value().xyz() + pt_wl2.value().xyz()));
    // The left-bottom corners of the boxes.
    let [l0, l1, _, _] = inputs.box1.get()?;
    let [l2, l3, _, _] = inputs.box2.get()?;
    let lbc: [f64; 4] = [l0, l1, l2, l3];
    if !is_intersection_point(pmid, inputs, pt_wl1, inputs.toler3d, new_point)? {
        return Ok(ConnectionType::NotConnected);
    }
    if is_out_of_domain(inputs.box1, inputs.box2, new_point, &inputs.periods)? {
        return Ok(ConnectionType::NotConnected);
    }
    let p1 = params_of(pt_wl1);
    let p2 = params_of(pt_wl2);
    let new_par = params_of(new_point);
    let mut is_on_boundary = false;
    for i in 0..4 {
        if inputs.periods[i] == 0.0 {
            // Strictly equal.
            continue;
        }
        match crosses_corner((p1[i], p2[i]), new_par[i], lbc[i], inputs.periods[i]) {
            None => return Ok(ConnectionType::NotConnected),
            Some(true) => is_on_boundary = true,
            Some(false) => {}
        }
    }
    if is_on_boundary {
        return Ok(ConnectionType::Singular);
    }
    if ret_val == ConnectionType::Common {
        return Ok(ConnectionType::Common);
    }
    Ok(ConnectionType::ReqExtend)
}

fn params_of(p: &IntSurfPntOn2S) -> [f64; 4] {
    let (a, b, c, d) = p.parameters();
    [a, b, c, d]
}
