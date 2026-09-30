// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_GLine.hxx/.lxx and the
// constructors, Replace and AddVertex of IntPatch_GLine.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_GLine`: an intersection line that is a conic.
use super::IntPatchIType;
use super::failures::domain_error;
use super::gline_compute::compute_vertex_parameters;
use super::line::{IntPatchLine, IntPatchLineBase, IntPatchLineTransitions};
use super::point::IntPatchPoint;
use super::vertices::VertexList;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Axis2, Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Plane3};
use std::f64::consts::PI;

/// The fields of `IntPatch_GLine`.
pub(super) struct GLineState {
    pos: Axis2,
    par1: f64,
    par2: f64,
    pub(super) vertices: VertexList,
}

/// `IntPatch_GLine`: a line, circle, ellipse, parabola or hyperbola of intersection
/// (its frame and radii), with its vertices.
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::{IntPatchGLine, IntPatchIType, IntPatchLine, IntPatchLineTransitions};
/// use rustycad_tkmath::gp::{Direction3, Line3, Point3};
/// let axis = Line3::new(Point3::new(0.0, 0.0, 0.0), Direction3::new(0.0, 0.0, 1.0).unwrap());
/// let line = IntPatchGLine::from_line(&axis, false, IntPatchLineTransitions::Undecided).unwrap();
/// assert_eq!(line.arc_type(), IntPatchIType::Lin);
/// assert!(line.circle().is_err());
/// ```
pub struct IntPatchGLine {
    transient: StandardTransient,
    base: IntPatchLineBase,
    state: SharedState<GLineState>,
}
impl_transient!(IntPatchGLine, "IntPatch_GLine", dyn IntPatchLine, transient,
    interfaces: [dyn IntPatchLine]);

pub(super) const GLINE: &str = "IntPatch_GLine";

impl IntPatchGLine {
    fn conic(
        (pos, par1, par2): (Axis2, f64, f64),
        tang: bool,
        transitions: IntPatchLineTransitions,
        typ: IntPatchIType,
    ) -> Self {
        let state = GLineState {
            pos,
            par1,
            par2,
            vertices: VertexList::default(),
        };
        Self {
            transient: StandardTransient::default(),
            base: IntPatchLineBase::new(tang, transitions, typ),
            state: SharedState::new(state),
        }
    }

    /// `IntPatch_GLine(L, Tang, ...)`: a straight line; its frame is the one of the plane
    /// through the line location normal to it (`gp_Pln(L.Location(), L.Direction())`).
    /// `IntPatchGLine::from_line(&l, false, IntPatchLineTransitions::Undecided)?`.
    pub fn from_line(
        l: &Line3,
        tang: bool,
        transitions: IntPatchLineTransitions,
    ) -> Result<Self, Failure> {
        let pos: Axis2 = Plane3::new(l.location(), l.direction())?.position().ax2()?;
        Ok(Self::conic(
            (pos, 0.0, 0.0),
            tang,
            transitions,
            IntPatchIType::Lin,
        ))
    }

    /// `IntPatch_GLine(C, Tang, ...)`: a circle.
    /// `IntPatchGLine::from_circle(&c, false, IntPatchLineTransitions::Undecided)`.
    pub fn from_circle(c: &Circle3, tang: bool, transitions: IntPatchLineTransitions) -> Self {
        let geometry = (c.position(), c.radius(), 0.0);
        Self::conic(geometry, tang, transitions, IntPatchIType::Circle)
    }

    /// `IntPatch_GLine(E, Tang, ...)`: an ellipse.
    /// `IntPatchGLine::from_ellipse(&e, false, IntPatchLineTransitions::Undecided)`.
    pub fn from_ellipse(e: &Ellipse3, tang: bool, transitions: IntPatchLineTransitions) -> Self {
        let geometry = (e.position(), e.major_radius(), e.minor_radius());
        Self::conic(geometry, tang, transitions, IntPatchIType::Ellipse)
    }

    /// `IntPatch_GLine(P, Tang, ...)`: a parabola.
    /// `IntPatchGLine::from_parabola(&p, false, IntPatchLineTransitions::Undecided)`.
    pub fn from_parabola(p: &Parabola3, tang: bool, transitions: IntPatchLineTransitions) -> Self {
        let geometry = (p.position(), p.focal(), 0.0);
        Self::conic(geometry, tang, transitions, IntPatchIType::Parabola)
    }

    /// `IntPatch_GLine(H, Tang, ...)`: a hyperbola.
    /// `IntPatchGLine::from_hyperbola(&h, false, IntPatchLineTransitions::Undecided)`.
    pub fn from_hyperbola(
        h: &Hyperbola3,
        tang: bool,
        transitions: IntPatchLineTransitions,
    ) -> Self {
        let geometry = (h.position(), h.major_radius(), h.minor_radius());
        Self::conic(geometry, tang, transitions, IntPatchIType::Hyperbola)
    }

    /// `AddVertex(Pnt)`: on a circle or an ellipse with a first and a last point, the
    /// parameter is brought into `[first, last]` by turns of 2 Pi (a point still before
    /// the first one by more than 1e-6 is not added); on the other conics with both
    /// points, a point outside them is not added. `line.add_vertex(&vertex)?`.
    pub fn add_vertex(&self, pnt: &IntPatchPoint) -> Result<(), Failure> {
        let typ: IntPatchIType = self.arc_type();
        let mut state = self.state.write();
        let v: &mut VertexList = &mut state.vertices;
        if v.nb_vertex() == 0 {
            v.svtx.push(pnt.clone());
            return Ok(());
        }
        let member: String = format!("{GLINE}::AddVertex");
        let pf: f64 = if v.fipt {
            v.param(v.indf, &member)?
        } else {
            0.0
        };
        let pl: f64 = if v.lapt {
            v.param(v.indl, &member)?
        } else {
            0.0
        };
        if !(v.fipt && v.lapt) {
            v.svtx.push(pnt.clone());
            return Ok(());
        }
        let mut par: f64 = pnt.parameter_on_line();
        if typ != IntPatchIType::Circle && typ != IntPatchIType::Ellipse {
            if !(pl < par || par < pf) {
                v.svtx.push(pnt.clone());
            }
            return Ok(());
        }
        while par < pf {
            par += PI + PI;
        }
        while par > pl {
            par -= PI + PI;
        }
        if par < pf && (pf - par) > precision::p_confusion() * 1000.0 {
            return Ok(());
        }
        let mut modified: IntPatchPoint = pnt.clone();
        modified.set_parameter(par);
        v.svtx.push(modified);
        Ok(())
    }

    /// `Replace(Index, Pnt)`: quirk kept, the native body appends `pnt` and ignores the
    /// index (the assignment is commented out). `line.replace(1, &vertex)`.
    pub fn replace(&self, _index: i32, pnt: &IntPatchPoint) {
        self.state.write().vertices.svtx.push(pnt.clone());
    }

    /// `SetFirstPoint(IndFirst)`. `line.set_first_point(1)`.
    pub fn set_first_point(&self, ind_first: i32) {
        self.state.write().vertices.set_first_point(ind_first);
    }

    /// `SetLastPoint(IndLast)`. `line.set_last_point(2)`.
    pub fn set_last_point(&self, ind_last: i32) {
        self.state.write().vertices.set_last_point(ind_last);
    }

    fn check_type(&self, expected: IntPatchIType, member: &str) -> Result<GeometryFields, Failure> {
        let typ: IntPatchIType = self.arc_type();
        if typ != expected {
            let reason = format!("the line is a {typ:?}; expected a {expected:?}");
            return Err(domain_error(&format!("{GLINE}::{member}"), &reason));
        }
        let state = self.state.read();
        Ok((state.pos, state.par1, state.par2))
    }

    /// `Line()`: Standard_DomainError unless the line is straight. `line.line()?`.
    pub fn line(&self) -> Result<Line3, Failure> {
        let (pos, _, _) = self.check_type(IntPatchIType::Lin, "Line")?;
        Ok(Line3::from_axis(pos.axis()))
    }

    /// `Circle()`: Standard_DomainError unless the line is a circle. `line.circle()?`.
    pub fn circle(&self) -> Result<Circle3, Failure> {
        let (pos, par1, _) = self.check_type(IntPatchIType::Circle, "Circle")?;
        Circle3::new(pos, par1)
    }

    /// `Ellipse()`: Standard_DomainError unless the line is an ellipse. `line.ellipse()?`.
    pub fn ellipse(&self) -> Result<Ellipse3, Failure> {
        let (pos, par1, par2) = self.check_type(IntPatchIType::Ellipse, "Ellipse")?;
        Ellipse3::new(pos, par1, par2)
    }

    /// `Parabola()`: Standard_DomainError unless the line is a parabola. `line.parabola()?`.
    pub fn parabola(&self) -> Result<Parabola3, Failure> {
        let (pos, par1, _) = self.check_type(IntPatchIType::Parabola, "Parabola")?;
        Parabola3::new(pos, par1)
    }

    /// `Hyperbola()`: Standard_DomainError unless the line is a hyperbola.
    /// `line.hyperbola()?`.
    pub fn hyperbola(&self) -> Result<Hyperbola3, Failure> {
        let (pos, par1, par2) = self.check_type(IntPatchIType::Hyperbola, "Hyperbola")?;
        Hyperbola3::new(pos, par1, par2)
    }

    /// `HasFirstPoint()`. `line.has_first_point()`.
    pub fn has_first_point(&self) -> bool {
        self.state.read().vertices.fipt
    }

    /// `HasLastPoint()`. `line.has_last_point()`.
    pub fn has_last_point(&self) -> bool {
        self.state.read().vertices.lapt
    }

    /// `FirstPoint()`: Standard_DomainError without a first point. `line.first_point()?`.
    pub fn first_point(&self) -> Result<IntPatchPoint, Failure> {
        self.state.read().vertices.first_point(GLINE).cloned()
    }

    /// `LastPoint()`: Standard_DomainError without a last point. `line.last_point()?`.
    pub fn last_point(&self) -> Result<IntPatchPoint, Failure> {
        self.state.read().vertices.last_point(GLINE).cloned()
    }

    /// `NbVertex()`. `line.nb_vertex()`.
    pub fn nb_vertex(&self) -> i32 {
        self.state.read().vertices.nb_vertex()
    }

    /// `Vertex(Index)`: Standard_OutOfRange outside `1..=NbVertex()`. `line.vertex(1)?`.
    pub fn vertex(&self, index: i32) -> Result<IntPatchPoint, Failure> {
        let member: String = format!("{GLINE}::Vertex");
        self.state.read().vertices.value(index, &member).cloned()
    }

    /// `ComputeVertexParameters(Tol)`: filters the duplicated vertices, sorts them, moves
    /// the seam vertices of a circle or an ellipse by 2 Pi, and makes the first and last
    /// vertices the bounds (the tolerance is not used). `line.compute_vertex_parameters(1e-7)?`.
    pub fn compute_vertex_parameters(&self, _tol: f64) -> Result<(), Failure> {
        let typ: IntPatchIType = self.arc_type();
        compute_vertex_parameters(&mut self.state.write().vertices, typ)
    }
}

/// The frame and the two radii of the conic.
type GeometryFields = (Axis2, f64, f64);

impl IntPatchLine for IntPatchGLine {
    fn line_base(&self) -> &IntPatchLineBase {
        &self.base
    }
}
