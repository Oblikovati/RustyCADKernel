//! Endpoint precedence, retaining OCCT's evaluation and tie ordering.
use super::CurveProjection;
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;

pub(super) fn snapping_tolerance(tolerance: f64, adjust: bool) -> f64 {
    if adjust {
        tolerance
    } else {
        precision::confusion()
    }
}
pub(super) struct EndpointCandidates {
    low: CurveProjection,
    high: CurveProjection,
}
impl EndpointCandidates {
    pub fn new(curve: &dyn Adaptor3dCurve) -> Result<Self, Failure> {
        let first = curve.first_parameter()?;
        let last = curve.last_parameter()?;
        // Native evaluates both endpoints before testing either distance.
        let low = CurveProjection {
            point: curve.value(first)?,
            parameter: first,
        };
        let high = CurveProjection {
            point: curve.value(last)?,
            parameter: last,
        };
        Ok(Self { low, high })
    }
    pub fn distances(&self, point: Point3) -> (f64, f64) {
        (
            self.low.point.distance(&point),
            self.high.point.distance(&point),
        )
    }
    pub fn snap(&self, point: Point3, tolerance: f64, output: &mut CurveProjection) -> Option<f64> {
        let distance = self.low.point.distance(&point);
        if distance <= tolerance {
            *output = self.low;
            return Some(distance);
        }
        let distance = self.high.point.distance(&point);
        if distance <= tolerance {
            *output = self.high;
            return Some(distance);
        }
        None
    }
    pub fn snap_distances(
        &self,
        distances: (f64, f64),
        tolerance: f64,
        output: &mut CurveProjection,
    ) -> Option<f64> {
        if distances.0 <= tolerance {
            *output = self.low;
            return Some(distances.0);
        }
        if distances.1 <= tolerance {
            *output = self.high;
            return Some(distances.1);
        }
        None
    }
    pub fn prefer_projection(
        &self,
        distances: (f64, f64),
        distance: f64,
        output: &mut CurveProjection,
    ) -> f64 {
        if distance < distances.0 + precision::confusion()
            && distance < distances.1 + precision::confusion()
        {
            return distance;
        }
        if distances.0 < distances.1 {
            *output = self.low;
            return distances.0;
        }
        *output = self.high;
        distances.1
    }
}
