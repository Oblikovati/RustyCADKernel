//! The vertex merge of `rebuildShell` (ShapeFix_FaceConnect): the groups of vertices
//! replacing each other, each group's kept vertex moved to the middle of their bounding
//! box with a tolerance covering all of them, and the others replaced by it.
use super::ShapeListMap;
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{self, Shape, Shell};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{FlatDataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point3, Xyz};

/// `theRepVertices`: the new vertices of each old vertex.
pub(super) type VertexReplacements = ShapeListMap;

type VertexMap = FlatDataMap<Shape, Shape, ShapeMapHasher>;

/// "Prepare vertices to replace": `theOldVertices` (each vertex to its kept vertex) and
/// `theNewVertices` (each kept vertex to the vertices it replaces).
fn vertex_groups(replacements: &VertexReplacements) -> Result<(VertexMap, ShapeListMap), Failure> {
    let mut old_vertices = VertexMap::default();
    let mut new_vertices = ShapeListMap::default();
    for (old, list) in replacements.items() {
        let mut new_list = List::default();
        for new in list.iter() {
            if old_vertices.try_bind(new.clone(), old.clone()) {
                new_list.append(new.clone());
                continue;
            }
            // "Vertex has a replacing vertex in the map".
            let rep = old_vertices.find(new)?.clone();
            if rep.is_same(old) {
                continue;
            }
            old_vertices.bound(rep.clone(), old.clone());
            new_list.append(rep.clone());
            let related: Vec<Shape> = new_vertices.find(&rep)?.iter().cloned().collect();
            for aux in related {
                *old_vertices.change_find(&aux)? = old.clone();
                new_list.append(aux);
            }
            new_vertices.unbind(&rep);
        }
        new_vertices.try_bind(old.clone(), new_list);
    }
    Ok((old_vertices, new_vertices))
}

/// The bounding box of the vertices (the native `else if`: a coordinate below the lower
/// bound never raises the upper one).
fn bounds(start: Point3, others: &[Point3]) -> (Xyz, Xyz) {
    let coords = |p: Point3| [p.x(), p.y(), p.z()];
    let (mut low, mut high) = (coords(start), coords(start));
    for p in others {
        for (index, value) in coords(*p).into_iter().enumerate() {
            if value < low[index] {
                low[index] = value;
            } else if value > high[index] {
                high[index] = value;
            }
        }
    }
    (
        Xyz::new(low[0], low[1], low[2]),
        Xyz::new(high[0], high[1], high[2]),
    )
}

/// "Update vertices positions and tolerances".
fn update_kept_vertex(kept: &Shape, others: &List<Shape>) -> Result<(), Failure> {
    let vertex = topods::vertex(kept)?;
    let mut points = Vec::new();
    for other in others.iter() {
        points.push(BRepTool::pnt(&topods::vertex(other)?)?);
    }
    let (low, high) = bounds(BRepTool::pnt(&vertex)?, &points);
    let middle = (low + high) / 2.;
    let position = Point3::new(middle.x(), middle.y(), middle.z());
    let mut tolerance = 0.0_f64;
    for (other, point) in others.iter().zip(&points) {
        let current =
            position.distance(point) + BRepTool::vertex_tolerance(&topods::vertex(other)?)?;
        tolerance = tolerance.max(current);
    }
    let own = position.distance(&BRepTool::pnt(&vertex)?) + BRepTool::vertex_tolerance(&vertex)?;
    tolerance = tolerance.max(own);
    BRepBuilder::default().update_vertex_point(&vertex, position, tolerance)
}

/// The vertex groups merged in the shell through the context.
pub(super) fn merge_vertices(
    shell: &Shell,
    replacements: &VertexReplacements,
    context: &ShapeBuildReShape,
) -> Result<Shell, Failure> {
    let (old_vertices, new_vertices) = vertex_groups(replacements)?;
    for (kept, others) in new_vertices.items() {
        update_kept_vertex(kept, others)?;
    }
    context.clear();
    for (old, new) in old_vertices.items() {
        context.replace(
            &old.oriented(Orientation::Forward),
            &new.oriented(Orientation::Forward),
        )?;
    }
    topods::shell(&context.apply(shell.as_ref(), ShapeEnum::Shape)?)
}
