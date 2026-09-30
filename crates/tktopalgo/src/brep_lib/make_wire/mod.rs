// Rust translation of BRepLib_MakeWire.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_MakeWire`: a wire from connected edges.
mod add_edge;
mod merge;

use super::errors::BRepLibWireError;
use super::make_shape::BRepLibMakeShape;
use rustycad_tkbrep::topods::{self, Edge, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use std::ops::{Deref, DerefMut};

/// `BRepLib_MakeWire` (docs/topalgo-design.md section 7): adds edges one by one to a
/// wire. An edge must share a vertex with the wire, or have a vertex within the
/// tolerance of a wire vertex (the edge is then copied onto the merged vertex); its
/// orientation follows the free ends of the wire when they are known.
///
/// The native `myError` is uninitialized until the first Add in every constructor but
/// the default one; the port starts it at `BRepLib_EmptyWire`, the default value.
///
/// ```
/// use rustycad_tkbrep::topods::Wire;
/// use rustycad_tktopalgo::brep_lib::{BRepLibMakeWire, BRepLibWireError};
/// let maker = BRepLibMakeWire::from_wire(&Wire::default())?;
/// assert!(!maker.is_done());
/// assert_eq!(maker.error(), BRepLibWireError::EmptyWire);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepLibMakeWire {
    base: BRepLibMakeShape,
    my_error: BRepLibWireError,
    my_edge: Edge,
    my_vertex: Vertex,
    my_vertices: IndexedMapOfShape,
    // `VF` and `VL`: the free ends of an open wire, null when unknown.
    vf: Vertex,
    vl: Vertex,
}

impl Default for BRepLibMakeWire {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BRepLibMakeWire {
    type Target = BRepLibMakeShape;
    fn deref(&self) -> &BRepLibMakeShape {
        &self.base
    }
}

impl DerefMut for BRepLibMakeWire {
    fn deref_mut(&mut self) -> &mut BRepLibMakeShape {
        &mut self.base
    }
}

impl BRepLibMakeWire {
    /// `BRepLib_MakeWire()`: an empty wire, not done, `BRepLib_EmptyWire`;
    /// `BRepLibMakeWire::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepLibMakeShape::new(),
            my_error: BRepLibWireError::EmptyWire,
            my_edge: Edge::default(),
            my_vertex: Vertex::default(),
            my_vertices: IndexedMapOfShape::default(),
            vf: Vertex::default(),
            vl: Vertex::default(),
        }
    }

    /// The wire of the edges added in order.
    fn from_edges(edges: &[&Edge]) -> Result<Self, Failure> {
        let mut maker = Self::new();
        for edge in edges {
            maker.add_edge(edge)?;
        }
        Ok(maker)
    }

    /// `BRepLib_MakeWire(E)`; `BRepLibMakeWire::from_edge(&e)?`.
    pub fn from_edge(edge: &Edge) -> Result<Self, Failure> {
        Self::from_edges(&[edge])
    }

    /// `BRepLib_MakeWire(E1, E2)`; `BRepLibMakeWire::from_two_edges(&e1, &e2)?`.
    pub fn from_two_edges(e1: &Edge, e2: &Edge) -> Result<Self, Failure> {
        Self::from_edges(&[e1, e2])
    }

    /// `BRepLib_MakeWire(E1, E2, E3)`; `BRepLibMakeWire::from_three_edges(&e1, &e2, &e3)?`.
    pub fn from_three_edges(e1: &Edge, e2: &Edge, e3: &Edge) -> Result<Self, Failure> {
        Self::from_edges(&[e1, e2, e3])
    }

    /// `BRepLib_MakeWire(E1, E2, E3, E4)`;
    /// `BRepLibMakeWire::from_four_edges(&e1, &e2, &e3, &e4)?`.
    pub fn from_four_edges(e1: &Edge, e2: &Edge, e3: &Edge, e4: &Edge) -> Result<Self, Failure> {
        Self::from_edges(&[e1, e2, e3, e4])
    }

    /// `BRepLib_MakeWire(W)`: the edges of `W`; a null wire adds nothing (not done,
    /// OCC30708); `BRepLibMakeWire::from_wire(&w)?`.
    pub fn from_wire(wire: &Wire) -> Result<Self, Failure> {
        let mut maker = Self::new();
        maker.add_wire(wire)?;
        Ok(maker)
    }

    /// `BRepLib_MakeWire(W, E)`; `BRepLibMakeWire::from_wire_edge(&w, &e)?`.
    pub fn from_wire_edge(wire: &Wire, edge: &Edge) -> Result<Self, Failure> {
        let mut maker = Self::from_wire(wire)?;
        maker.add_edge(edge)?;
        Ok(maker)
    }

    /// `Add(W)`: adds the edges of `W` in order, up to the first that fails;
    /// Standard_TypeMismatch when a child of `W` is not an edge; `maker.add_wire(&w)?`.
    pub fn add_wire(&mut self, wire: &Wire) -> Result<(), Failure> {
        let mut iterator = ShapeIterator::new(wire, true, true);
        while let Ok(child) = iterator.value() {
            let edge: Edge = topods::edge(child)?;
            self.add_edge(&edge)?;
            if self.my_error != BRepLibWireError::WireDone {
                break;
            }
            iterator.next();
        }
        Ok(())
    }

    /// `Add(E)`: adds `E`, checking the geometric proximity of its vertices;
    /// `maker.add_edge(&e)?`.
    pub fn add_edge(&mut self, edge: &Edge) -> Result<(), Failure> {
        self.add_edge_checked(edge, true)
    }

    /// `Error()`: the status of the last Add; `maker.error()`.
    pub fn error(&self) -> BRepLibWireError {
        self.my_error
    }

    /// `Wire()` and `operator TopoDS_Wire()`: the wire; StdFail_NotDone when not done;
    /// `maker.wire()?`.
    pub fn wire(&self) -> Result<Wire, Failure> {
        topods::wire(self.shape()?)
    }

    /// `Edge()`: the last edge added (a copy when it was merged on wire vertices);
    /// `maker.edge()`.
    pub fn edge(&self) -> &Edge {
        &self.my_edge
    }

    /// `Vertex()`: the last connecting vertex; `maker.vertex()`.
    pub fn vertex(&self) -> &Vertex {
        &self.my_vertex
    }
}
