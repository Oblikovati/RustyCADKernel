// Rust translation of BRepLib_MakePolygon.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_MakePolygon`: a wire of segments through points or vertices.
use super::make_edge::BRepLibMakeEdge;
use super::make_shape::BRepLibMakeShape;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::{self, Edge, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use std::ops::{Deref, DerefMut};

/// `BRepLib_MakePolygon` (docs/topalgo-design.md section 7): a polygonal wire. Each
/// added point or vertex after the first adds the segment from the last vertex; a
/// vertex that `BRepTools::Compare` finds equal to the first one closes the wire.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_lib::BRepLibMakePolygon;
/// let (a, b, c) = (Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0));
/// let polygon = BRepLibMakePolygon::from_three_points(a, b, c, true)?;
/// assert!(polygon.is_done());
/// assert!(polygon.wire()?.closed()?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct BRepLibMakePolygon {
    base: BRepLibMakeShape,
    my_first_vertex: Vertex,
    my_last_vertex: Vertex,
    my_edge: Edge,
}

impl Deref for BRepLibMakePolygon {
    type Target = BRepLibMakeShape;
    fn deref(&self) -> &BRepLibMakeShape {
        &self.base
    }
}

impl DerefMut for BRepLibMakePolygon {
    fn deref_mut(&mut self) -> &mut BRepLibMakeShape {
        &mut self.base
    }
}

impl BRepLibMakePolygon {
    /// `BRepLib_MakePolygon()`: an empty polygon, not done; `BRepLibMakePolygon::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The polygon through `points`, closed when `close`.
    fn through_points(points: &[Point3], close: bool) -> Result<Self, Failure> {
        let mut polygon = Self::new();
        for point in points {
            polygon.add_point(*point)?;
        }
        if close {
            polygon.close()?;
        }
        Ok(polygon)
    }

    /// The polygon through `vertices`, closed when `close`.
    fn through_vertices(vertices: &[&Vertex], close: bool) -> Result<Self, Failure> {
        let mut polygon = Self::new();
        for vertex in vertices {
            polygon.add_vertex(vertex)?;
        }
        if close {
            polygon.close()?;
        }
        Ok(polygon)
    }

    /// `BRepLib_MakePolygon(P1, P2)`; `BRepLibMakePolygon::from_two_points(a, b)?`.
    pub fn from_two_points(p1: Point3, p2: Point3) -> Result<Self, Failure> {
        Self::through_points(&[p1, p2], false)
    }

    /// `BRepLib_MakePolygon(P1, P2, P3, Close)`;
    /// `BRepLibMakePolygon::from_three_points(a, b, c, false)?`.
    pub fn from_three_points(
        p1: Point3,
        p2: Point3,
        p3: Point3,
        close: bool,
    ) -> Result<Self, Failure> {
        Self::through_points(&[p1, p2, p3], close)
    }

    /// `BRepLib_MakePolygon(P1, P2, P3, P4, Close)`;
    /// `BRepLibMakePolygon::from_four_points(a, b, c, d, true)?`.
    pub fn from_four_points(
        p1: Point3,
        p2: Point3,
        p3: Point3,
        p4: Point3,
        close: bool,
    ) -> Result<Self, Failure> {
        Self::through_points(&[p1, p2, p3, p4], close)
    }

    /// `BRepLib_MakePolygon(V1, V2)`; `BRepLibMakePolygon::from_two_vertices(&v1, &v2)?`.
    pub fn from_two_vertices(v1: &Vertex, v2: &Vertex) -> Result<Self, Failure> {
        Self::through_vertices(&[v1, v2], false)
    }

    /// `BRepLib_MakePolygon(V1, V2, V3, Close)`;
    /// `BRepLibMakePolygon::from_three_vertices(&v1, &v2, &v3, false)?`.
    pub fn from_three_vertices(
        v1: &Vertex,
        v2: &Vertex,
        v3: &Vertex,
        close: bool,
    ) -> Result<Self, Failure> {
        Self::through_vertices(&[v1, v2, v3], close)
    }

    /// `BRepLib_MakePolygon(V1, V2, V3, V4, Close)`;
    /// `BRepLibMakePolygon::from_four_vertices(&v1, &v2, &v3, &v4, true)?`.
    pub fn from_four_vertices(
        v1: &Vertex,
        v2: &Vertex,
        v3: &Vertex,
        v4: &Vertex,
        close: bool,
    ) -> Result<Self, Failure> {
        Self::through_vertices(&[v1, v2, v3, v4], close)
    }

    /// `Add(P)`: adds a vertex at `P` of tolerance Precision::Confusion();
    /// `polygon.add_point(p)?`.
    pub fn add_point(&mut self, point: Point3) -> Result<(), Failure> {
        let mut vertex = Vertex::default();
        BRepBuilder::default().make_vertex_at(&mut vertex, point, precision::confusion())?;
        self.add_vertex(&vertex)
    }

    /// `Add(V)`: the first vertex, or the segment from the last vertex to `V` (to the
    /// first vertex when they compare equal, which closes the wire). A segment that
    /// `BRepLib_MakeEdge` cannot build is skipped and the last vertex restored; a
    /// closed flag set on the way stays; `polygon.add_vertex(&v)?`.
    pub fn add_vertex(&mut self, vertex: &Vertex) -> Result<(), Failure> {
        if self.my_first_vertex.is_null() {
            self.my_first_vertex = vertex.clone();
            return Ok(());
        }
        self.my_edge.nullify();
        let second: bool = self.my_last_vertex.is_null();
        let last: Vertex = if second {
            self.start_wire(vertex)?
        } else {
            self.next_vertex(vertex)?
        };
        let maker = BRepLibMakeEdge::from_vertices(&last, &self.my_last_vertex)?;
        if !maker.is_done() {
            // restore the previous last vertex
            self.my_last_vertex = if second { Vertex::default() } else { last };
            return Ok(());
        }
        self.add_edge(maker.edge()?)?;
        self.set_done();
        Ok(())
    }

    /// The second vertex: creates the open, orientable wire; returns the first vertex.
    fn start_wire(&mut self, vertex: &Vertex) -> Result<Vertex, Failure> {
        self.my_last_vertex = vertex.clone();
        let mut wire = Wire::default();
        BRepBuilder::default().make_wire(&mut wire);
        self.my_shape = wire.into_shape();
        self.my_shape.set_closed(false)?;
        self.my_shape.set_orientable(true)?;
        Ok(self.my_first_vertex.clone())
    }

    /// A later vertex: the first vertex when they compare equal (the wire becomes
    /// closed), else `vertex`; returns the previous last vertex.
    fn next_vertex(&mut self, vertex: &Vertex) -> Result<Vertex, Failure> {
        let last: Vertex = self.my_last_vertex.clone();
        if brep_tools::compare_vertices(vertex, &self.my_first_vertex)? {
            self.my_last_vertex = self.my_first_vertex.clone();
            self.my_shape.set_closed(true)?;
        } else {
            self.my_last_vertex = vertex.clone();
        }
        Ok(last)
    }

    /// Stores `edge` as the last edge and adds it to the wire.
    fn add_edge(&mut self, edge: Edge) -> Result<(), Failure> {
        self.my_edge = edge;
        BRepBuilder::default().add(&mut self.base.my_shape, &self.my_edge)
    }

    /// `Added()`: whether the last Add or Close built an edge; `polygon.added()`.
    pub fn added(&self) -> bool {
        !self.my_edge.is_null()
    }

    /// `Close()`: the segment from the last vertex to the first one, unless the wire has
    /// fewer than two vertices or is closed; `polygon.close()?`.
    pub fn close(&mut self) -> Result<(), Failure> {
        if self.my_first_vertex.is_null() || self.my_last_vertex.is_null() {
            return Ok(());
        }
        // check not already closed
        if self.my_shape.closed()? {
            return Ok(());
        }
        self.my_edge.nullify();
        let maker = BRepLibMakeEdge::from_vertices(&self.my_last_vertex, &self.my_first_vertex)?;
        if maker.is_done() {
            self.add_edge(maker.edge()?)?;
            self.my_shape.set_closed(true)?;
        }
        Ok(())
    }

    /// `FirstVertex()`: the first vertex (null before any Add); `polygon.first_vertex()`.
    pub fn first_vertex(&self) -> &Vertex {
        &self.my_first_vertex
    }

    /// `LastVertex()`: the last vertex (null before a second Add); `polygon.last_vertex()`.
    pub fn last_vertex(&self) -> &Vertex {
        &self.my_last_vertex
    }

    /// `Edge()` and `operator TopoDS_Edge()`: the last edge built (null when the last
    /// Add or Close built none); `polygon.edge()`.
    pub fn edge(&self) -> &Edge {
        &self.my_edge
    }

    /// `Wire()` and `operator TopoDS_Wire()`: the wire; StdFail_NotDone when no edge was
    /// built; `polygon.wire()?`.
    pub fn wire(&self) -> Result<Wire, Failure> {
        topods::wire(self.shape()?)
    }
}
