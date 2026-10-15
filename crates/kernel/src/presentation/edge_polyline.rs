// SPDX-License-Identifier: GPL-2.0-only
//! Edge display polylines read from the polygons BRepMesh stores on face meshes.
//!
//! Each edge takes its polygon from the first face, in stable face order, that
//! bounds it, so its points coincide with vertices of that face mesh.
use super::detached::{native, refused};
use super::face_mesh::{MeshedFace, mesh_point};
use super::mesh::EdgePolyline;
use crate::query_error::{BodyQueryError, BodyQueryFailure, QueryEntity};
use crate::topology::EdgeIndex;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The polylines of the non-degenerated edges and the degenerated edge indices.
pub(crate) struct EdgeDisplay {
    pub polylines: Vec<EdgePolyline>,
    pub degenerated: Vec<EdgeIndex>,
}

/// The polylines of `edges` (images by stable index) on `faces` (stable order).
pub(crate) fn edge_display(
    edges: &[Edge],
    faces: &[MeshedFace],
) -> Result<EdgeDisplay, BodyQueryError> {
    let lookup = edge_lookup(edges);
    let mut found: Vec<Option<Vec<[f64; 3]>>> = vec![None; edges.len()];
    for face in faces {
        collect_face_polygons(face, &lookup, &mut found)?;
    }
    let mut display = EdgeDisplay {
        polylines: Vec::new(),
        degenerated: Vec::new(),
    };
    for (position, (edge, points)) in edges.iter().zip(found).enumerate() {
        classify_edge(&mut display, EdgeIndex(position as u32), edge, points)?;
    }
    Ok(display)
}

fn edge_lookup(edges: &[Edge]) -> IndexedMapOfShape {
    let mut lookup = IndexedMapOfShape::default();
    for edge in edges {
        lookup.add(Shape::clone(edge));
    }
    lookup
}

fn collect_face_polygons(
    face: &MeshedFace,
    lookup: &IndexedMapOfShape,
    found: &mut [Option<Vec<[f64; 3]>>],
) -> Result<(), BodyQueryError> {
    let mut explorer = Explorer::new(&face.face, ShapeEnum::Edge, ShapeEnum::Shape)
        .map_err(|failure| native(QueryEntity::Body, "explore face edges", &failure))?;
    while explorer.more() {
        let position = lookup.find_index(explorer.current()).wrapping_sub(1);
        if found.get(position).is_some_and(Option::is_none) {
            found[position] = face_polygon(face, explorer.current(), position)?;
        }
        explorer
            .next()
            .map_err(|failure| native(QueryEntity::Body, "next face edge", &failure))?;
    }
    Ok(())
}

/// The 3D points of the polygon of `edge` on the face mesh, or None without one.
fn face_polygon(
    face: &MeshedFace,
    edge: &Shape,
    position: usize,
) -> Result<Option<Vec<[f64; 3]>>, BodyQueryError> {
    let entity = QueryEntity::Edge(EdgeIndex(position as u32));
    let edge = topods::edge(edge).map_err(|failure| native(entity, "edge", &failure))?;
    let polygon = BRepTool::polygon_on_triangulation(&edge, &face.triangulation, &face.location)
        .map_err(|failure| native(entity, "edge polygon", &failure))?;
    let (Some(polygon), Some(mesh)) = (polygon.get(), face.triangulation.get()) else {
        return Ok(None);
    };
    let transform = face.location.transformation();
    (1..=polygon.nb_nodes())
        .map(|rank| {
            let node = polygon
                .node(rank)
                .map_err(|failure| native(entity, "polygon node", &failure))?;
            mesh_point(mesh, node, transform, entity)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn classify_edge(
    display: &mut EdgeDisplay,
    index: EdgeIndex,
    edge: &Edge,
    points: Option<Vec<[f64; 3]>>,
) -> Result<(), BodyQueryError> {
    let entity = QueryEntity::Edge(index);
    let degenerated = BRepTool::degenerated(edge)
        .map_err(|failure| native(entity, "edge degeneration", &failure))?;
    if degenerated {
        display.degenerated.push(index);
        return Ok(());
    }
    let Some(points) = points.filter(|points| points.len() >= 2) else {
        let cause = "edge has no polygon on an adjacent face mesh: expected two or more points";
        return Err(refused(entity, BodyQueryFailure::MissingEdgePolygon, cause));
    };
    let edge = index;
    display.polylines.push(EdgePolyline { edge, points });
    Ok(())
}
