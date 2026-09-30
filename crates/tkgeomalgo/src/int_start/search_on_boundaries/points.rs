// Rust translation of FindVertex and PointProcess of
// ModelingAlgorithms/TKGeomAlgo/IntStart/IntStart_SearchOnBoundaries.gxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertices of an arc that are solutions, and the bookkeeping of a solution point:
//! merged with a vertex, with an existing point, or appended.
use super::value_or_keep;
use crate::int_start::tools::{
    SearchOnBoundariesTypes, TheArc, TheArcFunction, ThePathPoint, TheSOBTool, TheTopolTool,
    TheVertex,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// `FindVertex(A, Domain, Func, pnt, Toler)`: appends every vertex of the arc where the
/// function is within `toler` of zero.
pub(super) fn find_vertex<T: SearchOnBoundariesTypes>(
    arc: &TheArc,
    domain: &T::TopolTool,
    func: &mut T::Function,
    pnt: &mut Vec<T::PathPoint>,
    toler: f64,
) -> Result<(), Failure> {
    // `valf` is uninitialized natively; see `value_or_keep`.
    let mut valf: f64 = 0.0;
    domain.initialize_arc(arc)?;
    domain.init_vertex_iterator();
    while domain.more_vertex() {
        let vtx: TheVertex = domain.vertex()?;
        let param: f64 = T::SOBTool::parameter(&vtx, arc)?;
        valf = value_or_keep(func, param, valf)?;
        if valf.abs() <= toler {
            let itemp: i32 = func.get_state_number();
            let point: Point3 = func.valpoint(itemp)?;
            pnt.push(T::PathPoint::on_vertex(point, toler, &vtx, arc, param));
        }
        domain.next_vertex();
    }
    Ok(())
}

/// Whether the path point `ptsol` is the vertex `vtx` of the arc at `para` (the vertex
/// search of `PointProcess`).
fn is_known_vertex<T: SearchOnBoundariesTypes>(
    domain: &T::TopolTool,
    ptsol: &T::PathPoint,
    (vtx, arc): (&TheVertex, &TheArc),
    para: f64,
    toler: f64,
) -> Result<bool, Failure> {
    if ptsol.is_new() {
        return Ok(false);
    }
    Ok(domain.identical(&ptsol.vertex()?, vtx)?
        && same_arc(&ptsol.arc(), arc)
        && (ptsol.parameter() - para).abs() <= toler)
}

/// `ptsol.Arc() == A`: the two handles point at the same arc.
fn same_arc(first: &TheArc, second: &TheArc) -> bool {
    first == second
}

/// `PointProcess(Pt, Para, A, Domain, pnt, Tol, Range)`: the index in `pnt` of the
/// solution point at `para`. A point on a vertex of the arc is merged with that vertex
/// when it is already in the list; an inner point is merged with a new point of the arc
/// at the same parameter; otherwise the point is appended.
///
/// The native `ptsol` is one variable that the loops copy points into, and the
/// four-argument `SetValue` keeps its vertex, so the appended point carries the vertex of
/// the last point copied; the port keeps the same copies.
pub(super) fn point_process<T: SearchOnBoundariesTypes>(
    pt: Point3,
    para: f64,
    (arc, domain): (&TheArc, &T::TopolTool),
    pnt: &mut Vec<T::PathPoint>,
    tol: f64,
) -> Result<i32, Failure> {
    let mut ptsol: T::PathPoint = T::PathPoint::default();
    if let Some(range) = on_vertex::<T>(pt, para, (arc, domain), pnt, (tol, &mut ptsol))? {
        return Ok(range);
    }
    // jgv: do not add the extremities of a segment when they already exist.
    let mut range: Option<i32> = None;
    for (k, point) in pnt.iter().enumerate() {
        ptsol = point.clone();
        if !same_arc(&ptsol.arc(), arc) || !ptsol.is_new() {
            continue;
        }
        if (ptsol.parameter() - para).abs() <= precision::p_confusion() {
            range = Some(k as i32 + 1);
        }
    }
    if let Some(range) = range {
        return Ok(range);
    }
    // #24643: the tolerance of a new point is 1000 times the given one, at most 0.005.
    let mut new_tol: f64 = tol;
    new_tol *= 1000.0;
    if new_tol > 0.005 {
        new_tol = 0.005;
    }
    ptsol.set_value_on_arc(pt, new_tol, arc, para);
    pnt.push(ptsol);
    Ok(pnt.len() as i32)
}

/// The vertex part of `PointProcess`: the index of the point when `para` falls on a
/// vertex of the arc (appending it when it is not in the list yet), `None` otherwise.
fn on_vertex<T: SearchOnBoundariesTypes>(
    pt: Point3,
    para: f64,
    (arc, domain): (&TheArc, &T::TopolTool),
    pnt: &mut Vec<T::PathPoint>,
    (tol, ptsol): (f64, &mut T::PathPoint),
) -> Result<Option<i32>, Failure> {
    domain.initialize_arc(arc)?;
    domain.init_vertex_iterator();
    while domain.more_vertex() {
        let vtx: TheVertex = domain.vertex()?;
        let dist: f64 = (para - T::SOBTool::parameter(&vtx, arc)?).abs();
        let toler: f64 = T::SOBTool::tolerance(&vtx, arc)?;
        if dist <= toler {
            for (k, point) in pnt.iter().enumerate() {
                *ptsol = point.clone();
                if is_known_vertex::<T>(domain, ptsol, (&vtx, arc), para, toler)? {
                    return Ok(Some(k as i32 + 1));
                }
            }
            ptsol.set_value_on_vertex(pt, tol, &vtx, arc, para);
            pnt.push(ptsol.clone());
            return Ok(Some(pnt.len() as i32));
        }
        domain.next_vertex();
    }
    Ok(None)
}
