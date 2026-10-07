// Rust translation of ChFiDS_CommonPoint.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The start or end point of a fillet on a face: a vertex, a point on an arc, or both.
use super::errors::domain_error;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point3, Vector3};

/// A point of a fillet extremity on a face (`ChFiDS_CommonPoint`): its 3d point, an optional
/// tangent vector, an optional vertex and an optional arc with the parameter and transition
/// on it, the parameter on the guideline and a tolerance that only grows.
///
/// ```
/// use rustycad_tkfillet::chfi_ds::CommonPoint;
/// let mut point = CommonPoint::new();
/// point.set_tolerance(1e-3);
/// point.set_tolerance(1e-5);
/// assert_eq!(point.tolerance(), 1e-3);
/// assert!(point.arc().is_err());
/// ```
#[derive(Clone, Debug)]
pub struct CommonPoint {
    arc: Edge,
    vertex: Vertex,
    point: Point3,
    vector: Vector3,
    tolerance: f64,
    parameter_on_arc: f64,
    parameter: f64,
    transition_on_arc: Orientation,
    is_on_arc: bool,
    is_vertex: bool,
    has_vector: bool,
}

impl Default for CommonPoint {
    fn default() -> Self {
        Self {
            arc: Edge::default(),
            vertex: Vertex::default(),
            point: Point3::default(),
            vector: Vector3::default(),
            tolerance: 0.0,
            parameter_on_arc: 0.0,
            parameter: 0.0,
            transition_on_arc: Orientation::Forward,
            is_on_arc: false,
            is_vertex: false,
            has_vector: false,
        }
    }
}

impl CommonPoint {
    /// An empty point: no vertex, no arc, no vector, zero tolerance; `CommonPoint::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Reset()`: clears the tolerance and the vertex, arc and vector flags, keeping the
    /// stored values (natively the fields are kept too); `point.reset()`.
    pub fn reset(&mut self) {
        self.tolerance = 0.0;
        self.is_vertex = false;
        self.is_on_arc = false;
        self.has_vector = false;
    }

    /// `SetVertex(V)`: the point is the vertex `vertex`; `point.set_vertex(&v)`.
    pub fn set_vertex(&mut self, vertex: &Vertex) {
        self.is_vertex = true;
        self.vertex = vertex.clone();
    }

    /// `SetArc(Tol, A, Param, TArc)`: the point lies on `arc` at `parameter` with the
    /// transition `transition`; the tolerance grows to `tolerance`;
    /// `point.set_arc(1e-7, &edge, 0.5, Orientation::Forward)`.
    pub fn set_arc(&mut self, tolerance: f64, arc: &Edge, parameter: f64, transition: Orientation) {
        self.is_on_arc = true;
        self.set_tolerance(tolerance);
        self.arc = arc.clone();
        self.parameter_on_arc = parameter;
        self.transition_on_arc = transition;
    }

    /// `SetParameter(Param)`: the parameter on the guideline; `point.set_parameter(2.0)`.
    pub fn set_parameter(&mut self, parameter: f64) {
        self.parameter = parameter;
    }

    /// `SetPoint(P)`: the 3d point; `point.set_point(Point3::default())`.
    pub fn set_point(&mut self, point: Point3) {
        self.point = point;
    }

    /// `SetVector(V)`: the tangent vector; `point.set_vector(Vector3::new(1., 0., 0.))`.
    pub fn set_vector(&mut self, vector: Vector3) {
        self.has_vector = true;
        self.vector = vector;
    }

    /// `SetTolerance(Tol)`: raises the tolerance to `tolerance` when larger (a NaN never
    /// replaces it, as the native `Tol > tol` comparison); `point.set_tolerance(1e-4)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        if tolerance > self.tolerance {
            self.tolerance = tolerance;
        }
    }

    /// `Tolerance()`; `point.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }

    /// `IsVertex()`; `point.is_vertex()`.
    pub fn is_vertex(&self) -> bool {
        self.is_vertex
    }

    /// `Vertex()`: Standard_DomainError when the point is not a vertex; `point.vertex()?`.
    pub fn vertex(&self) -> Result<&Vertex, Failure> {
        if !self.is_vertex {
            return Err(domain_error(
                "ChFiDS_CommonPoint::Vertex: the point is not a vertex; expected SetVertex first",
            ));
        }
        Ok(&self.vertex)
    }

    /// `IsOnArc()`; `point.is_on_arc()`.
    pub fn is_on_arc(&self) -> bool {
        self.is_on_arc
    }

    /// `Arc()`: Standard_DomainError when the point is not on an arc; `point.arc()?`.
    pub fn arc(&self) -> Result<&Edge, Failure> {
        self.require_arc()?;
        Ok(&self.arc)
    }

    /// `TransitionOnArc()`: Standard_DomainError when not on an arc; `point.transition_on_arc()?`.
    pub fn transition_on_arc(&self) -> Result<Orientation, Failure> {
        self.require_arc()?;
        Ok(self.transition_on_arc)
    }

    /// `ParameterOnArc()`: Standard_DomainError when not on an arc; `point.parameter_on_arc()?`.
    pub fn parameter_on_arc(&self) -> Result<f64, Failure> {
        self.require_arc()?;
        Ok(self.parameter_on_arc)
    }

    /// `Parameter()`: the parameter on the guideline; `point.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.parameter
    }

    /// `Point()`; `point.point()`.
    pub fn point(&self) -> Point3 {
        self.point
    }

    /// `HasVector()`; `point.has_vector()`.
    pub fn has_vector(&self) -> bool {
        self.has_vector
    }

    /// `Vector()`: Standard_DomainError when no vector was set; `point.vector()?`.
    pub fn vector(&self) -> Result<Vector3, Failure> {
        if !self.has_vector {
            return Err(domain_error("ChFiDS_CommonPoint::Vector"));
        }
        Ok(self.vector)
    }

    fn require_arc(&self) -> Result<(), Failure> {
        if self.is_on_arc {
            return Ok(());
        }
        Err(domain_error("CommonPoint not on Arc"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_members_raise_domain_error_until_set() {
        let point = CommonPoint::new();
        assert_eq!(
            point.arc().unwrap_err().exception_type(),
            "Standard_DomainError"
        );
        assert!(point.transition_on_arc().is_err());
        assert!(point.parameter_on_arc().is_err());
    }

    #[test]
    fn set_arc_raises_the_tolerance_and_records_the_transition() {
        let mut point = CommonPoint::new();
        point.set_arc(1e-4, &Edge::default(), 0.25, Orientation::Reversed);
        assert_eq!(point.tolerance(), 1e-4);
        assert_eq!(point.parameter_on_arc().unwrap(), 0.25);
        assert_eq!(point.transition_on_arc().unwrap(), Orientation::Reversed);
        point.set_arc(1e-6, &Edge::default(), 0.5, Orientation::Forward);
        assert_eq!(point.tolerance(), 1e-4);
    }

    #[test]
    fn reset_clears_flags_and_tolerance() {
        let mut point = CommonPoint::new();
        point.set_vertex(&Vertex::default());
        point.set_vector(Vector3::new(1., 0., 0.));
        point.set_arc(1.0, &Edge::default(), 0.0, Orientation::Forward);
        point.reset();
        assert!(!point.is_vertex() && !point.is_on_arc() && !point.has_vector());
        assert_eq!(point.tolerance(), 0.0);
        assert!(point.vertex().is_err() && point.vector().is_err());
    }

    #[test]
    fn nan_tolerance_never_replaces_the_current_one() {
        let mut point = CommonPoint::new();
        point.set_tolerance(f64::NAN);
        assert_eq!(point.tolerance(), 0.0);
    }
}
