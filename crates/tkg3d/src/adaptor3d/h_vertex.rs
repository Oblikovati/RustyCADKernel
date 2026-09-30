// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_HVertex.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor3d_HVertex`: a vertex of a restriction in the parametric space of a
//! surface, as `Adaptor3d_TopolTool` hands it out.
use crate::top_abs::Orientation;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::Point2;

/// A parametric vertex with an orientation and a resolution (`Adaptor3d_HVertex`).
///
/// The default vertex is at the origin with a zero resolution and the orientation
/// that the zero-filled native allocation gives (FORWARD);
/// `Adaptor3dHVertex::new(Point2::new(0.0, 1.0), Orientation::Forward, 1e-8)`.
pub struct Adaptor3dHVertex {
    parent: StandardTransient,
    point: Point2,
    tolerance: f64,
    orientation: Orientation,
}
impl Default for Adaptor3dHVertex {
    fn default() -> Self {
        Self::new(Point2::default(), Orientation::Forward, 0.0)
    }
}
impl_transient!(
    Adaptor3dHVertex,
    "Adaptor3d_HVertex",
    StandardTransient,
    parent
);

impl Adaptor3dHVertex {
    /// A vertex at `point` (`Adaptor3d_HVertex(P, Ori, Resolution)`);
    /// `Adaptor3dHVertex::new(p, Orientation::Reversed, 1e-8)`.
    pub fn new(point: Point2, orientation: Orientation, resolution: f64) -> Self {
        Self {
            parent: StandardTransient::default(),
            point,
            tolerance: resolution,
            orientation,
        }
    }
    /// The parametric point (`Value`); `vertex.value()`.
    pub fn value(&self) -> Point2 {
        self.point
    }
    /// The parameter of the point on `curve`, which must be a line (`Parameter`:
    /// ElCLib::Parameter on `C->Line()`); `vertex.parameter(&line)?`.
    pub fn parameter(&self, curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<f64, Failure> {
        let line = live_curve(curve)?.line()?;
        elclib::parameter(&line, self.point)
    }
    /// The resolution, whatever the curve (`Resolution`); `vertex.resolution(&curve)`.
    pub fn resolution(&self, _curve: &Handle<dyn Adaptor2dCurve2d>) -> f64 {
        self.tolerance
    }
    /// The orientation (`Orientation`); `vertex.orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }
    /// Whether the other vertex is within Precision::Confusion() (`IsSame`);
    /// `vertex.is_same(&other)?`.
    pub fn is_same(&self, other: &Handle<Adaptor3dHVertex>) -> Result<bool, Failure> {
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
fn live_vertex(vertex: &Handle<Adaptor3dHVertex>) -> Result<&Adaptor3dHVertex, Failure> {
    vertex.get().ok_or_else(|| {
        crate::geom::errors::null_object(
            "Adaptor3d_HVertex::IsSame: null vertex; expected a vertex",
        )
    })
}
