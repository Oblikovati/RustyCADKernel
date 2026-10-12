//! Native boundary arrays retain indexed edge polylines and nonindexed triangulation-only extras.
use super::{StdPrsShadedShape, StdPrsWfShape};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::{self, Explorer},
    topods::{self, Edge, Face, Shape},
    toptools::IndexedDataMapOfShapeListOfShape,
};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{
    geom_abs::Shape as Continuity,
    gp::Point3,
    poly::{PolygonOnTriangulation, Triangulation, mesh_purpose},
    toploc::Location,
};
use rustycad_tkservice::graphic3d::*;

struct ShadedBoundarySource {
    edge: Edge,
    face: Face,
    neighbors: usize,
}
struct ShadedBoundaryPolygon {
    polygon: Handle<PolygonOnTriangulation>,
    mesh: Handle<Triangulation>,
    location: Location,
}
impl StdPrsShadedShape {
    /// Native default continuity limit includes all edges; `Self::fill_face_boundaries(&shape)?`.
    pub fn fill_face_boundaries(
        shape: &Shape,
    ) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
        let continuity = Continuity::CN;
        Self::fill_face_boundaries_with_continuity(shape, continuity)
    }
    /// Preserve native continuity filtering and array capacities; `Self::fill_face_boundaries_with_continuity(&shape,Continuity::C1)?`.
    pub fn fill_face_boundaries_with_continuity(
        shape: &Shape,
        upper: Continuity,
    ) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
        let extras = shaded_boundary_extras(shape)?;
        let sources = shaded_boundary_sources(shape)?;
        let (nodes, polylines) = shaded_boundary_capacity(&sources, upper)?;
        let array = allocated_shaded_boundaries(nodes, polylines, extras.length())?;
        if array.is_null() {
            return Ok(array);
        }
        if nodes == 0 {
            return append_nonindexed_shaded_extras(array, &extras);
        }
        append_shaded_boundary_polygons(&array, &sources, upper)?;
        append_indexed_shaded_extras(&array, &extras)?;
        Ok(array)
    }
}
fn shaded_boundary_extras(shape: &Shape) -> Result<Sequence<Point3>, Failure> {
    let mut extras = Sequence::default();
    for entry in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        if entry.nb_children() != 0 {
            continue;
        }
        StdPrsWfShape::append_edges_on_triangulation(&mut extras, &entry, false)?;
    }
    Ok(extras)
}
fn shaded_boundary_sources(shape: &Shape) -> Result<Vec<ShadedBoundarySource>, Failure> {
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut map)?;
    let mut sources = Vec::new();
    for index in 1..=map.extent() {
        let neighbors = map.find_from_index(index)?;
        if neighbors.is_empty() {
            continue;
        }
        sources.push(ShadedBoundarySource {
            edge: topods::edge(map.find_key(index)?)?,
            face: topods::face(neighbors.first_checked()?)?,
            neighbors: neighbors.length() as usize,
        });
    }
    Ok(sources)
}
fn qualified_shaded_boundary(
    source: &ShadedBoundarySource,
    upper: Continuity,
) -> Result<Option<ShadedBoundaryPolygon>, Failure> {
    let (mesh, location) = BRepTool::triangulation(&source.face, mesh_purpose::NONE)?;
    if mesh.is_null() || shaded_boundary_exceeds_continuity(source, upper)? {
        return Ok(None);
    }
    let polygon = BRepTool::polygon_on_triangulation(&source.edge, &mesh, &location)?;
    if polygon.is_null() || polygon.nodes().len() < 2 {
        return Ok(None);
    }
    Ok(Some(ShadedBoundaryPolygon {
        polygon,
        mesh,
        location,
    }))
}
fn shaded_boundary_capacity(
    sources: &[ShadedBoundarySource],
    upper: Continuity,
) -> Result<(i32, i32), Failure> {
    let mut capacity = (0, 0);
    for source in sources {
        if let Some(boundary) = qualified_shaded_boundary(source, upper)? {
            capacity.0 += boundary.polygon.nodes().len() as i32;
            capacity.1 += 1;
        }
    }
    Ok(capacity)
}
fn allocated_shaded_boundaries(
    nodes: i32,
    polylines: i32,
    extras: i32,
) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
    if nodes == 0 && extras < 2 {
        return Ok(Handle::default());
    }
    let indices = if nodes == 0 {
        0
    } else {
        (nodes - polylines) * 2 + extras
    };
    Ok(Handle::new(Graphic3dArrayOfSegments::new(
        nodes + extras,
        indices,
        0,
    )?))
}
fn append_shaded_boundary_polygons(
    array: &Graphic3dArrayOfSegments,
    sources: &[ShadedBoundarySource],
    upper: Continuity,
) -> Result<(), Failure> {
    // Repeat native property queries after capacity allocation; aliases can expose native failures.
    for source in sources {
        if let Some(boundary) = qualified_shaded_boundary(source, upper)? {
            append_shaded_boundary_polyline(array, &boundary)?;
        }
    }
    Ok(())
}
fn append_shaded_boundary_polyline(
    array: &Graphic3dArrayOfSegments,
    boundary: &ShadedBoundaryPolygon,
) -> Result<(), Failure> {
    let mut previous = 0;
    for index in boundary.polygon.nodes().iter() {
        let point = boundary
            .mesh
            .node(*index)?
            .transformed(boundary.location.transformation());
        let current = array.add_vertex_pnt(&point)?;
        if previous != 0 {
            array.add_edges2(previous, current)?;
        }
        previous = current;
    }
    Ok(())
}
fn append_nonindexed_shaded_extras(
    array: Handle<Graphic3dArrayOfSegments>,
    extras: &Sequence<Point3>,
) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
    for point in extras.iter() {
        array.add_vertex_pnt(point)?;
    }
    Ok(array)
}
fn append_indexed_shaded_extras(
    array: &Graphic3dArrayOfSegments,
    extras: &Sequence<Point3>,
) -> Result<(), Failure> {
    for point in extras.iter() {
        let index = array.add_vertex_pnt(point)?;
        array.add_edge(index)?;
    }
    Ok(())
}

fn shaded_boundary_exceeds_continuity(
    source: &ShadedBoundarySource,
    upper: Continuity,
) -> Result<bool, Failure> {
    Ok((upper as i32) < (Continuity::CN as i32)
        && source.neighbors >= 2
        && (BRepTool::max_continuity(&source.edge)? as i32) > (upper as i32))
}
