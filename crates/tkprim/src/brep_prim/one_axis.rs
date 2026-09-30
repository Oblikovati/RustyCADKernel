// Rust translation of BRepPrim_OneAxis.hxx/.cxx (the data, the accessors and the
// virtual members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrim_OneAxis`: the primitives with one axis of revolution, as a trait over a
//! base struct (docs/prim-design.md section 2). The topology builders are in
//! `one_axis_faces.rs`, `one_axis_wires.rs`, `one_axis_edges.rs` and
//! `one_axis_vertices.rs`.
use super::BRepPrimBuilder;
use super::{one_axis_edges as edges, one_axis_faces as faces};
use super::{one_axis_vertices as vertices, one_axis_wires as wires};
use rustycad_tkbrep::topods::{Edge, Face, Shell, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2, Point2, Point3, Vector3};
use std::f64::consts::PI;

/// The vertex slots (the native `VertexIndex`).
pub(crate) const VAXISTOP: usize = 0;
pub(crate) const VAXISBOT: usize = 1;
pub(crate) const VTOPSTART: usize = 2;
pub(crate) const VTOPEND: usize = 3;
pub(crate) const VBOTSTART: usize = 4;
pub(crate) const VBOTEND: usize = 5;
/// The edge slots (the native `EdgeIndex`).
pub(crate) const EAXIS: usize = 0;
pub(crate) const ESTART: usize = 1;
pub(crate) const EEND: usize = 2;
pub(crate) const ETOPSTART: usize = 3;
pub(crate) const ETOPEND: usize = 4;
pub(crate) const EBOTSTART: usize = 5;
pub(crate) const EBOTEND: usize = 6;
pub(crate) const ETOP: usize = 7;
pub(crate) const EBOTTOM: usize = 8;
/// The wire slots (the native `WireIndex`; slot 4 is unused, as natively).
pub(crate) const WLATERAL: usize = 0;
pub(crate) const WLATERALSTART: usize = WLATERAL;
pub(crate) const WLATERALEND: usize = 1;
pub(crate) const WTOP: usize = 2;
pub(crate) const WBOTTOM: usize = 3;
pub(crate) const WSTART: usize = 5;
pub(crate) const WAXISSTART: usize = 6;
pub(crate) const WEND: usize = 7;
pub(crate) const WAXISEND: usize = 8;
/// The face slots (the native `FaceIndex`).
pub(crate) const FLATERAL: usize = 0;
pub(crate) const FTOP: usize = 1;
pub(crate) const FBOTTOM: usize = 2;
pub(crate) const FSTART: usize = 3;
pub(crate) const FEND: usize = 4;

/// The protected and private data of `BRepPrim_OneAxis`: the builder, the frame, the
/// angle, the meridian range and offset, and the sub-shapes with their built flags.
///
/// ```
/// use rustycad_tkmath::gp::Axis2;
/// use rustycad_tkprim::brep_prim::{BRepPrimBuilder, BRepPrimOneAxisBase};
/// let base = BRepPrimOneAxisBase::new(BRepPrimBuilder::new(), Axis2::default(), 0.0, 1.0);
/// assert_eq!(base.angle(), 2.0 * std::f64::consts::PI);
/// ```
#[derive(Clone, Debug)]
pub struct BRepPrimOneAxisBase {
    /// `myBuilder` (protected): the builder the derived classes make their shapes with.
    pub my_builder: BRepPrimBuilder,
    pub(crate) my_axes: Axis2,
    pub(crate) my_angle: f64,
    pub(crate) my_vmin: f64,
    pub(crate) my_vmax: f64,
    pub(crate) my_meridian_offset: f64,
    pub(crate) my_shell: Shell,
    pub(crate) shell_built: bool,
    pub(crate) my_vertices: [Vertex; 6],
    pub(crate) vertices_built: [bool; 6],
    pub(crate) my_edges: [Edge; 9],
    pub(crate) edges_built: [bool; 9],
    pub(crate) my_wires: [Wire; 9],
    pub(crate) wires_built: [bool; 9],
    pub(crate) my_faces: [Face; 5],
    pub(crate) faces_built: [bool; 5],
}

impl BRepPrimOneAxisBase {
    /// `BRepPrim_OneAxis(B, A, VMin, VMax)` (protected): the angle is 2*PI, the offset
    /// 0 and nothing is built; `BRepPrimOneAxisBase::new(builder, axes, 0.0, 1.0)`.
    pub fn new(builder: BRepPrimBuilder, axes: Axis2, vmin: f64, vmax: f64) -> Self {
        Self {
            my_builder: builder,
            my_axes: axes,
            my_angle: 2.0 * PI,
            my_vmin: vmin,
            my_vmax: vmax,
            my_meridian_offset: 0.0,
            my_shell: Shell::default(),
            shell_built: false,
            my_vertices: Default::default(),
            vertices_built: [false; 6],
            my_edges: Default::default(),
            edges_built: [false; 9],
            my_wires: Default::default(),
            wires_built: [false; 9],
            my_faces: Default::default(),
            faces_built: [false; 5],
        }
    }

    /// `Axes()`; `base.axes()`.
    pub fn axes(&self) -> Axis2 {
        self.my_axes
    }

    /// `Angle()`; `base.angle()`.
    pub fn angle(&self) -> f64 {
        self.my_angle
    }

    /// `VMin()`; `base.vmin()`.
    pub fn vmin(&self) -> f64 {
        self.my_vmin
    }

    /// `VMax()`; `base.vmax()`.
    pub fn vmax(&self) -> f64 {
        self.my_vmax
    }

    /// `BRepPrim_OneAxis_Check`: Standard_DomainError once a vertex, an edge, a wire or
    /// a face is built (the shell flag is not checked, as natively).
    fn check_nothing_built(&self, member: &str) -> Result<(), Failure> {
        let built = self.vertices_built.iter().any(|&b| b)
            || self.edges_built.iter().any(|&b| b)
            || self.wires_built.iter().any(|&b| b)
            || self.faces_built.iter().any(|&b| b);
        if built {
            return Err(one_axis_domain_error(
                member,
                "sub-shapes are already built; expected an unbuilt primitive",
            ));
        }
        Ok(())
    }
}

/// The Standard_DomainError of a `BRepPrim_OneAxis` member.
pub(crate) fn one_axis_domain_error(member: &str, detail: &str) -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        &format!("BRepPrim_OneAxis::{member}: {detail}"),
    )
}

/// `BRepPrim_OneAxis` (docs/prim-design.md section 2): a body of revolution around the Z
/// axis of `Axes`, swept by a meridian over `VMin..VMax` and the angle `0..Angle`, with
/// at most five faces (lateral, top, bottom, start and end). Building an element builds
/// its sub-elements, each once. Implementors provide the base accessors and the four
/// pure virtual members, may override the virtual members with a native body, and never
/// override the topology builders (non-virtual natively).
pub trait BRepPrimOneAxis {
    /// The embedded base data.
    fn one_axis_base(&self) -> &BRepPrimOneAxisBase;

    /// The embedded base data, to write.
    fn one_axis_base_mut(&mut self) -> &mut BRepPrimOneAxisBase;

    /// `MakeEmptyLateralFace()`: a face without edges on the lateral surface, its normal
    /// outward; `prim.make_empty_lateral_face()?`.
    fn make_empty_lateral_face(&self) -> Result<Face, Failure>;

    /// `MakeEmptyMeridianEdge(Ang)`: an edge on the meridian rotated by `angle` (0 or
    /// `Angle()`) around the axis; `prim.make_empty_meridian_edge(0.0)?`.
    fn make_empty_meridian_edge(&self, angle: f64) -> Result<Edge, Failure>;

    /// `SetMeridianPCurve(E, F)`: sets the p-curve of `edge` on `face` to the 2d meridian;
    /// `prim.set_meridian_pcurve(&edge, &face)?`.
    fn set_meridian_pcurve(&self, edge: &Edge, face: &Face) -> Result<(), Failure>;

    /// `MeridianValue(V)`: the meridian point of parameter `v` in the XZ plane;
    /// `prim.meridian_value(0.0)?`.
    fn meridian_value(&self, v: f64) -> Result<Point2, Failure>;

    /// `MeridianOnAxis(V)`: whether the meridian point of parameter `v` is on the axis
    /// (|X| < Precision::Confusion()); `prim.meridian_on_axis(0.0)?`.
    fn meridian_on_axis(&self, v: f64) -> Result<bool, Failure> {
        Ok(self.meridian_value(v)?.x().abs() < precision::confusion())
    }

    /// `MeridianClosed()`: a finite meridian whose ends coincide within
    /// Precision::Confusion(); `prim.meridian_closed()?`.
    fn meridian_closed(&self) -> Result<bool, Failure> {
        if self.vmax_infinite() || self.vmin_infinite() {
            return Ok(false);
        }
        let base = self.one_axis_base();
        let first = self.meridian_value(base.my_vmin)?;
        Ok(first.is_equal(self.meridian_value(base.my_vmax)?, precision::confusion()))
    }

    /// `VMaxInfinite()`: Precision::IsPositiveInfinite(VMax); `prim.vmax_infinite()`.
    fn vmax_infinite(&self) -> bool {
        precision::is_positive_infinite(self.one_axis_base().my_vmax)
    }

    /// `VMinInfinite()`: Precision::IsNegativeInfinite(VMin); `prim.vmin_infinite()`.
    fn vmin_infinite(&self) -> bool {
        precision::is_negative_infinite(self.one_axis_base().my_vmin)
    }

    /// `HasTop()`: neither VMaxInfinite, MeridianClosed nor MeridianOnAxis(VMax);
    /// `prim.has_top()?`.
    fn has_top(&self) -> Result<bool, Failure> {
        if self.vmax_infinite() || self.meridian_closed()? {
            return Ok(false);
        }
        Ok(!self.meridian_on_axis(self.one_axis_base().my_vmax)?)
    }

    /// `HasBottom()`: neither VMinInfinite, MeridianClosed nor MeridianOnAxis(VMin);
    /// `prim.has_bottom()?`.
    fn has_bottom(&self) -> Result<bool, Failure> {
        if self.vmin_infinite() || self.meridian_closed()? {
            return Ok(false);
        }
        Ok(!self.meridian_on_axis(self.one_axis_base().my_vmin)?)
    }

    /// `HasSides()`: 2*PI - Angle > Precision::Angular(); `prim.has_sides()`.
    fn has_sides(&self) -> bool {
        2.0 * PI - self.one_axis_base().my_angle > precision::angular()
    }

    /// `SetMeridianOffset(MeridianOffset)`: the offset added to the meridian parameters
    /// of the meridian edges and to the V of the p-curves;
    /// `prim.set_meridian_offset(2.0 * std::f64::consts::PI)`.
    fn set_meridian_offset(&mut self, offset: f64) {
        self.one_axis_base_mut().my_meridian_offset = offset;
    }

    /// `Axes()`; `prim.axes()`.
    fn axes(&self) -> Axis2 {
        self.one_axis_base().my_axes
    }

    /// `Axes(A)`: Standard_DomainError once something is built; `prim.set_axes(axes)?`.
    fn set_axes(&mut self, axes: Axis2) -> Result<(), Failure> {
        let base = self.one_axis_base_mut();
        base.check_nothing_built("Axes")?;
        base.my_axes = axes;
        Ok(())
    }

    /// `Angle()`; `prim.angle()`.
    fn angle(&self) -> f64 {
        self.one_axis_base().my_angle
    }

    /// `Angle(A)`: Standard_DomainError once something is built; `prim.set_angle(1.0)?`.
    fn set_angle(&mut self, angle: f64) -> Result<(), Failure> {
        let base = self.one_axis_base_mut();
        base.check_nothing_built("Angle")?;
        base.my_angle = angle;
        Ok(())
    }

    /// `VMin()`; `prim.vmin()`.
    fn vmin(&self) -> f64 {
        self.one_axis_base().my_vmin
    }

    /// `VMin(V)`: Standard_DomainError once something is built; `prim.set_vmin(0.0)?`.
    fn set_vmin(&mut self, v: f64) -> Result<(), Failure> {
        let base = self.one_axis_base_mut();
        base.check_nothing_built("VMin")?;
        base.my_vmin = v;
        Ok(())
    }

    /// `VMax()`; `prim.vmax()`.
    fn vmax(&self) -> f64 {
        self.one_axis_base().my_vmax
    }

    /// `VMax(V)`: Standard_DomainError once something is built; `prim.set_vmax(1.0)?`.
    fn set_vmax(&mut self, v: f64) -> Result<(), Failure> {
        let base = self.one_axis_base_mut();
        base.check_nothing_built("VMax")?;
        base.my_vmax = v;
        Ok(())
    }

    /// `Shell()`: the shell of every existing face; `prim.shell()?`.
    fn shell(&mut self) -> Result<Shell, Failure> {
        faces::shell(self)
    }

    /// `LateralFace()`: the lateral face, oriented outward; `prim.lateral_face()?`.
    fn lateral_face(&mut self) -> Result<Face, Failure> {
        faces::lateral_face(self)
    }

    /// `TopFace()`: the top planar face, oriented toward +Z; Standard_DomainError
    /// without a top; `prim.top_face()?`.
    fn top_face(&mut self) -> Result<Face, Failure> {
        faces::top_face(self)
    }

    /// `BottomFace()`: the bottom planar face, oriented toward -Z; Standard_DomainError
    /// without a bottom; `prim.bottom_face()?`.
    fn bottom_face(&mut self) -> Result<Face, Failure> {
        faces::bottom_face(self)
    }

    /// `StartFace()`: the face at angle 0, oriented outward; Standard_DomainError without
    /// sides; `prim.start_face()?`.
    fn start_face(&mut self) -> Result<Face, Failure> {
        faces::start_face(self)
    }

    /// `EndFace()`: the face at `Angle()`, oriented outward; Standard_DomainError without
    /// sides; `prim.end_face()?`.
    fn end_face(&mut self) -> Result<Face, Failure> {
        faces::end_face(self)
    }

    /// `LateralWire()`: the wire of the lateral face; `prim.lateral_wire()?`.
    fn lateral_wire(&mut self) -> Result<Wire, Failure> {
        wires::lateral_wire(self)
    }

    /// `LateralStartWire()`: the lateral wire of the start edge alone;
    /// `prim.lateral_start_wire()?`.
    fn lateral_start_wire(&mut self) -> Result<Wire, Failure> {
        wires::lateral_start_wire(self)
    }

    /// `LateralEndWire()`: the lateral wire of the end edge alone;
    /// `prim.lateral_end_wire()?`.
    fn lateral_end_wire(&mut self) -> Result<Wire, Failure> {
        wires::lateral_end_wire(self)
    }

    /// `TopWire()`: the wire of the top face; `prim.top_wire()?`.
    fn top_wire(&mut self) -> Result<Wire, Failure> {
        wires::top_wire(self)
    }

    /// `BottomWire()`: the wire of the bottom face; `prim.bottom_wire()?`.
    fn bottom_wire(&mut self) -> Result<Wire, Failure> {
        wires::bottom_wire(self)
    }

    /// `StartWire()`: the wire of the start face; `prim.start_wire()?`.
    fn start_wire(&mut self) -> Result<Wire, Failure> {
        wires::start_wire(self)
    }

    /// `AxisStartWire()`: the wire of the start face with the axis edge alone;
    /// `prim.axis_start_wire()?`.
    fn axis_start_wire(&mut self) -> Result<Wire, Failure> {
        wires::axis_start_wire(self)
    }

    /// `EndWire()`: the wire of the end face; `prim.end_wire()?`.
    fn end_wire(&mut self) -> Result<Wire, Failure> {
        wires::end_wire(self)
    }

    /// `AxisEndWire()`: the wire of the end face with the axis edge alone;
    /// `prim.axis_end_wire()?`.
    fn axis_end_wire(&mut self) -> Result<Wire, Failure> {
        wires::axis_end_wire(self)
    }

    /// `AxisEdge()`: the edge on the axis, oriented +Z; `prim.axis_edge()?`.
    fn axis_edge(&mut self) -> Result<Edge, Failure> {
        edges::axis_edge(self)
    }

    /// `StartEdge()`: the meridian edge at angle 0; `prim.start_edge()?`.
    fn start_edge(&mut self) -> Result<Edge, Failure> {
        edges::start_edge(self)
    }

    /// `EndEdge()`: the meridian edge at `Angle()` (the start edge without sides);
    /// `prim.end_edge()?`.
    fn end_edge(&mut self) -> Result<Edge, Failure> {
        edges::end_edge(self)
    }

    /// `StartTopEdge()`: the line between the start and top faces; `prim.start_top_edge()?`.
    fn start_top_edge(&mut self) -> Result<Edge, Failure> {
        edges::start_top_edge(self)
    }

    /// `StartBottomEdge()`: the line between the start and bottom faces;
    /// `prim.start_bottom_edge()?`.
    fn start_bottom_edge(&mut self) -> Result<Edge, Failure> {
        edges::start_bottom_edge(self)
    }

    /// `EndTopEdge()`: the line between the end and top faces; `prim.end_top_edge()?`.
    fn end_top_edge(&mut self) -> Result<Edge, Failure> {
        edges::end_top_edge(self)
    }

    /// `EndBottomEdge()`: the line between the end and bottom faces;
    /// `prim.end_bottom_edge()?`.
    fn end_bottom_edge(&mut self) -> Result<Edge, Failure> {
        edges::end_bottom_edge(self)
    }

    /// `TopEdge()`: the circle at VMax (the bottom edge of a closed meridian);
    /// `prim.top_edge()?`.
    fn top_edge(&mut self) -> Result<Edge, Failure> {
        edges::top_edge(self)
    }

    /// `BottomEdge()`: the circle at VMin (the top edge of a closed meridian);
    /// `prim.bottom_edge()?`.
    fn bottom_edge(&mut self) -> Result<Edge, Failure> {
        edges::bottom_edge(self)
    }

    /// `AxisTopVertex()`: the vertex on the axis at the top height;
    /// `prim.axis_top_vertex()?`.
    fn axis_top_vertex(&mut self) -> Result<Vertex, Failure> {
        vertices::axis_top_vertex(self)
    }

    /// `AxisBottomVertex()`: the vertex on the axis at the bottom height;
    /// `prim.axis_bottom_vertex()?`.
    fn axis_bottom_vertex(&mut self) -> Result<Vertex, Failure> {
        vertices::axis_bottom_vertex(self)
    }

    /// `TopStartVertex()`: the vertex (0, VMax); `prim.top_start_vertex()?`.
    fn top_start_vertex(&mut self) -> Result<Vertex, Failure> {
        vertices::top_start_vertex(self)
    }

    /// `TopEndVertex()`: the vertex (Angle, VMax); `prim.top_end_vertex()?`.
    fn top_end_vertex(&mut self) -> Result<Vertex, Failure> {
        vertices::top_end_vertex(self)
    }

    /// `BottomStartVertex()`: the vertex (0, VMin); `prim.bottom_start_vertex()?`.
    fn bottom_start_vertex(&mut self) -> Result<Vertex, Failure> {
        vertices::bottom_start_vertex(self)
    }

    /// `BottomEndVertex()`: the vertex (Angle, VMin); `prim.bottom_end_vertex()?`.
    fn bottom_end_vertex(&mut self) -> Result<Vertex, Failure> {
        vertices::bottom_end_vertex(self)
    }
}

/// `isHeightInverted()`: whether the meridian height at VMax is below the height at
/// VMin (beyond Precision::Confusion()), as for a torus over (90, 270) degrees; false
/// for an infinite range.
pub(crate) fn is_height_inverted<T: BRepPrimOneAxis + ?Sized>(prim: &T) -> Result<bool, Failure> {
    if prim.vmax_infinite() || prim.vmin_infinite() {
        return Ok(false);
    }
    let base = prim.one_axis_base();
    let y_max = prim.meridian_value(base.my_vmax)?.y();
    let y_min = prim.meridian_value(base.my_vmin)?.y();
    Ok(y_max < y_min - precision::confusion())
}

/// `areHeightsEqual()`: whether the meridian heights at VMin and VMax are equal within
/// Precision::Confusion(), which makes the axis edge degenerated; false for an infinite
/// range.
pub(crate) fn are_heights_equal<T: BRepPrimOneAxis + ?Sized>(prim: &T) -> Result<bool, Failure> {
    if prim.vmax_infinite() || prim.vmin_infinite() {
        return Ok(false);
    }
    let base = prim.one_axis_base();
    let y_max = prim.meridian_value(base.my_vmax)?.y();
    let y_min = prim.meridian_value(base.my_vmin)?.y();
    Ok((y_max - y_min).abs() < precision::confusion())
}

/// The axis direction multiplied by `height` (`V = Axes.Direction(); V.Multiply(z)`).
pub(crate) fn axis_vector(axes: Axis2, height: f64) -> Vector3 {
    let mut v = Vector3::from(axes.direction());
    v.scale(height);
    v
}

/// The point of the axis at `height`: the axes location translated by
/// [`axis_vector`].
pub(crate) fn axis_point(axes: Axis2, height: f64) -> Point3 {
    axes.location().translated(axis_vector(axes, height))
}
