//! OCCT geometric primitives with explicit native numeric behavior.
mod axis;
pub(crate) mod direction;
mod point;
pub use axis::Axis3;
pub use direction::{Direction3, StandardDirection3};
pub use point::{Point3, PointCoordinatesMut, PointHashPolicy};
use rustycad_tkernel::standard::Failure;
pub(crate) fn cross(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}
pub(crate) fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}
/// Reflects `value` about the unit line `normal` with the precomputed terms and operation
/// order shared by gp_Dir::Mirror and gp_Vec::Mirror; `reflect([1.0,0.0,0.0],[1.0,2.0,3.0])`.
pub(crate) fn reflect(normal: [f64; 3], value: [f64; 3]) -> [f64; 3] {
    let [a, b, c] = normal;
    let [x, y, z] = value;
    let (m1, m2, m3) = (2.0 * a * b, 2.0 * a * c, 2.0 * b * c);
    [
        ((2.0 * a * a) - 1.0) * x + m1 * y + m2 * z,
        m1 * x + ((2.0 * b * b) - 1.0) * y + m3 * z,
        m2 * x + m3 * y + ((2.0 * c * c) - 1.0) * z,
    ]
}
fn construction(message: &str) -> Failure {
    Failure::with_type("Standard_ConstructionError", message, "")
}
fn domain(message: &str) -> Failure {
    Failure::with_type("Standard_DomainError", message, "")
}
/// Raises gp_VectorWithNullMagnitude, the gp_Vec/gp_Vec2d angular-query exception (a
/// Standard_DomainError subclass declared by DEFINE_STANDARD_EXCEPTION);
/// `gp::vector_with_null_magnitude("magnitude 0")`.
pub fn vector_with_null_magnitude(message: &str) -> Failure {
    Failure::with_type("gp_VectorWithNullMagnitude", message, "")
}
mod placement;

mod transform;
pub use transform::{Transform3, TransformForm, TransformNLerp};
mod xyz;
pub use xyz::Xyz;
mod matrix;
pub use matrix::Matrix3;

mod axis1;
mod axis2;
pub use axis1::Axis1;
pub use axis2::Axis2;
mod vector;
pub use vector::Vector3;
mod general_transform;
pub use general_transform::GeneralTransform3;
mod planar;
pub use planar::{
    Axis2d, Axis22d, Direction2, GeneralTransform2, Matrix2, Point2, StandardDirection2,
    Transform2, Vector2, Xy,
};
mod line;
pub use line::Line3;
pub use planar::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};

mod plane;
pub use plane::Plane3;

mod circle;
pub use circle::Circle3;
mod ellipse;
pub use ellipse::Ellipse3;
mod hyperbola;
pub use hyperbola::Hyperbola3;
mod parabola;
pub use parabola::Parabola3;
mod cone;
mod cylinder;
mod sphere;
mod torus;
pub use cone::Cone3;
pub use cylinder::Cylinder3;
pub use sphere::Sphere3;
pub use torus::Torus3;

mod quaternion;
pub use quaternion::{EulerSequence, Quaternion, QuaternionNLerp, QuaternionSLerp};
mod standard_frames;
pub use standard_frames::{
    dx, dx2d, dy, dy2d, dz, origin, origin2d, ox, ox2d, oy, oy2d, oz, resolution, xoy, yoz, zox,
};
