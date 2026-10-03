use super::*;
use rustycad_tkbrep::{topexp, toptools::MapOfShape};
use rustycad_tkernel::precision;
use std::collections::VecDeque;
pub(super) struct VertexEdgeGroups {
    pub suitable: Vec<Edge>,
    pub rejected: Vec<Edge>,
    pub center: Point3,
    reject_center: Point3,
}
impl VertexEdgeGroups {
    pub fn new(center: Point3) -> Self {
        Self {
            suitable: Vec::new(),
            rejected: Vec::new(),
            center,
            reject_center: center,
        }
    }
}
struct EdgeEnds {
    first: Vertex,
    last: Vertex,
    points: Vec<Point3>,
}
impl EdgeEnds {
    fn read(edge: &Edge, vertex: &Vertex, anchor: bool) -> Result<Option<Self>, Failure> {
        let (first, last) = topexp::vertices(edge, false);
        let (curve, lower, upper) = BRepTool::curve(edge)?;
        if curve.is_null() {
            return Ok(None);
        }
        let parameter = if vertex.is_same(&first) { lower } else { upper };
        let initial = if anchor && first.is_same(&last) && !vertex.is_same(&first) {
            Point3::default()
        } else {
            curve.value(parameter)?
        };
        let mut points = vec![initial];
        if first.is_same(&last) {
            points.push(curve.value(upper)?);
        }
        Ok(Some(Self {
            first,
            last,
            points,
        }))
    }
    fn is_loop_with(&self, edge: &Edge) -> bool {
        let (first, last) = topexp::vertices(edge, false);
        (first.is_same(&self.first) && last.is_same(&self.last))
            || (first.is_same(&self.last) && last.is_same(&self.first))
    }
}
pub(super) fn nearest_edges(
    edges: &[Edge],
    vertex: &Vertex,
    tolerance: f64,
    output: &mut VertexEdgeGroups,
) -> Result<bool, Failure> {
    let Some(anchor) = edges.first() else {
        return Ok(false);
    };
    let Some(ends) = EdgeEnds::read(anchor, vertex, true)? else {
        return Ok(false);
    };
    let mut local = VertexEdgeGroups::new(output.center);
    classify_edges(edges, vertex, tolerance, &ends, &mut local)?;
    append_groups(edges, vertex, tolerance, output, local)
}
fn append_groups(
    edges: &[Edge],
    vertex: &Vertex,
    tolerance: f64,
    output: &mut VertexEdgeGroups,
    local: VertexEdgeGroups,
) -> Result<bool, Failure> {
    let anchor = &edges[0];
    let done = !local.suitable.is_empty() || !local.rejected.is_empty();
    if local.suitable.is_empty() {
        output.rejected.push(anchor.clone());
        if done {
            nearest_edges(&edges[1..], vertex, tolerance, output)?;
        }
    } else {
        output.center = local.center;
        output.reject_center = local.reject_center;
        output.suitable.push(anchor.clone());
        output.suitable.extend(local.suitable);
        output.rejected.extend(local.rejected);
    }
    Ok(done)
}
fn classify_edges(
    edges: &[Edge],
    vertex: &Vertex,
    tolerance: f64,
    anchor: &EdgeEnds,
    groups: &mut VertexEdgeGroups,
) -> Result<(), Failure> {
    let mut visited = MapOfShape::default();
    visited.add(edges[0].clone().into_shape());
    let mut pending: VecDeque<Edge> = edges[1..].iter().cloned().collect();
    let mut loops = 0;
    while let Some(edge) = pending.pop_front() {
        if visited.contains(&edge) {
            continue;
        }
        // Native list retains its first item while moving loop edges to its tail.
        if anchor.is_loop_with(&edge) && pending.len() + 2 > loops {
            pending.push_back(edge);
            loops += 1;
            continue;
        }
        visited.add(edge.clone().into_shape());
        if let Some(ends) = EdgeEnds::read(&edge, vertex, false)? {
            classify_candidate(edge, anchor, &ends, tolerance, groups);
        }
    }
    Ok(())
}
fn classify_candidate(
    edge: Edge,
    anchor: &EdgeEnds,
    ends: &EdgeEnds,
    tolerance: f64,
    groups: &mut VertexEdgeGroups,
) {
    let (distance, center) = near_point(&anchor.points, &ends.points);
    if distance > tolerance {
        if groups.rejected.is_empty() {
            groups.reject_center = center;
        }
        groups.rejected.push(edge);
        return;
    }
    if groups.suitable.is_empty() {
        groups.center = center;
        groups.suitable.push(edge);
        return;
    }
    if anchor.points.len() == 1 || groups.center.distance(&center) < tolerance {
        groups.suitable.push(edge);
    } else {
        groups.rejected.push(edge);
    }
}
fn near_point(first: &[Point3], second: &[Point3]) -> (f64, Point3) {
    if first.len() == 1 && second.len() == 1 {
        return (first[0].distance(&second[0]), midpoint(first[0], second[0]));
    }
    let mut distance = f64::MAX;
    let mut center = Point3::default();
    for (a, b) in first
        .iter()
        .flat_map(|a| second.iter().map(move |b| (a, b)))
    {
        let candidate = a.distance(b);
        if (candidate - distance).abs() <= precision::confusion() {
            continue;
        }
        if candidate < distance {
            distance = candidate;
            center = midpoint(*a, *b);
        }
    }
    (distance, center)
}
