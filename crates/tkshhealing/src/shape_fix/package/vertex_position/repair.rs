use super::grouping::VertexEdgeGroups;
use super::*;
use crate::shape_build::ShapeBuildEdge;
use rustycad_tkbrep::topexp;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::Orientation;
pub(super) fn apply_groups(
    vertex: &Vertex,
    point: Point3,
    tolerance: f64,
    context: &Handle<ShapeBuildReShape>,
    mut groups: VertexEdgeGroups,
) -> Result<bool, Failure> {
    let reshape = context.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "ShapeFix::FixVertexPosition: null context; expected a ShapeBuild_ReShape",
            "",
        )
    })?;
    let (mut changed, added) = suitable_edges(vertex, tolerance, reshape, &mut groups)?;
    if added && point.distance(&groups.center) > tolerance {
        let mut replacement = make_vertex(groups.center)?;
        replacement.set_orientation(vertex.orientation());
        reshape.replace(vertex, &replacement)?;
        changed = true;
    }
    for edge in &groups.rejected {
        changed |= reject_edge(edge, vertex, reshape)?;
    }
    Ok(changed)
}
fn suitable_edges(
    vertex: &Vertex,
    tolerance: f64,
    context: &ShapeBuildReShape,
    groups: &mut VertexEdgeGroups,
) -> Result<(bool, bool), Failure> {
    let (mut changed, mut added) = (false, false);
    for (index, old) in groups.suitable.iter().enumerate() {
        let (first, last) = topexp::vertices(old, false);
        if !first.is_same(vertex) && !last.is_same(vertex) {
            continue;
        }
        let edge = topods::edge(&context.apply(old, ShapeEnum::Shape)?)?;
        let Some((a, b)) = curve_ends(&edge)? else {
            continue;
        };
        if index > 1 {
            groups.center = midpoint(groups.center, if first.is_same(vertex) { a } else { b });
        }
        changed |= separate_loop(&edge, (a, b), groups.center, tolerance, context)?;
        added = true;
    }
    Ok((changed, added))
}
fn separate_loop(
    edge: &Edge,
    ends: (Point3, Point3),
    center: Point3,
    tolerance: f64,
    context: &ShapeBuildReShape,
) -> Result<bool, Failure> {
    let (first, last) = topexp::vertices(edge, false);
    let separated = ends.0.distance(&ends.1) > tolerance;
    if !first.is_same(&last) || !separated {
        return Ok(false);
    }
    let nearer_first = ends.0.distance(&center) < ends.1.distance(&center);
    let replace_first = !nearer_first;
    let point = if replace_first { ends.0 } else { ends.1 };
    context.replace(
        edge,
        &replace_vertex(edge, point, replace_first)?.into_shape(),
    )?;
    Ok(true)
}
fn reject_edge(old: &Edge, vertex: &Vertex, context: &ShapeBuildReShape) -> Result<bool, Failure> {
    let (first, last) = topexp::vertices(old, false);
    if !first.is_same(vertex) && !last.is_same(vertex) {
        return Ok(false);
    }
    let edge = topods::edge(&context.apply(old, ShapeEnum::Shape)?)?;
    let Some((a, b)) = curve_ends(&edge)? else {
        return Ok(false);
    };
    let is_first = first.is_same(vertex);
    let mut replacement = replace_vertex(&edge, if is_first { a } else { b }, is_first)?;
    if first.is_same(&last) {
        replacement = replace_vertex(&replacement, if is_first { b } else { a }, !is_first)?;
    }
    context.replace(&edge, &replacement)?;
    Ok(true)
}
fn curve_ends(edge: &Edge) -> Result<Option<(Point3, Point3)>, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    if curve.is_null() {
        return Ok(None);
    }
    Ok(Some((curve.value(first)?, curve.value(last)?)))
}
fn make_vertex(point: Point3) -> Result<Vertex, Failure> {
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, point, precision::confusion())?;
    Ok(vertex)
}
fn replace_vertex(edge: &Edge, point: Point3, forward: bool) -> Result<Edge, Failure> {
    let mut vertex = make_vertex(point)?;
    vertex.set_orientation(if forward {
        Orientation::Forward
    } else {
        Orientation::Reversed
    });
    let (first, last) = if forward {
        (vertex, Vertex::default())
    } else {
        (Vertex::default(), vertex)
    };
    let mut source = edge.clone();
    source.set_orientation(Orientation::Forward);
    let mut replacement = ShapeBuildEdge.copy_replace_vertices(&source, &first, &last)?;
    replacement.set_orientation(edge.orientation());
    Ok(replacement)
}
