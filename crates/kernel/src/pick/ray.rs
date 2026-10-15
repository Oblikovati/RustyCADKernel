// SPDX-License-Identifier: GPL-2.0-only
//! A pick ray: an origin and a unit direction.
use super::refused;
use crate::query_error::{BodyQueryError, BodyQueryFailure, QueryEntity};
use rustycad_tkmath::gp::{Direction3, Line3, Point3};

/// A half-line from `origin` along a unit `direction`; hit distances are measured
/// along it from the origin.
///
/// ```
/// use rustycad_kernel::pick::PickRay;
/// let ray = PickRay::new([0.0, 0.0, 5.0], [0.0, 0.0, -2.0])?;
/// assert_eq!(ray.direction(), [0.0, 0.0, -1.0]);
/// assert!(PickRay::new([0.0; 3], [0.0; 3]).is_err());
/// # Ok::<(), rustycad_kernel::query_error::BodyQueryError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PickRay {
    origin: [f64; 3],
    direction: [f64; 3],
}

impl PickRay {
    /// A ray from a finite origin along a finite non-zero direction (normalized);
    /// anything else is an [`BodyQueryFailure::InvalidRay`] refusal.
    pub fn new(origin: [f64; 3], direction: [f64; 3]) -> Result<Self, BodyQueryError> {
        let finite = origin
            .iter()
            .chain(&direction)
            .all(|value| value.is_finite());
        let unit = Direction3::new(direction[0], direction[1], direction[2]);
        match (finite, unit) {
            (true, Ok(unit)) => Ok(Self {
                origin,
                direction: [unit.x(), unit.y(), unit.z()],
            }),
            _ => Err(invalid_ray(origin, direction)),
        }
    }

    /// The ray origin; `ray.origin()`.
    pub fn origin(&self) -> [f64; 3] {
        self.origin
    }

    /// The unit ray direction; `ray.direction()`.
    pub fn direction(&self) -> [f64; 3] {
        self.direction
    }

    /// The supporting line, parameterized by distance from the origin.
    pub(crate) fn line(&self) -> Line3 {
        let [x, y, z] = self.origin;
        let [dx, dy, dz] = self.direction;
        let direction = Direction3::new(dx, dy, dz).expect("unit direction");
        Line3::new(Point3::new(x, y, z), direction)
    }
}

fn invalid_ray(origin: [f64; 3], direction: [f64; 3]) -> BodyQueryError {
    let cause = format!(
        "ray origin {origin:?}, direction {direction:?}: expected finite coordinates and \
         a non-zero direction"
    );
    refused(QueryEntity::Body, BodyQueryFailure::InvalidRay, cause)
}
