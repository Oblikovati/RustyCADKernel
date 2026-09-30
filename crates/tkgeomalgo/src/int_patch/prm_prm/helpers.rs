// Rust translation of the static helpers SectionPointToParameters, IsPointOnLine,
// AddWLine, AdjustOnPeriodic, MakeNewPoint and DublicateOfLinesProcessing of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The helpers of the `Perform` overloads: the parameters of a section point, the test
//! of a point against a walking line, the adding of a line, the periodic end points,
//! and the duplicated lines.
use crate::int_patch::{IntPatchLine, IntPatchPointLine, IntPatchPolyhedron, IntPatchWLine};
use crate::int_start::failures::sequence_value;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use crate::int_walk::IntWalkPWalking;
use crate::intf::{PIType, SectionPoint};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector3};

/// The parameters of a point given by its place in the polyhedron: a vertex, a point of
/// an edge, or a point of a triangle (by its barycentric coordinates); the parameters
/// are left unchanged for another place.
fn place_to_parameters(
    info: (PIType, i32, i32, f64),
    poly: &IntPatchPolyhedron,
    p: Point3,
    uv: &mut (f64, f64),
) -> Result<(), Failure> {
    let (typ, adr1, adr2, param) = info;
    match typ {
        // Adr1 is the number of the vertex.
        PIType::Vertex => *uv = poly.parameters(adr1)?,
        PIType::Edge => {
            *uv = poly.parameters(adr1)?;
            let (u, v) = poly.parameters(adr2)?;
            uv.0 += param * (u - uv.0);
            uv.1 += param * (v - uv.1);
        }
        PIType::Face => *uv = face_parameters(poly, adr1, p)?,
        PIType::External => {}
    }
    Ok(())
}

/// The parameters of the point `p` of the triangle `triangle`.
fn face_parameters(
    poly: &IntPatchPolyhedron,
    triangle: i32,
    p: Point3,
) -> Result<(f64, f64), Failure> {
    let [pt1, pt2, pt3] = poly.triangle(triangle);
    let (pa, pb, pc) = (poly.point(pt1)?, poly.point(pt2)?, poly.point(pt3)?);
    let (ua, va) = poly.parameters(pt1)?;
    let (ub, vb) = poly.parameters(pt2)?;
    let (uc, vc) = poly.parameters(pt3)?;
    let normale: Vector3 = Vector3::between(pa, pb).crossed(Vector3::between(pa, pc));
    let mut cc: f64 = Vector3::between(pa, pb)
        .crossed(Vector3::between(pa, p))
        .dot(normale);
    let mut ca: f64 = Vector3::between(pb, pc)
        .crossed(Vector3::between(pb, p))
        .dot(normale);
    let mut cb: f64 = Vector3::between(pc, pa)
        .crossed(Vector3::between(pc, p))
        .dot(normale);
    let cabc: f64 = ca + cb + cc;
    ca /= cabc;
    cb /= cabc;
    cc /= cabc;
    Ok((ca * ua + cb * ub + cc * uc, ca * va + cb * vb + cc * vc))
}

/// `SectionPointToParameters(Sp, Poly1, Poly2, u1, v1, u2, v2)`: the parameters of the
/// section point on both polyhedra, written into `uv` (`[u1, v1, u2, v2]`); a side of
/// an unknown type keeps its previous values (the native locals keep theirs).
pub(super) fn section_point_to_parameters(
    sp: &SectionPoint,
    poly1: &IntPatchPolyhedron,
    poly2: &IntPatchPolyhedron,
    uv: &mut [f64; 4],
) -> Result<(), Failure> {
    let p: Point3 = sp.pnt();
    let mut first: (f64, f64) = (uv[0], uv[1]);
    place_to_parameters(sp.info_first(), poly1, p, &mut first)?;
    (uv[0], uv[1]) = first;
    let mut second: (f64, f64) = (uv[2], uv[3]);
    place_to_parameters(sp.info_second(), poly2, p, &mut second)?;
    (uv[2], uv[3]) = second;
    Ok(())
}

/// The distance test of `IsPointOnLine` on the segment `[pa, pb]` for a point `pstart`
/// projecting inside it: the components of its distance to the segment line, while
/// they are below the deflection.
fn is_near_segment(
    pa: Point3,
    pb: Point3,
    pstart: Point3,
    deflection: f64,
) -> Result<bool, Failure> {
    let deflection2: f64 = deflection * deflection;
    let am: Vector3 = Vector3::between(pa, pstart);
    let abn: Direction3 = Direction3::new(pb.x() - pa.x(), pb.y() - pa.y(), pb.z() - pa.z())?;
    let lan: f64 = abn.x() * am.x() + abn.y() * am.y() + abn.z() * am.z();
    let ah = Vector3::new(lan * abn.x(), lan * abn.y(), lan * abn.z());
    let hm = Vector3::new(am.x() - ah.x(), am.y() - ah.y(), am.z() - ah.z());
    let mut d: f64 = 0.0;
    if hm.x() < deflection {
        d += hm.x() * hm.x();
        if hm.y() < deflection {
            d += hm.y() * hm.y();
            if hm.z() < deflection {
                d += hm.z() * hm.z();
            } else {
                d = deflection2;
            }
        } else {
            d = deflection2;
        }
    } else {
        d = deflection2;
    }
    Ok(d < deflection2)
}

/// Whether `pstart` is on the segment `[pa, pb]` of a walking line.
fn is_on_segment(pa: Point3, pb: Point3, pstart: Point3, deflection: f64) -> Result<bool, Failure> {
    let am: Vector3 = Vector3::between(pa, pstart);
    let mb: Vector3 = Vector3::between(pstart, pb);
    if am.dot(mb) > 0.0 {
        return is_near_segment(pa, pb, pstart, deflection);
    }
    let dab: f64 = pa.square_distance(&pb);
    if pa.square_distance(&pstart) < dab {
        return Ok(true);
    }
    Ok(pb.square_distance(&pstart) < dab)
}

/// `IsPointOnLine(thePOn2S, theWLine, Deflection)`: whether the point is in the boxes of
/// the walking line and within the deflection of one of its segments.
pub(super) fn is_point_on_line(
    pon2s: &IntSurfPntOn2S,
    wline: &IntPatchWLine,
    deflection: f64,
) -> Result<bool, Failure> {
    let (pu1, pv1, pu2, pv2) = pon2s.parameters();
    if wline.is_out_surf1_box(Point2::new(pu1, pv1))?
        || wline.is_out_surf2_box(Point2::new(pu2, pv2))?
        || wline.is_out_box(pon2s.value())?
    {
        return Ok(false);
    }
    let nb_pnt_on_2s_on_line: i32 = wline.nb_pnts()?;
    for ll in 1..nb_pnt_on_2s_on_line {
        let (pa, pb) = (wline.point(ll)?.value(), wline.point(ll + 1)?.value());
        if is_on_segment(pa, pb, pon2s.value(), deflection)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Whether the walking line `wline` lies on `new_line`: its middle point and all its
/// vertices are on it.
fn is_on_wline(
    wline: &IntPatchWLine,
    new_line: &IntPatchWLine,
    deflection: f64,
) -> Result<bool, Failure> {
    let mid_index: i32 = (wline.nb_pnts()? + 1) / 2;
    if mid_index <= 0 || !is_point_on_line(&wline.point(mid_index)?, new_line, deflection)? {
        return Ok(false);
    }
    // Middle point is on theWLine. Check vertices.
    for j in 1..=wline.nb_vertex() {
        if !is_point_on_line(&wline.vertex(j)?.pnt_on_2s(), new_line, deflection)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `AddWLine(theLines, theWLine, Deflection)`: removes the walking lines of `lines`
/// that lie on `wline`, then appends it.
pub(super) fn add_wline(
    lines: &mut Vec<Handle<dyn IntPatchLine>>,
    wline: &Handle<IntPatchWLine>,
    deflection: f64,
) -> Result<(), Failure> {
    let mut i: usize = 0;
    while i < lines.len() {
        let existing: Handle<IntPatchWLine> = lines[i].cast::<IntPatchWLine>();
        if !existing.is_null() && is_on_wline(&existing, wline, deflection)? {
            lines.remove(i);
        } else {
            i += 1;
        }
    }
    lines.push(wline.cast());
    Ok(())
}

/// Widens the bounds `[min, max]` (`U1, V1, U2, V2`) to the parameters `uv`, the
/// maxima first, as the native code compares them.
pub(super) fn widen_bounds((min, max): (&mut [f64; 4], &mut [f64; 4]), uv: [f64; 4]) {
    for k in 0..4 {
        if uv[k] > max[k] {
            max[k] = uv[k];
        }
    }
    for k in 0..4 {
        if uv[k] < min[k] {
            min[k] = uv[k];
        }
    }
}

/// The periods of both surfaces (0 on a non periodic direction), `U1, V1, U2, V2`.
pub(super) fn surface_periods(
    s1: &Adaptor3dSurfaceHandle,
    s2: &Adaptor3dSurfaceHandle,
) -> Result<[f64; 4], Failure> {
    Ok([
        if s1.is_u_periodic()? {
            s1.u_period()?
        } else {
            0.0
        },
        if s1.is_v_periodic()? {
            s1.v_period()?
        } else {
            0.0
        },
        if s2.is_u_periodic()? {
            s2.u_period()?
        } else {
            0.0
        },
        if s2.is_v_periodic()? {
            s2.v_period()?
        } else {
            0.0
        },
    ])
}

/// One end of a line of `AdjustOnPeriodic`: a periodic parameter on 0 (on the period)
/// moves to the period (to 0) when its neighbour is more than a quarter of a period away.
fn adjust_end(
    line: &IntSurfLineOn2S,
    (indx, indq): (i32, i32),
    periodic: ([bool; 4], [f64; 4]),
) -> Result<(), Failure> {
    let (is_periodic, period) = periodic;
    let eps: f64 = precision::confusion();
    let psx: IntSurfPntOn2S = line.value(indx)?;
    let (a, b, c, d) = psx.parameters();
    let mut ux: [f64; 4] = [a, b, c, d];
    let (a, b, c, d) = line.value(indq)?.parameters();
    let uq: [f64; 4] = [a, b, c, d];
    // The flag is not reset between the parameters, as natively.
    let mut modified = false;
    for k in 0..4 {
        if !is_periodic[k] {
            continue;
        }
        let (mut is_null, mut is_period) = (false, false);
        if ux[k].abs() < eps {
            (modified, is_null) = (true, true);
        } else if (ux[k] - period[k]).abs() < eps {
            (modified, is_period) = (true, true);
        }
        if modified && (ux[k] - uq[k]).abs() > 0.25 * period[k] {
            if is_null {
                ux[k] = period[k];
            }
            if is_period {
                ux[k] = 0.0;
            }
        }
    }
    if modified {
        let mut pnt: IntSurfPntOn2S = psx;
        pnt.set_parameters(ux[0], ux[1], ux[2], ux[3]);
        line.set_value(indx, &pnt)?;
    }
    Ok(())
}

/// `AdjustOnPeriodic(Surf1, Surf2, aSLin)`: the ends of the walking lines of at least
/// 10 points on the seam of a periodic surface are moved to the side of their
/// neighbour.
pub(super) fn adjust_on_periodic(
    s1: &Adaptor3dSurfaceHandle,
    s2: &Adaptor3dSurfaceHandle,
    lines: &[Handle<dyn IntPatchLine>],
) -> Result<(), Failure> {
    let is_periodic: [bool; 4] = [
        s1.is_u_periodic()?,
        s1.is_v_periodic()?,
        s2.is_u_periodic()?,
        s2.is_v_periodic()?,
    ];
    let mut period: [f64; 4] = [0.0; 4];
    let getters: [&dyn Fn() -> Result<f64, Failure>; 4] = [
        &|| s1.u_period(),
        &|| s1.v_period(),
        &|| s2.u_period(),
        &|| s2.v_period(),
    ];
    for k in 0..4 {
        if is_periodic[k] {
            period[k] = getters[k]()?;
        }
    }
    for line in lines {
        let curve: Handle<IntSurfLineOn2S> = line.cast::<IntPatchWLine>().curve();
        let nb_px: i32 = curve.nb_points();
        if nb_px < 10 {
            continue;
        }
        adjust_end(&curve, (1, 2), (is_periodic, period))?;
        adjust_end(&curve, (nb_px, nb_px - 1), (is_periodic, period))?;
    }
    Ok(())
}

/// `MakeNewPoint(replacePnt, oldPnt, Periods)`: the point `replace` with its periodic
/// parameters moved by a period toward those of `old` when they are half a period away.
pub(super) fn make_new_point(
    replace: &IntSurfPntOn2S,
    old: &IntSurfPntOn2S,
    periods: &[f64; 4],
) -> IntSurfPntOn2S {
    let mut new_point = IntSurfPntOn2S::new();
    new_point.set_point(replace.value());
    let (a, b, c, d) = old.parameters();
    let old_params: [f64; 4] = [a, b, c, d];
    let (a, b, c, d) = replace.parameters();
    let mut new_params: [f64; 4] = [a, b, c, d];
    for i in 0..4 {
        if periods[i] != 0.0 && (new_params[i] - old_params[i]).abs() >= 0.5 * periods[i] {
            if new_params[i] < old_params[i] {
                new_params[i] += periods[i];
            } else {
                new_params[i] -= periods[i];
            }
        }
    }
    new_point.set_parameters(new_params[0], new_params[1], new_params[2], new_params[3]);
    new_point
}

/// `DublicateOfLinesProcessing(thePW, theWLID, theLines, theIsRejectReq)`: the line
/// `wl_id` of `lines`, which the walking duplicates, is removed (and the walking kept)
/// when the walking has more points, or as many points and a longer polyline.
pub(super) fn dublicate_of_lines_processing(
    pw: &IntWalkPWalking,
    wl_id: i32,
    lines: &mut Vec<Handle<dyn IntPatchLine>>,
    is_reject_req: &mut bool,
) -> Result<(), Failure> {
    let exist_wl: Handle<IntPatchWLine> =
        sequence_value(lines, wl_id, "NCollection_Sequence::Value")?.cast::<IntPatchWLine>();
    let nb_prev_points: i32 = exist_wl.nb_pnts()?;
    let nb_curr_points: i32 = pw.nb_points()?;
    if nb_prev_points < nb_curr_points {
        // Remove preview line.
        lines.remove((wl_id - 1) as usize);
        *is_reject_req = false;
    } else if nb_prev_points == nb_curr_points {
        let (mut l_prev, mut l_curr) = (0.0, 0.0);
        for nb_pp in 1..nb_prev_points {
            let (p1prev, p2prev) = (
                exist_wl.point(nb_pp)?.value(),
                exist_wl.point(nb_pp + 1)?.value(),
            );
            let (p1curr, p2curr) = (pw.value(nb_pp)?.value(), pw.value(nb_pp + 1)?.value());
            l_prev += p1prev.distance(&p2prev);
            l_curr += p1curr.distance(&p2curr);
        }
        if l_prev < l_curr {
            // Remove preview line.
            lines.remove((wl_id - 1) as usize);
            *is_reject_req = false;
        }
    }
    Ok(())
}
