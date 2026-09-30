// Rust translation of ModelingAlgorithms/TKHLR/Contap/Contap_Point.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Contap_Point: a vertex of a contour line.
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_start::{TheArc, TheVertex};
use rustycad_tkgeomalgo::int_surf::IntSurfTransition;
use rustycad_tkmath::gp::Point3;

/// Contap_Point: a point of a contour line with its surface parameters, its parameter
/// on the line, and optionally its arc of restriction (parameter and transitions) and
/// its vertex; it can be multiple (on several lines) or internal.
///
/// ```
/// use rustycad_tkhlr::contap::ContapPoint;
/// use rustycad_tkmath::gp::Point3;
/// let mut point = ContapPoint::with_value(Point3::new(1.0, 2.0, 3.0), 0.5, 0.25);
/// point.set_parameter(4.0);
/// point.set_multiple();
/// assert_eq!((point.parameter_on_line(), point.parameters()), (4.0, (0.5, 0.25)));
/// assert!(point.is_multiple() && !point.is_on_arc());
/// assert_eq!(point.arc().err().unwrap().exception_type(), "Standard_DomainError");
/// ```
#[derive(Clone, Default)]
pub struct ContapPoint {
    pt: Point3,
    uparam: f64,
    vparam: f64,
    paraline: f64,
    onarc: bool,
    arc: TheArc,
    traline: IntSurfTransition,
    traarc: IntSurfTransition,
    prmarc: f64,
    isvtx: bool,
    vtx: TheVertex,
    ismult: bool,
    internal: bool,
}

/// The Standard_DomainError of an accessor used on the wrong kind of point.
fn not_on(member: &str, what: &str) -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        &format!("Contap_Point::{member}: the point is not {what}, expected a point {what}"),
    )
}

impl ContapPoint {
    /// Contap_Point(): the origin at parameters 0, on no arc; `ContapPoint::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Contap_Point(Pt, U, V): the point `pt` at the surface parameters `(u, v)`;
    /// `ContapPoint::with_value(p, 0.5, 0.5)`.
    pub fn with_value(pt: Point3, u: f64, v: f64) -> Self {
        Self {
            pt,
            uparam: u,
            vparam: v,
            ..Self::default()
        }
    }

    /// SetValue(Pt, U, V): the point and its parameters; clears the arc, vertex,
    /// multiple and internal flags (not the arc data nor the parameter on the line);
    /// `point.set_value(p, 0.5, 0.5)`.
    pub fn set_value(&mut self, pt: Point3, u: f64, v: f64) {
        self.pt = pt;
        self.uparam = u;
        self.vparam = v;
        self.onarc = false;
        self.isvtx = false;
        self.ismult = false;
        self.internal = false;
    }

    /// SetParameter: the parameter on the line; `point.set_parameter(2.0)`.
    pub fn set_parameter(&mut self, para: f64) {
        self.paraline = para;
    }

    /// SetVertex: the point is the vertex `v`; `point.set_vertex(&vertex)`.
    pub fn set_vertex(&mut self, v: &TheVertex) {
        self.isvtx = true;
        self.vtx = v.clone();
    }

    /// SetArc: the point is on the arc `a` at `param`, with the transitions of the
    /// line and of the arc; `point.set_arc(&arc, 0.5, t_line, t_arc)`.
    pub fn set_arc(
        &mut self,
        a: &TheArc,
        param: f64,
        t_line: IntSurfTransition,
        t_arc: IntSurfTransition,
    ) {
        self.onarc = true;
        self.arc = a.clone();
        self.prmarc = param;
        self.traline = t_line;
        self.traarc = t_arc;
    }

    /// SetMultiple; `point.set_multiple()`.
    pub fn set_multiple(&mut self) {
        self.ismult = true;
    }

    /// SetInternal; `point.set_internal()`.
    pub fn set_internal(&mut self) {
        self.internal = true;
    }

    /// Value; `point.value()`.
    pub fn value(&self) -> Point3 {
        self.pt
    }

    /// ParameterOnLine; `point.parameter_on_line()`.
    pub fn parameter_on_line(&self) -> f64 {
        self.paraline
    }

    /// Parameters: `(u, v)`; `point.parameters()`.
    pub fn parameters(&self) -> (f64, f64) {
        (self.uparam, self.vparam)
    }

    /// IsOnArc; `point.is_on_arc()`.
    pub fn is_on_arc(&self) -> bool {
        self.onarc
    }

    /// Arc: Standard_DomainError unless the point is on an arc; `point.arc()?`.
    pub fn arc(&self) -> Result<TheArc, Failure> {
        self.on_arc("Arc")?;
        Ok(self.arc.clone())
    }

    /// ParameterOnArc: Standard_DomainError unless on an arc; `point.parameter_on_arc()?`.
    pub fn parameter_on_arc(&self) -> Result<f64, Failure> {
        self.on_arc("ParameterOnArc")?;
        Ok(self.prmarc)
    }

    /// TransitionOnLine: Standard_DomainError unless on an arc;
    /// `point.transition_on_line()?`.
    pub fn transition_on_line(&self) -> Result<IntSurfTransition, Failure> {
        self.on_arc("TransitionOnLine")?;
        Ok(self.traline)
    }

    /// TransitionOnArc: Standard_DomainError unless on an arc;
    /// `point.transition_on_arc()?`.
    pub fn transition_on_arc(&self) -> Result<IntSurfTransition, Failure> {
        self.on_arc("TransitionOnArc")?;
        Ok(self.traarc)
    }

    fn on_arc(&self, member: &str) -> Result<(), Failure> {
        if self.onarc {
            return Ok(());
        }
        Err(not_on(member, "on an arc"))
    }

    /// IsVertex; `point.is_vertex()`.
    pub fn is_vertex(&self) -> bool {
        self.isvtx
    }

    /// Vertex: Standard_DomainError unless the point is a vertex; `point.vertex()?`.
    pub fn vertex(&self) -> Result<TheVertex, Failure> {
        if !self.isvtx {
            return Err(not_on("Vertex", "a vertex"));
        }
        Ok(self.vtx.clone())
    }

    /// IsMultiple; `point.is_multiple()`.
    pub fn is_multiple(&self) -> bool {
        self.ismult
    }

    /// IsInternal; `point.is_internal()`.
    pub fn is_internal(&self) -> bool {
        self.internal
    }
}
