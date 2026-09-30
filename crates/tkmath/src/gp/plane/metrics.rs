//! gp_Pln distances and containment tests.
//!
//! Native quirk: the line and plane overloads return 0 unless the line direction is normal
//! to the plane normal (the line is parallel to the plane), respectively unless the normals
//! are parallel, both within gp::Resolution rather than any user tolerance.
use super::super::{Line3, Plane3, Point3};
impl Plane3 {
    /// Measures along the stored normal, independently of handedness; `plane.signed_distance(point)`.
    pub fn signed_distance(&self, point: Point3) -> f64 {
        let n = self.position.direction();
        let o = self.location();
        n.x() * (point.x() - o.x()) + n.y() * (point.y() - o.y()) + n.z() * (point.z() - o.z())
    }
    /// Signed distance of a parallel line's origin, 0 for a crossing line;
    /// `plane.signed_distance_line(line)`.
    pub fn signed_distance_line(&self, line: Line3) -> f64 {
        let normal = self.position.direction();
        if !normal.is_normal(line.direction(), f64::MIN_POSITIVE) {
            return 0.0;
        }
        self.signed_distance(line.location())
    }
    /// Signed distance of a parallel plane's origin, 0 for a crossing plane;
    /// `plane.signed_distance_plane(other)`.
    pub fn signed_distance_plane(&self, other: Self) -> f64 {
        let normal = self.position.direction();
        if !normal.is_parallel(other.position.direction(), f64::MIN_POSITIVE) {
            return 0.0;
        }
        self.signed_distance(other.location())
    }
    /// Returns absolute normal distance; `plane.distance(point)`.
    pub fn distance(&self, point: Point3) -> f64 {
        self.signed_distance(point).abs()
    }
    /// Returns the absolute line distance; `plane.distance_line(line)`.
    pub fn distance_line(&self, line: Line3) -> f64 {
        self.signed_distance_line(line).abs()
    }
    /// Returns the absolute plane distance; `plane.distance_plane(other)`.
    pub fn distance_plane(&self, other: Self) -> f64 {
        self.signed_distance_plane(other).abs()
    }
    /// Squares the point distance; `plane.square_distance(point)`.
    pub fn square_distance(&self, point: Point3) -> f64 {
        square(self.distance(point))
    }
    /// Squares the line distance; `plane.square_distance_line(line)`.
    pub fn square_distance_line(&self, line: Line3) -> f64 {
        square(self.distance_line(line))
    }
    /// Squares the plane distance; `plane.square_distance_plane(other)`.
    pub fn square_distance_plane(&self, other: Self) -> f64 {
        square(self.distance_plane(other))
    }
    /// Includes points within the supplied tolerance; `plane.contains(point,1e-7)`.
    pub fn contains(&self, point: Point3, tolerance: f64) -> bool {
        self.distance(point) <= tolerance
    }
    /// Includes a line whose origin lies in the plane and whose direction is normal to the
    /// plane normal; `plane.contains_line(line,1e-7,1e-12)`.
    pub fn contains_line(&self, line: Line3, linear: f64, angular: f64) -> bool {
        self.contains(line.location(), linear)
            && self
                .position
                .direction()
                .is_normal(line.direction(), angular)
    }
}
fn square(value: f64) -> f64 {
    value * value
}
