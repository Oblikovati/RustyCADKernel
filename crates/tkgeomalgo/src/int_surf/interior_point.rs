// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntSurf/IntSurf_InteriorPoint.hxx/.lxx/.cxx
// and IntSurf_InteriorPointTool.hxx/.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntSurf_InteriorPoint` (a start point inside a domain) and its walking tool.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction2, Point3, Vector2, Vector3};

/// A point inside the domain where a marching algorithm starts, with its parameters
/// and the tangent of the intersection in 3d and in the parametric space
/// (`IntSurf_InteriorPoint`).
///
/// `IntSurfInteriorPoint::new().u_parameter() == 0.0`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IntSurfInteriorPoint {
    point: Point3,
    paramu: f64,
    paramv: f64,
    direc: Vector3,
    direc2d: Vector2,
}

impl IntSurfInteriorPoint {
    /// The origin with null parameters and tangents (`IntSurf_InteriorPoint()`).
    /// `IntSurfInteriorPoint::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// A point with its parameters and tangents (`IntSurf_InteriorPoint(P, U, V, Direc, Direc2d)`).
    /// `IntSurfInteriorPoint::with_value(p, 0.5, 0.5, tangent, tangent2d)`.
    pub fn with_value(p: Point3, u: f64, v: f64, direc: Vector3, direc2d: Vector2) -> Self {
        Self {
            point: p,
            paramu: u,
            paramv: v,
            direc,
            direc2d,
        }
    }

    /// Replaces every field (`SetValue`); `ip.set_value(p, 0.5, 0.5, tangent, tangent2d)`.
    pub fn set_value(&mut self, p: Point3, u: f64, v: f64, direc: Vector3, direc2d: Vector2) {
        *self = Self::with_value(p, u, v, direc, direc2d);
    }

    /// The 3d point (`Value`); `ip.value()`.
    pub const fn value(&self) -> Point3 {
        self.point
    }

    /// `(U, V)` on the parametric surface (`Parameters`); `ip.parameters()`.
    pub const fn parameters(&self) -> (f64, f64) {
        (self.paramu, self.paramv)
    }

    /// U on the parametric surface (`UParameter`); `ip.u_parameter()`.
    pub const fn u_parameter(&self) -> f64 {
        self.paramu
    }

    /// V on the parametric surface (`VParameter`); `ip.v_parameter()`.
    pub const fn v_parameter(&self) -> f64 {
        self.paramv
    }

    /// The 3d tangent of the intersection (`Direction`); `ip.direction()`.
    pub const fn direction(&self) -> Vector3 {
        self.direc
    }

    /// The parametric tangent of the intersection (`Direction2d`); `ip.direction2d()`.
    pub const fn direction2d(&self) -> Vector2 {
        self.direc2d
    }
}

/// The tool through which the walking algorithms read an interior point
/// (`IntSurf_InteriorPointTool`); `IntSurfInteriorPointTool::value3d(&ip)`.
pub struct IntSurfInteriorPointTool;

impl IntSurfInteriorPointTool {
    /// The 3d point (`Value3d`); `IntSurfInteriorPointTool::value3d(&ip)`.
    pub const fn value3d(p_start: &IntSurfInteriorPoint) -> Point3 {
        p_start.value()
    }

    /// The start parameters (`Value2d`); `IntSurfInteriorPointTool::value2d(&ip)`.
    pub const fn value2d(p_start: &IntSurfInteriorPoint) -> (f64, f64) {
        p_start.parameters()
    }

    /// The 3d tangent (`Direction3d`); `IntSurfInteriorPointTool::direction3d(&ip)`.
    pub const fn direction3d(p_start: &IntSurfInteriorPoint) -> Vector3 {
        p_start.direction()
    }

    /// The parametric tangent as a direction (`Direction2d`, the native implicit
    /// `gp_Dir2d(gp_Vec2d)`), Standard_ConstructionError for a null tangent.
    /// `IntSurfInteriorPointTool::direction2d(&ip)?`.
    pub fn direction2d(p_start: &IntSurfInteriorPoint) -> Result<Direction2, Failure> {
        Direction2::from_vector(p_start.direction2d())
    }
}
