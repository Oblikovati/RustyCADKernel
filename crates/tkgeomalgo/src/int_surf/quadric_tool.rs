// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntSurf/IntSurf_QuadricTool.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntSurf_QuadricTool`: the implicit-function tool of the quadrics.
use super::IntSurfQuadric;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The tool through which the implicit-surface algorithms evaluate a quadric
/// (`IntSurf_QuadricTool`); `IntSurfQuadricTool::value(&quadric, 1.0, 0.0, 0.0)`.
pub struct IntSurfQuadricTool;

impl IntSurfQuadricTool {
    /// The value of the implicit function at `(x, y, z)` (`Value`).
    /// `IntSurfQuadricTool::value(&quadric, 1.0, 0.0, 0.0)`.
    pub fn value(quad: &IntSurfQuadric, x: f64, y: f64, z: f64) -> f64 {
        quad.distance(Point3::new(x, y, z))
    }

    /// The gradient at `(x, y, z)` (`Gradient`); `IntSurfQuadricTool::gradient(&quadric, 1.0, 0.0, 0.0)`.
    pub fn gradient(quad: &IntSurfQuadric, x: f64, y: f64, z: f64) -> Vector3 {
        quad.gradient(Point3::new(x, y, z))
    }

    /// The value and the gradient (`ValueAndGradient`); `None` for an undefined quadric,
    /// whose native output arguments are left unchanged.
    /// `IntSurfQuadricTool::value_and_gradient(&quadric, 1.0, 0.0, 0.0)`.
    pub fn value_and_gradient(
        quad: &IntSurfQuadric,
        x: f64,
        y: f64,
        z: f64,
    ) -> Option<(f64, Vector3)> {
        quad.val_and_grad(Point3::new(x, y, z))
    }

    /// The tolerance of the zero of the implicit function (`Tolerance`): 2e-6 times the
    /// radius of a sphere or cylinder, 1e-6 otherwise. `IntSurfQuadricTool::tolerance(&quadric)`.
    pub fn tolerance(quad: &IntSurfQuadric) -> f64 {
        match quad.type_quadric() {
            // The radius is read from the stored parameter, as the native Sphere() and
            // Cylinder() do before Radius().
            SurfaceType::Sphere | SurfaceType::Cylinder => 2.0e-6 * quad.prm1,
            _ => 1e-6,
        }
    }
}
