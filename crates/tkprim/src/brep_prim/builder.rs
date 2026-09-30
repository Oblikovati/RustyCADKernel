// Rust translation of BRepPrim_Builder.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrim_Builder`: the topological services of the primitives, on the BRep builder.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::{Edge, Face, Shell, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Circle2, Circle3, Line2, Line3, Plane3, Point3};

/// `BRepPrim_Builder`: builds the shapes of the primitives with a `BRep_Builder`, at
/// Precision::Confusion() (docs/prim-design.md section 2).
///
/// ```
/// use rustycad_tkbrep::brep::BRepTool;
/// use rustycad_tkbrep::topods::Vertex;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tkprim::brep_prim::BRepPrimBuilder;
/// let mut vertex = Vertex::default();
/// BRepPrimBuilder::new().make_vertex(&mut vertex, Point3::new(1.0, 2.0, 3.0))?;
/// assert_eq!(BRepTool::vertex_tolerance(&vertex)?, 1e-7);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct BRepPrimBuilder {
    my_builder: BRepBuilder,
}

impl BRepPrimBuilder {
    /// `BRepPrim_Builder()`: a builder on a default `BRep_Builder`;
    /// `BRepPrimBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepPrim_Builder(B)`; `BRepPrimBuilder::with_builder(BRepBuilder::default())`.
    pub fn with_builder(builder: BRepBuilder) -> Self {
        Self {
            my_builder: builder,
        }
    }

    /// `Builder()`: the BRep builder; `builder.builder()`.
    pub fn builder(&self) -> &BRepBuilder {
        &self.my_builder
    }

    /// `MakeShell(S)`: an empty shell flagged closed; `builder.make_shell(&mut shell)?`.
    pub fn make_shell(&self, shell: &mut Shell) -> Result<(), Failure> {
        self.my_builder.make_shell(shell);
        shell.set_closed(true)
    }

    /// `MakeFace(F, P)`: a face on a `Geom_Plane` of `plane`;
    /// `builder.make_face(&mut face, plane)?`.
    pub fn make_face(&self, face: &mut Face, plane: Plane3) -> Result<(), Failure> {
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(plane)).cast();
        self.my_builder
            .make_face_on_surface(face, &surface, precision::confusion())
    }

    /// `MakeWire(W)`: an empty wire; `builder.make_wire(&mut wire)`.
    pub fn make_wire(&self, wire: &mut Wire) {
        self.my_builder.make_wire(wire);
    }

    /// `MakeDegeneratedEdge(E)`: an edge without geometry flagged degenerated;
    /// `builder.make_degenerated_edge(&mut edge)?`.
    pub fn make_degenerated_edge(&self, edge: &mut Edge) -> Result<(), Failure> {
        self.my_builder.make_edge(edge)?;
        self.my_builder.degenerated(edge, true)
    }

    /// `MakeEdge(E, L)`: an edge on a `Geom_Line` of `line`;
    /// `builder.make_edge_line(&mut edge, line)?`.
    pub fn make_edge_line(&self, edge: &mut Edge, line: Line3) -> Result<(), Failure> {
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(line)).cast();
        self.my_builder
            .make_edge_with_curve(edge, &curve, precision::confusion())
    }

    /// `MakeEdge(E, C)`: an edge on a `Geom_Circle` of `circle`;
    /// `builder.make_edge_circle(&mut edge, circle)?`.
    pub fn make_edge_circle(&self, edge: &mut Edge, circle: Circle3) -> Result<(), Failure> {
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(circle)).cast();
        self.my_builder
            .make_edge_with_curve(edge, &curve, precision::confusion())
    }

    /// `SetPCurve(E, F, L)`: the p-curve of `edge` on `face` is a `Geom2d_Line`;
    /// `builder.set_pcurve_line(&edge, &face, line)?`.
    pub fn set_pcurve_line(&self, edge: &Edge, face: &Face, line: Line2) -> Result<(), Failure> {
        let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(line)).cast();
        self.my_builder
            .update_edge_pcurve_on_face(edge, &pcurve, face, precision::confusion())
    }

    /// `SetPCurve(E, F, L1, L2)`: the seam p-curves of `edge` (taken FORWARD) on the
    /// closed surface of `face`, and a CN continuity of the edge between the face and
    /// itself; `builder.set_pcurve_lines(&edge, &face, first, last)?`.
    pub fn set_pcurve_lines(
        &self,
        edge: &Edge,
        face: &Face,
        first: Line2,
        last: Line2,
    ) -> Result<(), Failure> {
        let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
        let first: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(first)).cast();
        let last: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(last)).cast();
        self.my_builder.update_edge_pcurves_on_face(
            &forward,
            (&first, &last),
            face,
            precision::confusion(),
        )?;
        self.my_builder
            .continuity_on_faces(edge, face, face, Continuity::CN)
    }

    /// `SetPCurve(E, F, C)`: the p-curve of `edge` on `face` is a `Geom2d_Circle`;
    /// `builder.set_pcurve_circle(&edge, &face, circle)?`.
    pub fn set_pcurve_circle(
        &self,
        edge: &Edge,
        face: &Face,
        circle: Circle2,
    ) -> Result<(), Failure> {
        let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_circ2d(circle)).cast();
        self.my_builder
            .update_edge_pcurve_on_face(edge, &pcurve, face, precision::confusion())
    }

    /// `MakeVertex(V, P)`: a vertex at `point`; `builder.make_vertex(&mut vertex, point)?`.
    pub fn make_vertex(&self, vertex: &mut Vertex, point: Point3) -> Result<(), Failure> {
        self.my_builder
            .make_vertex_at(vertex, point, precision::confusion())
    }

    /// `ReverseFace(F)`: reverses the orientation of `face`; `builder.reverse_face(&mut face)`.
    pub fn reverse_face(&self, face: &mut Face) {
        face.reverse();
    }

    /// `AddEdgeVertex(E, V, P, direct)`: adds `vertex` (reversed unless `direct`) to
    /// `edge` at the parameter `parameter`;
    /// `builder.add_edge_vertex(&mut edge, &vertex, 0.0, true)?`.
    pub fn add_edge_vertex(
        &self,
        edge: &mut Edge,
        vertex: &Vertex,
        parameter: f64,
        direct: bool,
    ) -> Result<(), Failure> {
        let mut oriented = vertex.clone();
        if !direct {
            oriented.reverse();
        }
        self.my_builder.add(edge, &oriented)?;
        self.my_builder
            .update_vertex_on_edge(&oriented, parameter, edge, precision::confusion())
    }

    /// `AddEdgeVertex(E, V, P1, P2)`: adds `vertex` FORWARD and REVERSED to the closed
    /// `edge` and sets its range to `[first, last]`;
    /// `builder.add_edge_vertex_closed(&mut edge, &vertex, 0.0, 6.28)?`.
    pub fn add_edge_vertex_closed(
        &self,
        edge: &mut Edge,
        vertex: &Vertex,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        let mut oriented = vertex.clone();
        oriented.set_orientation(Orientation::Forward);
        self.my_builder.add(edge, &oriented)?;
        oriented.set_orientation(Orientation::Reversed);
        self.my_builder.add(edge, &oriented)?;
        self.my_builder.range(edge, first, last, false)
    }

    /// `SetParameters(E, V, P1, P2)`: sets the range of the closed `edge` (the vertex is
    /// not used, as natively); `builder.set_parameters(&edge, &vertex, 0.0, 6.28)?`.
    pub fn set_parameters(
        &self,
        edge: &Edge,
        _vertex: &Vertex,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        self.my_builder.range(edge, first, last, false)
    }

    /// `AddWireEdge(W, E, direct)`: adds `edge` (reversed unless `direct`) to `wire`;
    /// `builder.add_wire_edge(&mut wire, &edge, true)?`.
    pub fn add_wire_edge(&self, wire: &mut Wire, edge: &Edge, direct: bool) -> Result<(), Failure> {
        let mut oriented = edge.clone();
        if !direct {
            oriented.reverse();
        }
        self.my_builder.add(wire, &oriented)
    }

    /// `AddFaceWire(F, W)`; `builder.add_face_wire(&mut face, &wire)?`.
    pub fn add_face_wire(&self, face: &mut Face, wire: &Wire) -> Result<(), Failure> {
        self.my_builder.add(face, wire)
    }

    /// `AddShellFace(Sh, F)`; `builder.add_shell_face(&mut shell, &face)?`.
    pub fn add_shell_face(&self, shell: &mut Shell, face: &Face) -> Result<(), Failure> {
        self.my_builder.add(shell, face)
    }

    /// `CompleteEdge(E)`: `BRepTools::Update(E)`, which does nothing;
    /// `builder.complete_edge(&edge)`.
    pub fn complete_edge(&self, edge: &Edge) {
        brep_tools::update_edge(edge);
    }

    /// `CompleteWire(W)`: flags the wire closed when it is, then `BRepTools::Update(W)`;
    /// `builder.complete_wire(&wire)?`.
    pub fn complete_wire(&self, wire: &Wire) -> Result<(), Failure> {
        wire.set_closed(BRepTool::is_closed(wire)?)?;
        brep_tools::update_wire(wire);
        Ok(())
    }

    /// `CompleteFace(F)`: `BRepTools::Update(F)`; `builder.complete_face(&face)?`.
    pub fn complete_face(&self, face: &Face) -> Result<(), Failure> {
        brep_tools::update_face(face)
    }

    /// `CompleteShell(S)`: flags the shell closed when it is, then
    /// `BRepTools::Update(S)`; `builder.complete_shell(&shell)?`.
    pub fn complete_shell(&self, shell: &Shell) -> Result<(), Failure> {
        shell.set_closed(BRepTool::is_closed(shell)?)?;
        brep_tools::update_shell(shell)
    }
}
