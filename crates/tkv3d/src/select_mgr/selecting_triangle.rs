//! Shared native barycentric and nearest-edge arithmetic for selecting rays.
use rustycad_tkmath::gp::{Axis1, Point3, Xyz};
pub(super) struct SelectingTriangle {
    pub points: [Point3; 3],
    pub edges: [Xyz; 3],
    pub normal: Xyz,
}
impl SelectingTriangle {
    pub fn new(points: [Point3; 3]) -> Self {
        let edges = [
            points[1].xyz() - points[0].xyz(),
            points[2].xyz() - points[1].xyz(),
            points[0].xyz() - points[2].xyz(),
        ];
        Self {
            points,
            edges,
            normal: edges[2].crossed(edges[0]),
        }
    }
    pub fn ray_parameters(&self, axis: Axis1, alpha: f64) -> (Point3, bool) {
        let edge = (self.points[0].xyz() - axis.location().xyz()) * (1.0 / alpha);
        let time = self.normal.dot(edge);
        let cross = axis.direction().xyz().crossed(edge);
        let u = cross.dot(self.edges[2]);
        let v = cross.dot(self.edges[0]);
        let interior = time >= 0. && u >= 0. && v >= 0. && u + v <= 1.;
        (
            Point3::from(axis.location().xyz() + axis.direction().xyz() * time),
            interior,
        )
    }
    pub fn nearest_edge(&self, point: Point3) -> usize {
        let mut minimum = f64::MAX;
        let mut nearest = 0;
        for index in 0..3 {
            let offset = point.xyz() - self.points[index].xyz();
            let edge = self.edges[index];
            let coefficient = edge.dot(offset) / edge.dot(edge);
            let distance =
                point.distance(&Point3::from(self.points[index].xyz() + edge * coefficient));
            if distance < minimum {
                minimum = distance;
                nearest = index;
            }
        }
        nearest
    }
}
