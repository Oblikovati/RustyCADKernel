// SPDX-License-Identifier: GPL-2.0-only
//! Stable deterministic topology enumeration of a body (ADR 0076).
//!
//! Faces, edges and vertices are numbered in the depth-first order of the body's
//! topological structure (`TopExp_Explorer` order, deduplicated with `IsSame`). The
//! order depends only on the child order stored in the topology, never on handle
//! addresses, pointer values or hash-map iteration, so the same body enumerates the
//! same way in every run and on every thread.
use crate::query_error::{BodyQuery, BodyQueryError, BodyQueryFailure, QueryEntity};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkg3d::top_abs::ShapeEnum;

macro_rules! stable_index {
    ($name:ident, $what:literal) => {
        #[doc = concat!("The zero-based stable index of a ", $what, " in [`BodyTopology`] order.")]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub u32);

        impl $name {
            /// The index as a `usize` for slice access; `index.position()`.
            pub fn position(self) -> usize {
                self.0 as usize
            }
        }
    };
}
stable_index!(FaceIndex, "face");
stable_index!(EdgeIndex, "edge");
stable_index!(VertexIndex, "vertex");

/// The faces, edges and vertices of one body in stable order.
///
/// ```
/// use rustycad_kernel::topology::{BodyTopology, FaceIndex};
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let body = BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0)?.shape()?.clone();
/// let topology = BodyTopology::enumerate(&body)?;
/// assert_eq!((topology.face_count(), topology.edge_count(), topology.vertex_count()), (6, 12, 8));
/// let first = topology.face(FaceIndex(0)).unwrap();
/// assert_eq!(topology.face_index(&first), Some(FaceIndex(0)));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct BodyTopology {
    faces: IndexedMapOfShape,
    edges: IndexedMapOfShape,
    vertices: IndexedMapOfShape,
}

impl BodyTopology {
    /// Enumerates `body`; a null body is a [`BodyQueryFailure::NullBody`] refusal.
    /// See the type example.
    pub fn enumerate(body: &Shape) -> Result<Self, BodyQueryError> {
        if body.is_null() {
            return Err(null_body(BodyQuery::Enumerate));
        }
        Ok(Self {
            faces: mapped_of_type(body, ShapeEnum::Face)?,
            edges: mapped_of_type(body, ShapeEnum::Edge)?,
            vertices: mapped_of_type(body, ShapeEnum::Vertex)?,
        })
    }

    /// The number of distinct faces; `topology.face_count()`.
    pub fn face_count(&self) -> usize {
        self.faces.extent()
    }

    /// The number of distinct edges; `topology.edge_count()`.
    pub fn edge_count(&self) -> usize {
        self.edges.extent()
    }

    /// The number of distinct vertices; `topology.vertex_count()`.
    pub fn vertex_count(&self) -> usize {
        self.vertices.extent()
    }

    /// The face at `index` as first met in the body; `topology.face(FaceIndex(0))`.
    pub fn face(&self, index: FaceIndex) -> Option<Face> {
        let shape = keyed(&self.faces, index.position())?;
        topods::face(shape).ok()
    }

    /// The edge at `index` as first met in the body; `topology.edge(EdgeIndex(0))`.
    pub fn edge(&self, index: EdgeIndex) -> Option<Edge> {
        let shape = keyed(&self.edges, index.position())?;
        topods::edge(shape).ok()
    }

    /// The vertex at `index`; `topology.vertex(VertexIndex(0))`.
    pub fn vertex(&self, index: VertexIndex) -> Option<Vertex> {
        let shape = keyed(&self.vertices, index.position())?;
        topods::vertex(shape).ok()
    }

    /// The stable index of a face that `IsSame` one of the body; `topology.face_index(&face)`.
    pub fn face_index(&self, face: &Shape) -> Option<FaceIndex> {
        position_of(&self.faces, face).map(FaceIndex)
    }

    /// The stable index of an edge of the body; `topology.edge_index(&edge)`.
    pub fn edge_index(&self, edge: &Shape) -> Option<EdgeIndex> {
        position_of(&self.edges, edge).map(EdgeIndex)
    }

    /// The stable index of a vertex of the body; `topology.vertex_index(&vertex)`.
    pub fn vertex_index(&self, vertex: &Shape) -> Option<VertexIndex> {
        position_of(&self.vertices, vertex).map(VertexIndex)
    }
}

// Published query results are read from any thread (AGENTS.md ownership rules).
const _: fn() = || {
    fn require_send_sync<T: Send + Sync>() {}
    require_send_sync::<BodyTopology>();
    require_send_sync::<FaceIndex>();
};

/// The refusal of a null body for `operation`.
pub(crate) fn null_body(operation: BodyQuery) -> BodyQueryError {
    BodyQueryError::new(
        operation,
        QueryEntity::Body,
        BodyQueryFailure::NullBody,
        "body is null: expected a shape with topology",
    )
}

fn mapped_of_type(body: &Shape, kind: ShapeEnum) -> Result<IndexedMapOfShape, BodyQueryError> {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(body, kind, &mut map).map_err(|failure| {
        let context = format!("explore {kind:?}");
        BodyQueryError::native(BodyQuery::Enumerate, QueryEntity::Body, &context, &failure)
    })?;
    Ok(map)
}

fn keyed(map: &IndexedMapOfShape, position: usize) -> Option<&Shape> {
    map.find_key(position + 1).ok()
}

fn position_of(map: &IndexedMapOfShape, shape: &Shape) -> Option<u32> {
    let index = map.find_index(shape);
    (index > 0).then(|| (index - 1) as u32)
}

#[cfg(test)]
#[path = "topology_tests.rs"]
mod tests;
