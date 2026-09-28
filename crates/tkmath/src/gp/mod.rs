//! OCCT geometric primitives with explicit native numeric behavior.
mod axis;
pub(crate) mod direction;
mod point;
pub use axis::Axis3;
pub use direction::{Direction3, StandardDirection3};
pub use point::Point3;
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
fn construction(message: &str) -> Failure {
    Failure::with_type("Standard_ConstructionError", message, "")
}
fn domain(message: &str) -> Failure {
    Failure::with_type("Standard_DomainError", message, "")
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
