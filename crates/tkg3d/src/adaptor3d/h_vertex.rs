// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_HVertex.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor3d_HVertex`: a vertex of a restriction in the parametric space of a
//! surface, as `Adaptor3d_TopolTool` hands it out.
//!
//! The native class is virtual and BRepTopAdaptor_HVertex overrides every member, so it
//! is the trait [`Adaptor3dHVertex`] reached through `Handle<dyn Adaptor3dHVertex>`, and
//! [`Adaptor3dHVertexBase`] is an instance of the base class itself
//! (docs/topalgo-design.md section 8).
use crate::top_abs::Orientation;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::Point2;

/// The virtual members of `Adaptor3d_HVertex`: a parametric vertex with an orientation
/// and a resolution. `Parameter`, `Resolution` and `IsSame` return `Result`, because an
/// override can raise where the base class cannot.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertex, Adaptor3dHVertexBase};
/// use rustycad_tkg3d::top_abs::Orientation;
/// use rustycad_tkmath::gp::Point2;
/// let base = Adaptor3dHVertexBase::new(Point2::new(0.0, 1.0), Orientation::Reversed, 1e-8);
/// let vertex: Handle<dyn Adaptor3dHVertex> = Handle::new(base).cast();
/// assert_eq!(vertex.orientation(), Orientation::Reversed);
/// ```
pub trait Adaptor3dHVertex: Transient {
    /// The parametric point (`Value`); `vertex.value()`.
    fn value(&self) -> Point2;
    /// The parameter of the vertex on `curve` (`Parameter`); `vertex.parameter(&arc)?`.
    fn parameter(&self, curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<f64, Failure>;
    /// The parametric (2d) resolution of the vertex on `curve` (`Resolution`);
    /// `vertex.resolution(&arc)?`.
    fn resolution(&self, curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<f64, Failure>;
    /// The orientation (`Orientation`); `vertex.orientation()`.
    fn orientation(&self) -> Orientation;
    /// Whether `other` is the same vertex (`IsSame`); `vertex.is_same(&other)?`.
    fn is_same(&self, other: &Handle<dyn Adaptor3dHVertex>) -> Result<bool, Failure>;
}
impl_transient_interface!(dyn Adaptor3dHVertex, "Adaptor3d_HVertex", StandardTransient);

/// An instance of the native base class `Adaptor3d_HVertex` itself: a point, a
/// resolution and an orientation.
///
/// The default vertex is at the origin with a zero resolution and the orientation
/// that the zero-filled native allocation gives (FORWARD);
/// `Adaptor3dHVertexBase::new(Point2::new(0.0, 1.0), Orientation::Forward, 1e-8)`.
pub struct Adaptor3dHVertexBase {
    parent: StandardTransient,
    point: Point2,
    tolerance: f64,
    orientation: Orientation,
}
impl Default for Adaptor3dHVertexBase {
    fn default() -> Self {
        Self::new(Point2::default(), Orientation::Forward, 0.0)
    }
}
impl_transient!(
    Adaptor3dHVertexBase,
    "Adaptor3d_HVertex",
    StandardTransient,
    parent,
    interfaces: [dyn Adaptor3dHVertex]
);

impl Adaptor3dHVertexBase {
    /// A vertex at `point` (`Adaptor3d_HVertex(P, Ori, Resolution)`);
    /// `Adaptor3dHVertexBase::new(p, Orientation::Reversed, 1e-8)`.
    pub fn new(point: Point2, orientation: Orientation, resolution: f64) -> Self {
        Self {
            parent: StandardTransient::default(),
            point,
            tolerance: resolution,
            orientation,
        }
    }
}

impl Adaptor3dHVertex for Adaptor3dHVertexBase {
    fn value(&self) -> Point2 {
        self.point
    }
    // ElCLib::Parameter on `C->Line()`: the curve must be a line.
    fn parameter(&self, curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<f64, Failure> {
        let line = live_curve(curve)?.line()?;
        elclib::parameter(&line, self.point)
    }
    // The resolution, whatever the curve.
    fn resolution(&self, _curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<f64, Failure> {
        Ok(self.tolerance)
    }
    fn orientation(&self) -> Orientation {
        self.orientation
    }
    // Within Precision::Confusion() of the point of `other`, read through its own
    // (virtual) `Value`.
    fn is_same(&self, other: &Handle<dyn Adaptor3dHVertex>) -> Result<bool, Failure> {
        let other_point: Point2 = live_vertex(other)?.value();
        Ok(self.point.distance(other_point) <= precision::confusion())
    }
}

// The objects behind the handles; natively a null one is dereferenced.
fn live_curve(curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<&dyn Adaptor2dCurve2d, Failure> {
    curve.get().ok_or_else(|| {
        crate::geom::errors::null_object(
            "Adaptor3d_HVertex::Parameter: null curve; expected a line",
        )
    })
}
fn live_vertex(vertex: &Handle<dyn Adaptor3dHVertex>) -> Result<&dyn Adaptor3dHVertex, Failure> {
    vertex.get().ok_or_else(|| {
        crate::geom::errors::null_object(
            "Adaptor3d_HVertex::IsSame: null vertex; expected a vertex",
        )
    })
}
