//! Fuses coincident vertex hits into common topology for intersecting edge blocks.
use super::EdgeFaceCandidate;
use super::PendingEFVertex;
use crate::bop_tools::make_vertex;
use crate::int_tools::{CommonPrt, Context};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_adaptor::BRepAdaptorCurve,
    topods::Vertex,
};
use rustycad_tkernel::{ncollection::List, standard::Failure};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};

pub(super) fn update_vertex_tolerance(
    candidate: &EdgeFaceCandidate,
    vertex: &Vertex,
) -> Result<(), Failure> {
    let tolerance = base_vertex_tolerance(candidate, vertex)?;
    BRepBuilder::default().update_vertex_tolerance(vertex, tolerance)
}

pub(super) fn candidate_tolerance(
    candidate: &EdgeFaceCandidate,
    common: &CommonPrt,
    vertex: &Vertex,
    context: &Context,
) -> Result<f64, Failure> {
    let base = base_vertex_tolerance(candidate, vertex)?;
    if !is_line_plane(candidate, context)? {
        return Ok(base);
    }
    let (first, last) = common.range1_bounds();
    Ok(base.max((last - first) / 2.0))
}

fn base_vertex_tolerance(candidate: &EdgeFaceCandidate, vertex: &Vertex) -> Result<f64, Failure> {
    let vertex_tolerance = BRepTool::vertex_tolerance(vertex)?;
    let edge_tolerance = BRepTool::edge_tolerance(&candidate.edge)?;
    let face_tolerance = BRepTool::face_tolerance(&candidate.face)?;
    Ok(vertex_tolerance.max(edge_tolerance).max(face_tolerance))
}

fn is_line_plane(candidate: &EdgeFaceCandidate, context: &Context) -> Result<bool, Failure> {
    let curve = BRepAdaptorCurve::with_edge(&candidate.edge)?;
    if curve.get_type()? != CurveType::Line {
        return Ok(false);
    }
    let surface = context.surface_adaptor(&candidate.face)?;
    Ok(surface.read().get_type()? == SurfaceType::Plane)
}

pub(super) fn pending_vertex_groups(
    pending: &[PendingEFVertex],
    fuzzy: f64,
) -> Result<Vec<Vec<usize>>, Failure> {
    let mut groups = Vec::new();
    for current in 0..pending.len() {
        let joined = overlapping_groups(current, &groups, pending, fuzzy)?;
        merge_pending_groups(&mut groups, current, &joined);
    }
    Ok(groups)
}

fn overlapping_groups(
    current: usize,
    groups: &[Vec<usize>],
    pending: &[PendingEFVertex],
    fuzzy: f64,
) -> Result<Vec<usize>, Failure> {
    let mut joined = Vec::new();
    for (index, group) in groups.iter().enumerate() {
        if group_intersects(current, group, pending, fuzzy)? {
            joined.push(index);
        }
    }
    Ok(joined)
}

fn group_intersects(
    current: usize,
    group: &[usize],
    pending: &[PendingEFVertex],
    fuzzy: f64,
) -> Result<bool, Failure> {
    for other in group {
        if vertices_intersect(&pending[current], &pending[*other], fuzzy)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn vertices_intersect(
    first: &PendingEFVertex,
    second: &PendingEFVertex,
    fuzzy: f64,
) -> Result<bool, Failure> {
    let first_tolerance = BRepTool::vertex_tolerance(&first.vertex)?.max(first.tolerance);
    let second_tolerance = BRepTool::vertex_tolerance(&second.vertex)?.max(second.tolerance);
    let point1 = BRepTool::pnt(&first.vertex)?;
    let point2 = BRepTool::pnt(&second.vertex)?;
    let tolerance = first_tolerance + second_tolerance + fuzzy;
    Ok(point1.square_distance(&point2) < tolerance * tolerance)
}

fn merge_pending_groups(groups: &mut Vec<Vec<usize>>, current: usize, joined: &[usize]) {
    if joined.is_empty() {
        groups.push(vec![current]);
        return;
    }
    let keep = joined[0];
    groups[keep].push(current);
    for index in joined.iter().skip(1).rev() {
        let absorbed = groups.remove(*index);
        groups[keep].extend(absorbed);
    }
}

pub(super) fn fused_vertex(
    pending: &[PendingEFVertex],
    group: &[usize],
) -> Result<Vertex, Failure> {
    let mut shapes: List<rustycad_tkbrep::topods::Shape> = List::default();
    for index in group {
        shapes.append(pending[*index].vertex.clone().into_shape());
    }
    let mut vertex = Vertex::default();
    make_vertex(&shapes, &mut vertex)?;
    Ok(vertex)
}
