// Rust translation of BRepBuilderAPI_MakePolygon.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_MakePolygon`: a polygonal wire through points or vertices, built by
//! the wrapped `BRepLib_MakePolygon`.
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::wrap::{self, wrapped_constructors};
use crate::brep_lib::BRepLibMakePolygon;
use rustycad_tkbrep::topods::{Edge, Vertex, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// `BRepBuilderAPI_MakePolygon` (docs/topalgo-design.md section 10): each constructor
/// and Add runs the `BRepLib_MakePolygon` member of the same Rust name; `IsDone` is the
/// flag of the BRepLib maker.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape};
/// let mut maker = BRepBuilderAPIMakePolygon::new();
/// maker.add_point(Point3::new(0.0, 0.0, 0.0))?;
/// assert!(!maker.is_done());
/// maker.add_point(Point3::new(1.0, 0.0, 0.0))?;
/// maker.add_point(Point3::new(1.0, 1.0, 0.0))?;
/// maker.close()?;
/// assert!(maker.is_done() && maker.wire()?.closed()?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepBuilderAPIMakePolygon {
    base: BRepBuilderAPIMakeShapeBase,
    my_make_polygon: BRepLibMakePolygon,
}

impl Default for BRepBuilderAPIMakePolygon {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPIMakePolygon {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `IsDone()`: the flag of the BRepLib maker; `maker.is_done()`.
    fn is_done(&self) -> bool {
        self.my_make_polygon.is_done()
    }
}

impl BRepBuilderAPIMakePolygon {
    /// `BRepBuilderAPI_MakePolygon()`: an empty polygon, not done;
    /// `BRepBuilderAPIMakePolygon::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_polygon: BRepLibMakePolygon::new(),
        }
    }

    /// Wraps a constructed BRepLib maker and takes its shape when it is done.
    fn wrap(my_make_polygon: BRepLibMakePolygon) -> Result<Self, Failure> {
        let mut maker = Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_polygon,
        };
        wrap::adopt(&mut maker.base, &maker.my_make_polygon)?;
        Ok(maker)
    }

    wrapped_constructors!(BRepBuilderAPIMakePolygon, BRepLibMakePolygon;
        from_two_points(p1: Point3, p2: Point3) = "BRepBuilderAPI_MakePolygon(P1, P2)";
        from_three_points(p1: Point3, p2: Point3, p3: Point3, close: bool)
            = "BRepBuilderAPI_MakePolygon(P1, P2, P3, Close)";
        from_four_points(p1: Point3, p2: Point3, p3: Point3, p4: Point3, close: bool)
            = "BRepBuilderAPI_MakePolygon(P1, P2, P3, P4, Close)";
        from_two_vertices(v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakePolygon(V1, V2)";
        from_three_vertices(v1: &Vertex, v2: &Vertex, v3: &Vertex, close: bool)
            = "BRepBuilderAPI_MakePolygon(V1, V2, V3, Close)";
        from_four_vertices(v1: &Vertex, v2: &Vertex, v3: &Vertex, v4: &Vertex, close: bool)
            = "BRepBuilderAPI_MakePolygon(V1, V2, V3, V4, Close)";
    );

    /// `Add(P)`: adds a point; when the BRepLib maker is done, the command is done, and
    /// takes the wire only when a last vertex exists (as natively);
    /// `maker.add_point(p)?`.
    pub fn add_point(&mut self, point: Point3) -> Result<(), Failure> {
        self.my_make_polygon.add_point(point)?;
        if !self.my_make_polygon.is_done() {
            return Ok(());
        }
        self.base.set_done();
        if !self.last_vertex().is_null() {
            self.base.my_shape = self.my_make_polygon.shape()?.clone();
        }
        Ok(())
    }

    /// `Add(V)`: adds a vertex and takes the wire when the BRepLib maker is done;
    /// `maker.add_vertex(&v)?`.
    pub fn add_vertex(&mut self, vertex: &Vertex) -> Result<(), Failure> {
        self.my_make_polygon.add_vertex(vertex)?;
        wrap::adopt(&mut self.base, &self.my_make_polygon)
    }

    /// `Added()`: whether the last Add made an edge; `maker.added()`.
    pub fn added(&self) -> bool {
        self.my_make_polygon.added()
    }

    /// `Close()`: closes the polygon and takes its shape whatever the done flag, so a
    /// polygon that is not done raises StdFail_NotDone (from `Shape()`, as natively);
    /// `maker.close()?`.
    pub fn close(&mut self) -> Result<(), Failure> {
        self.my_make_polygon.close()?;
        self.base.my_shape = self.my_make_polygon.shape()?.clone();
        Ok(())
    }

    /// `FirstVertex()`: the first vertex (null before the first Add);
    /// `maker.first_vertex()`.
    pub fn first_vertex(&self) -> &Vertex {
        self.my_make_polygon.first_vertex()
    }

    /// `LastVertex()`: the last vertex (null before the second Add);
    /// `maker.last_vertex()`.
    pub fn last_vertex(&self) -> &Vertex {
        self.my_make_polygon.last_vertex()
    }

    /// `Edge()` and `operator TopoDS_Edge()`: the last edge made (null when the last Add
    /// made none); `maker.edge()`.
    pub fn edge(&self) -> &Edge {
        self.my_make_polygon.edge()
    }

    /// `Wire()` and `operator TopoDS_Wire()`: the wire of the BRepLib maker;
    /// StdFail_NotDone when it is not done; `maker.wire()?`.
    pub fn wire(&self) -> Result<Wire, Failure> {
        self.my_make_polygon.wire()
    }
}
