//! Planar coordinate algebra and geometric primitives with OCCT operation order.
mod coordinates;
mod direction;
mod point;
mod transform;
mod vector;
use super::{Failure, TransformForm, construction};
pub use coordinates::Xy;
pub use direction::Direction2;
pub use point::{Axis2d, Point2};
pub use transform::Transform2;
pub use vector::Vector2;
fn oriented_angle(cosine: f64, sine: f64, threshold: f64) -> f64 {
    if cosine > -threshold && cosine < threshold {
        return if sine > 0.0 {
            cosine.acos()
        } else {
            -cosine.acos()
        };
    }
    if cosine > 0.0 {
        return sine.asin();
    }
    if sine > 0.0 {
        std::f64::consts::PI - sine.asin()
    } else {
        -std::f64::consts::PI - sine.asin()
    }
}
mod line;
pub use line::Line2;
