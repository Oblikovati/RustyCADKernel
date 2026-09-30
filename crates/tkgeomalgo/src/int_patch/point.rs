// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Point.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_Point`: an intersection point between two surfaces, with its parameters,
//! its restriction arcs and vertices and their transitions.
use super::failures::domain_error;
use crate::int_start::{TheArc, TheVertex};
use crate::int_surf::{IntSurfPntOn2S, IntSurfTransition, IntSurfTypeTrans};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// The domain data of the point on one surface: the arc, the vertex and the
/// transitions (`onS1`, `vtxonS1`, `vS1`, `arcS1`, `traline1`, `tra1`, `prm1`).
#[derive(Clone, Default)]
pub(super) struct PointOnDomain {
    pub(super) on: bool,
    pub(super) vtx_on: bool,
    pub(super) vertex: TheVertex,
    pub(super) arc: TheArc,
    pub(super) traline: IntSurfTransition,
    pub(super) tra: IntSurfTransition,
    pub(super) prm: f64,
}

/// `IntPatch_Point`: the geometric point with its parameters on both surfaces and on the
/// line, and the logical information (tangency, multiplicity, arcs and vertices of each
/// domain).
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchPoint;
/// use rustycad_tkmath::gp::Point3;
/// let mut p = IntPatchPoint::new();
/// p.set_value_tangent(Point3::new(1.0, 2.0, 3.0), 1e-7, true);
/// assert!(p.is_tangency_point() && !p.is_on_dom_s1());
/// assert!(p.arc_on_s1().is_err());
/// ```
#[derive(Clone, Default)]
pub struct IntPatchPoint {
    pt: IntSurfPntOn2S,
    para: f64,
    tol: f64,
    tgt: bool,
    mult: bool,
    pub(super) s1: PointOnDomain,
    pub(super) s2: PointOnDomain,
}

/// The Standard_DomainError of a domain accessor on a point that is not on that domain.
fn not_on(member: &str, what: &str) -> Failure {
    domain_error(
        &format!("IntPatch_Point::{member}"),
        &format!("the point is not {what}"),
    )
}

impl IntPatchPoint {
    /// The empty constructor: every flag false, every value 0.
    /// `IntPatchPoint::new().tolerance() == 0.0`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `SetValue(Pt, Tol, Tangent)`: a point on no domain; resets the domain and
    /// multiplicity flags (the arcs, vertices and transitions are kept, as natively).
    /// `p.set_value_tangent(point, 1e-7, false)`.
    pub fn set_value_tangent(&mut self, pt: Point3, tol: f64, tangent: bool) {
        (self.s1.on, self.s2.on, self.s1.vtx_on, self.s2.vtx_on) = (false, false, false, false);
        self.mult = false;
        self.tgt = tangent;
        self.pt.set_point(pt);
        self.tol = tol;
    }

    /// `SetValue(Pt)`: the 3d point only. `p.set_value(point)`.
    pub fn set_value(&mut self, pt: Point3) {
        self.pt.set_point(pt);
    }

    /// `SetValue(thePOn2S)`: the point and its parameters. `p.set_pnt_on_2s(&pon2s)`.
    pub fn set_pnt_on_2s(&mut self, pon2s: &IntSurfPntOn2S) {
        self.pt = *pon2s;
    }

    /// `SetTolerance(Tol)`. `p.set_tolerance(1e-6)`.
    pub fn set_tolerance(&mut self, tol: f64) {
        self.tol = tol;
    }

    /// `SetParameters(U1, V1, U2, V2)`. `p.set_parameters(0.0, 1.0, 2.0, 3.0)`.
    pub fn set_parameters(&mut self, u1: f64, v1: f64, u2: f64, v2: f64) {
        self.pt.set_parameters(u1, v1, u2, v2);
    }

    /// `SetParameter(Para)`: the parameter on the line. `p.set_parameter(2.0)`.
    pub fn set_parameter(&mut self, para: f64) {
        self.para = para;
    }

    /// `SetVertex(OnFirst, V)`: the point is the vertex `v` of the domain of the first
    /// (or second) surface; it is on that domain too. `p.set_vertex(true, &vertex)`.
    pub fn set_vertex(&mut self, on_first: bool, v: &TheVertex) {
        let domain: &mut PointOnDomain = self.domain_mut(on_first);
        (domain.on, domain.vtx_on, domain.vertex) = (true, true, v.clone());
    }

    /// `SetArc(OnFirst, A, Param, TLine, TArc)`: the point is on the arc `a` of the first
    /// (or second) domain at `param`. `p.set_arc(true, &arc, 0.5, &tline, &tarc)`.
    pub fn set_arc(
        &mut self,
        on_first: bool,
        a: &TheArc,
        param: f64,
        tline: &IntSurfTransition,
        tarc: &IntSurfTransition,
    ) {
        let domain: &mut PointOnDomain = self.domain_mut(on_first);
        (domain.on, domain.arc, domain.traline) = (true, a.clone(), *tline);
        (domain.tra, domain.prm) = (*tarc, param);
    }

    /// `self.SetArc(OnFirst, from.ArcOnSx(), from.ParameterOnArcx(),
    /// from.TransitionLineArcx(), from.TransitionOnSx())`: Standard_DomainError when
    /// `from` is not on that domain.
    pub(crate) fn copy_arc_from(
        &mut self,
        from: &IntPatchPoint,
        on_first: bool,
    ) -> Result<(), Failure> {
        let domain: &PointOnDomain = if on_first { &from.s1 } else { &from.s2 };
        let what: &str = if on_first {
            "on the domain of S1"
        } else {
            "on the domain of S2"
        };
        let d: PointOnDomain =
            on_domain(domain, if on_first { "ArcOnS1" } else { "ArcOnS2" }, what)?.clone();
        self.set_arc(on_first, &d.arc, d.prm, &d.traline, &d.tra);
        Ok(())
    }

    fn domain_mut(&mut self, on_first: bool) -> &mut PointOnDomain {
        if on_first { &mut self.s1 } else { &mut self.s2 }
    }

    /// `SetMultiple(IsMult)`. `p.set_multiple(true)`.
    pub fn set_multiple(&mut self, is_mult: bool) {
        self.mult = is_mult;
    }

    /// `Value()`: the 3d point. `p.value()`.
    pub fn value(&self) -> Point3 {
        self.pt.value()
    }

    /// `ParameterOnLine()`. `p.parameter_on_line()`.
    pub fn parameter_on_line(&self) -> f64 {
        self.para
    }

    /// `Tolerance()`. `p.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tol
    }

    /// `IsTangencyPoint()`: the stored flag (the native member does not raise despite its
    /// documentation). `p.is_tangency_point()`.
    pub fn is_tangency_point(&self) -> bool {
        self.tgt
    }

    /// `ParametersOnS1(U1, V1)`. `p.parameters_on_s1()`.
    pub fn parameters_on_s1(&self) -> (f64, f64) {
        self.pt.parameters_on_s1()
    }

    /// `ParametersOnS2(U2, V2)`. `p.parameters_on_s2()`.
    pub fn parameters_on_s2(&self) -> (f64, f64) {
        self.pt.parameters_on_s2()
    }

    /// `IsMultiple()`. `p.is_multiple()`.
    pub fn is_multiple(&self) -> bool {
        self.mult
    }

    /// `IsOnDomS1()`. `p.is_on_dom_s1()`.
    pub fn is_on_dom_s1(&self) -> bool {
        self.s1.on
    }

    /// `IsVertexOnS1()`. `p.is_vertex_on_s1()`.
    pub fn is_vertex_on_s1(&self) -> bool {
        self.s1.vtx_on
    }

    /// `VertexOnS1()`: Standard_DomainError unless the point is a vertex on S1.
    /// `p.vertex_on_s1()?`.
    pub fn vertex_on_s1(&self) -> Result<TheVertex, Failure> {
        vertex_of(&self.s1, "VertexOnS1", "a vertex on S1")
    }

    /// `ArcOnS1()`: Standard_DomainError unless the point is on the domain of S1.
    /// `p.arc_on_s1()?`.
    pub fn arc_on_s1(&self) -> Result<TheArc, Failure> {
        on_domain(&self.s1, "ArcOnS1", "on the domain of S1").map(|d| d.arc.clone())
    }

    /// `TransitionLineArc1()`: Standard_DomainError unless on the domain of S1.
    /// `p.transition_line_arc1()?`.
    pub fn transition_line_arc1(&self) -> Result<IntSurfTransition, Failure> {
        on_domain(&self.s1, "TransitionLineArc1", "on the domain of S1").map(|d| d.traline)
    }

    /// `TransitionOnS1()`: Standard_DomainError unless on the domain of S1.
    /// `p.transition_on_s1()?`.
    pub fn transition_on_s1(&self) -> Result<IntSurfTransition, Failure> {
        on_domain(&self.s1, "TransitionOnS1", "on the domain of S1").map(|d| d.tra)
    }

    /// `ParameterOnArc1()`: Standard_DomainError unless on the domain of S1.
    /// `p.parameter_on_arc1()?`.
    pub fn parameter_on_arc1(&self) -> Result<f64, Failure> {
        on_domain(&self.s1, "ParameterOnArc1", "on the domain of S1").map(|d| d.prm)
    }

    /// `IsOnDomS2()`. `p.is_on_dom_s2()`.
    pub fn is_on_dom_s2(&self) -> bool {
        self.s2.on
    }

    /// `IsVertexOnS2()`. `p.is_vertex_on_s2()`.
    pub fn is_vertex_on_s2(&self) -> bool {
        self.s2.vtx_on
    }

    /// `VertexOnS2()`: Standard_DomainError unless the point is a vertex on S2.
    /// `p.vertex_on_s2()?`.
    pub fn vertex_on_s2(&self) -> Result<TheVertex, Failure> {
        vertex_of(&self.s2, "VertexOnS2", "a vertex on S2")
    }

    /// `ArcOnS2()`: Standard_DomainError unless the point is on the domain of S2.
    /// `p.arc_on_s2()?`.
    pub fn arc_on_s2(&self) -> Result<TheArc, Failure> {
        on_domain(&self.s2, "ArcOnS2", "on the domain of S2").map(|d| d.arc.clone())
    }

    /// `TransitionLineArc2()`: Standard_DomainError unless on the domain of S2.
    /// `p.transition_line_arc2()?`.
    pub fn transition_line_arc2(&self) -> Result<IntSurfTransition, Failure> {
        on_domain(&self.s2, "TransitionLineArc2", "on the domain of S2").map(|d| d.traline)
    }

    /// `TransitionOnS2()`: Standard_DomainError unless on the domain of S2.
    /// `p.transition_on_s2()?`.
    pub fn transition_on_s2(&self) -> Result<IntSurfTransition, Failure> {
        on_domain(&self.s2, "TransitionOnS2", "on the domain of S2").map(|d| d.tra)
    }

    /// `ParameterOnArc2()`: Standard_DomainError unless on the domain of S2.
    /// `p.parameter_on_arc2()?`.
    pub fn parameter_on_arc2(&self) -> Result<f64, Failure> {
        on_domain(&self.s2, "ParameterOnArc2", "on the domain of S2").map(|d| d.prm)
    }

    /// `PntOn2S()`: the point and its parameters. `p.pnt_on_2s()`.
    pub fn pnt_on_2s(&self) -> IntSurfPntOn2S {
        self.pt
    }

    /// `Parameters(U1, V1, U2, V2)`. `p.parameters()`.
    pub fn parameters(&self) -> (f64, f64, f64, f64) {
        self.pt.parameters()
    }

    /// `ReverseTransition()`: on each domain the point is on, an IN transition becomes a
    /// non tangent OUT and an OUT a non tangent IN; any other transition becomes the
    /// default (undecided) one, as the native code assigns a fresh transition.
    /// `p.reverse_transition()`.
    pub fn reverse_transition(&mut self) {
        for domain in [&mut self.s1, &mut self.s2] {
            if domain.on {
                domain.traline = reversed(&domain.traline);
                domain.tra = reversed(&domain.tra);
            }
        }
    }
}

/// The transition of `ReverseTransition`: a fresh `IntSurf_Transition`, set to the
/// opposite type when `t` is IN or OUT.
fn reversed(t: &IntSurfTransition) -> IntSurfTransition {
    let mut fresh = IntSurfTransition::new();
    match t.transition_type() {
        IntSurfTypeTrans::In => fresh.set_type(false, IntSurfTypeTrans::Out),
        IntSurfTypeTrans::Out => fresh.set_type(false, IntSurfTypeTrans::In),
        _ => {}
    }
    fresh
}

fn on_domain<'a>(
    domain: &'a PointOnDomain,
    member: &str,
    what: &str,
) -> Result<&'a PointOnDomain, Failure> {
    if !domain.on {
        return Err(not_on(member, what));
    }
    Ok(domain)
}

fn vertex_of(domain: &PointOnDomain, member: &str, what: &str) -> Result<TheVertex, Failure> {
    if !domain.vtx_on {
        return Err(not_on(member, what));
    }
    Ok(domain.vertex.clone())
}
