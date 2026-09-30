// Rust translation of BRepBuilderAPI_MakeWire.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_MakeWire`: a wire from connected edges, built by the wrapped
//! `BRepLib_MakeWire`.
use super::errors::BRepBuilderAPIWireError;
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::wrap::{wrapped_constructors, wrapped_updates};
use crate::brep_lib::BRepLibMakeWire;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;

/// `BRepBuilderAPI_MakeWire` (docs/topalgo-design.md section 10): each constructor and
/// Add runs the `BRepLib_MakeWire` member of the same Rust name and takes its wire when
/// it is done; `IsDone` and `Error` are those of the BRepLib maker.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_builder_api::{
///     BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire, BRepBuilderAPIWireError,
/// };
/// let edge = BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))?.edge()?;
/// let mut maker = BRepBuilderAPIMakeWire::new();
/// assert_eq!(maker.error(), BRepBuilderAPIWireError::EmptyWire);
/// maker.add_edge(&edge)?;
/// assert!(maker.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepBuilderAPIMakeWire {
    base: BRepBuilderAPIMakeShapeBase,
    my_make_wire: BRepLibMakeWire,
}

impl Default for BRepBuilderAPIMakeWire {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPIMakeWire {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `IsDone()`: the flag of the BRepLib maker; `maker.is_done()`.
    fn is_done(&self) -> bool {
        self.my_make_wire.is_done()
    }
}

impl BRepBuilderAPIMakeWire {
    /// `BRepBuilderAPI_MakeWire()`: an empty wire, `BRepBuilderAPI_EmptyWire`;
    /// `BRepBuilderAPIMakeWire::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_wire: BRepLibMakeWire::new(),
        }
    }

    /// Wraps a constructed BRepLib maker and takes its wire when it is done.
    fn wrap(my_make_wire: BRepLibMakeWire) -> Result<Self, Failure> {
        let mut maker = Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_wire,
        };
        maker.adopt()?;
        Ok(maker)
    }

    /// `if (myMakeWire.IsDone()) { Done(); myShape = myMakeWire.Wire(); }`.
    fn adopt(&mut self) -> Result<(), Failure> {
        if self.my_make_wire.is_done() {
            self.base.set_done();
            self.base.my_shape = self.my_make_wire.wire()?.into_shape();
        }
        Ok(())
    }

    wrapped_constructors!(BRepBuilderAPIMakeWire, BRepLibMakeWire;
        from_edge(e: &Edge) = "BRepBuilderAPI_MakeWire(E)";
        from_two_edges(e1: &Edge, e2: &Edge) = "BRepBuilderAPI_MakeWire(E1, E2)";
        from_three_edges(e1: &Edge, e2: &Edge, e3: &Edge) = "BRepBuilderAPI_MakeWire(E1, E2, E3)";
        from_four_edges(e1: &Edge, e2: &Edge, e3: &Edge, e4: &Edge) = "BRepBuilderAPI_MakeWire(E1, E2, E3, E4)";
        from_wire(w: &Wire) = "BRepBuilderAPI_MakeWire(W)";
        from_wire_edge(w: &Wire, e: &Edge) = "BRepBuilderAPI_MakeWire(W, E)";
    );

    wrapped_updates!(BRepBuilderAPIMakeWire, my_make_wire;
        add_edge(edge: &Edge) = "Add(const TopoDS_Edge& E)";
        add_wire(wire: &Wire) = "Add(const TopoDS_Wire& W)";
        add_list(edges: &List<Shape>) = "Add(const NCollection_List<TopoDS_Shape>& L)";
    );

    /// `Error()`: the status of the BRepLib maker, through the native switch;
    /// `maker.error()`.
    pub fn error(&self) -> BRepBuilderAPIWireError {
        BRepBuilderAPIWireError::from(self.my_make_wire.error())
    }

    /// `Wire()` and `operator TopoDS_Wire()`: the wire of the BRepLib maker;
    /// StdFail_NotDone when it is not done; `maker.wire()?`.
    pub fn wire(&self) -> Result<Wire, Failure> {
        self.my_make_wire.wire()
    }

    /// `Edge()`: the last edge added (a copy of it when its vertices were merged);
    /// `maker.edge()`.
    pub fn edge(&self) -> &Edge {
        self.my_make_wire.edge()
    }

    /// `Vertex()`: the last vertex connected; `maker.vertex()`.
    pub fn vertex(&self) -> &Vertex {
        self.my_make_wire.vertex()
    }
}
