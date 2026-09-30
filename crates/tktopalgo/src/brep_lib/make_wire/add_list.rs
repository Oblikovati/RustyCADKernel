// Rust translation of BRepLib_MakeWire_1.cxx (BRepLib_MakeWire::Add(L) and its private
// helpers) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_MakeWire::Add(L)`: a list of edges added in connection order, after the
//! vertices within tolerance of each other are merged.
use super::BRepLibMakeWire;
use super::coincident::collect_coincident_vertices;
use crate::brep_lib::bounding_vertex::bounding_vertex;
use crate::brep_lib::errors::BRepLibWireError;
use crate::brep_lib::make_vertex::BRepLibMakeVertex;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

/// The map from the merged vertices to their new vertex.
type OldToNew = DataMap<Shape, Shape, ShapeMapHasher>;

impl BRepLibMakeWire {
    /// `Add(L)`: adds the edges of the list (their vertices merged first within their
    /// tolerances, the edges touching a merged vertex copied onto it), the first one (on
    /// an empty wire) with the proximity check, then every edge connected to the wire,
    /// front by front, without it; `BRepLib_DisconnectedWire` and not done when an edge
    /// of the list was never reached; `maker.add_list(&edges)?`.
    pub fn add_list(&mut self, edges: &List<Shape>) -> Result<(), Failure> {
        self.my_error = BRepLibWireError::WireDone;
        let ref_size: i32 = edges.extent();
        let mut added: i32 = 0;
        if !edges.is_empty() {
            let groups = collect_coincident_vertices(&self.my_shape, &self.my_vertices, edges)?;
            let old_to_new = self.create_new_vertices(&groups)?;
            let new_edges = create_new_list_of_edges(edges, &old_to_new)?;
            added = self.add_connected(&new_edges)?;
        }
        if added == ref_size {
            self.set_done();
        } else {
            self.set_not_done();
            self.my_error = BRepLibWireError::DisconnectedWire;
        }
        Ok(())
    }

    /// The loop of `Add(L)` over the new edges: the first one or the wire edges start
    /// the front, then the unprocessed neighbours of each front are added; returns the
    /// number of edges added.
    fn add_connected(&mut self, new_edges: &List<Shape>) -> Result<i32, Failure> {
        let mut map_ve = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.my_shape,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut map_ve,
        )?;
        let mut processed = MapOfShape::default();
        let mut active: List<Shape> = List::default();
        let mut added: i32 = 0;
        if self.my_edge.is_null() {
            // take the first edge from the list and add it
            let first: Edge = topods::edge(new_edges.first().expect("a non-empty list"))?;
            self.add_edge(&first)?;
            processed.add(first.clone().into_shape());
            active.append(first.into_shape());
            added += 1;
        } else {
            // existing edges are already connected
            for edge in Explorer::new(&self.my_shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
                processed.add(edge.clone());
                active.append(edge);
            }
        }
        for edge in new_edges.iter() {
            topexp::map_shapes_and_ancestors(
                edge,
                ShapeEnum::Vertex,
                ShapeEnum::Edge,
                &mut map_ve,
            )?;
        }
        while !active.is_empty() {
            let mut neighbours: List<Shape> = List::default();
            for edge in active.iter() {
                for vertex in Explorer::new(edge, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                    for neighbour in map_ve.find_from_key(&vertex)?.iter() {
                        if processed.contains(neighbour) {
                            continue;
                        }
                        self.add_edge_checked(&topods::edge(neighbour)?, false)?;
                        neighbours.append(neighbour.clone());
                        processed.add(neighbour.clone());
                        added += 1;
                    }
                }
            }
            active = neighbours;
        }
        Ok(added)
    }

    /// `CreateNewVertices`: for each group, the wire vertex of the group (the last one
    /// met) moved to the bounding vertex of the group, or a new vertex there; every
    /// vertex of the group maps to it.
    fn create_new_vertices(&self, groups: &[Vec<Vertex>]) -> Result<OldToNew, Failure> {
        let builder = BRepBuilder::default();
        let mut old_to_new = OldToNew::default();
        for group in groups {
            let mut new_vertex = Vertex::default();
            let mut values: List<Shape> = List::default();
            for vertex in group {
                values.append(vertex.clone().into_shape());
                if self.my_vertices.contains(vertex) {
                    new_vertex = vertex.clone();
                }
            }
            let (center, tolerance) = bounding_vertex(&values)?.unwrap_or((Point3::default(), 0.));
            if new_vertex.is_null() {
                // vertices from the original shape isn't found in this group: create the new vertex
                new_vertex = BRepLibMakeVertex::new(center)?.vertex()?;
                builder.update_vertex_tolerance(&new_vertex, tolerance)?;
            } else {
                // update already existing vertex
                builder.update_vertex_point(&new_vertex, center, tolerance)?;
            }
            for vertex in group {
                old_to_new.bind(vertex.clone().into_shape(), new_vertex.clone().into_shape());
            }
        }
        Ok(old_to_new)
    }
}

/// `CreateNewListOfEdges`: each edge of the list, or an empty copy of it on the new
/// vertices (with the orientations of the old ones) when one of its vertices maps to a
/// new one.
fn create_new_list_of_edges(
    edges: &List<Shape>,
    old_to_new: &OldToNew,
) -> Result<List<Shape>, Failure> {
    let builder = BRepBuilder::default();
    let mut new_edges: List<Shape> = List::default();
    for shape in edges.iter() {
        let edge: Edge = topods::edge(shape)?;
        let mut is_new: bool = false;
        let mut vertices: Vec<Shape> = Vec::new();
        for vertex in Explorer::new(&edge, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            match old_to_new.find(&vertex) {
                Ok(new_vertex) => {
                    is_new = true;
                    // append the new vertex
                    vertices.push(new_vertex.oriented(vertex.orientation()));
                }
                Err(_) => vertices.push(vertex),
            }
        }
        if !is_new {
            new_edges.append(edge.into_shape());
            continue;
        }
        let mut copy: Shape = edge.empty_copied()?;
        for vertex in &vertices {
            builder.add(&mut copy, vertex)?;
        }
        new_edges.append(copy);
    }
    Ok(new_edges)
}
