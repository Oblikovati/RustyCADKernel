//! Closest points on finite view segments, with native far-plane clipping.
use super::rectangular_frustum::RectangularRay;
use super::{SelectMgrBaseIntersector, SelectMgrRectangularFrustum, SelectMgrViewClipRange};
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::{ncollection::Vec3, precision::confusion, standard::Failure};
use rustycad_tkmath::{
    bvh::{BvhRay, std_max, std_min, tools::ray_box_intersection_by_corners},
    gp::{Point3, Vector3, resolution},
};

fn rectangular_segment_parameter(ray: RectangularRay, first: Point3, last: Point3) -> f64 {
    let u = last.xyz() - first.xyz();
    let v = ray.far.xyz() - ray.near.xyz();
    let w = first.xyz() - ray.near.xyz();
    let (a, b, c, d, e) = (u.dot(u), u.dot(v), v.dot(v), u.dot(w), v.dot(w));
    let coefficient = a * c - b * b;
    let (numerator, denominator) = rectangular_segment_fraction((a, b, c, d, e), coefficient);
    let numerator = if numerator < 0. {
        0.
    } else if numerator > denominator {
        denominator
    } else {
        numerator
    };
    if denominator.abs() < resolution() {
        0.
    } else {
        numerator / denominator
    }
}
fn rectangular_segment_fraction(
    (a, b, c, d, e): (f64, f64, f64, f64, f64),
    coefficient: f64,
) -> (f64, f64) {
    if coefficient < resolution() {
        return (e, c);
    }
    let segment = b * e - c * d;
    if segment < 0. {
        return (e, c);
    }
    if segment > coefficient {
        return (e + b, c);
    }
    (a * e - b * d, coefficient)
}
fn rectangular_closest_on_segment(point: Point3, first: Point3, last: Point3) -> Point3 {
    let picked = Vector3::between(first, point);
    let figure = Vector3::between(first, last);
    let (picked_length, figure_length) = (picked.magnitude(), figure.magnitude());
    if picked_length <= resolution() || figure_length <= resolution() {
        return point;
    }
    let cosine = figure.dot(picked) / (picked_length * figure_length);
    let shift = std_min(figure_length, std_max(0., cosine * picked_length));
    Point3::from(first.xyz() + *figure.xyz() * (shift / figure_length))
}
impl SelectMgrRectangularFrustum {
    /// Pick the closest finite view-segment depth; `volume.segment_segment_distance(&a, &b, &mut pick)`.
    pub fn segment_segment_distance(
        &self,
        first: &Point3,
        last: &Point3,
        pick: &mut SelectBasicsPickResult,
    ) {
        let ray = *self.ray.read();
        let parameter = rectangular_segment_parameter(ray, *first, *last);
        let point = Point3::from(ray.near.xyz() + (ray.far.xyz() - ray.near.xyz()) * parameter);
        pick.set_depth(ray.near.distance(&point) * ray.scale);
        pick.set_picked_point(rectangular_closest_on_segment(point, *first, *last));
    }
    /// Intersect the finite view segment with a plane, updating only depth on success.
    /// Example: `volume.segment_plane_intersection(&normal, &origin, &mut pick)`.
    pub fn segment_plane_intersection(
        &self,
        normal: &Vector3,
        point: &Point3,
        pick: &mut SelectBasicsPickResult,
    ) -> bool {
        let ray = *self.ray.read();
        let direction = ray.far.xyz() - ray.near.xyz();
        let denominator = normal.xyz().dot(direction);
        let numerator = -normal.xyz().dot(ray.near.xyz() - point.xyz());
        if denominator.abs() < confusion() {
            pick.invalidate();
            return false;
        }
        let parameter = numerator / denominator;
        if parameter < 0. || parameter > 1. {
            pick.invalidate();
            return false;
        }
        let closest = Point3::from(ray.near.xyz() + direction * parameter);
        pick.set_depth(ray.near.distance(&closest) * ray.scale);
        true
    }
    pub(super) fn rectangular_box_pick(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        if !self.overlaps_box(min, max, None)? {
            return Ok(false);
        }
        let ray = *self.ray.read();
        let bvh_ray = BvhRay::new(
            Vec3::new(ray.near.x(), ray.near.y(), ray.near.z()),
            Vec3::new(ray.direction.x(), ray.direction.y(), ray.direction.z()),
        );
        let Some((entry, leave)) = ray_box_intersection_by_corners(&bvh_ray, min, max) else {
            let point = rectangular_nearest_box_point(ray.near, min, max);
            pick.set_depth(point.distance(&ray.near));
            return Ok(!clip.is_clipped(pick.depth()));
        };
        let Some(depth) = super::axis_quadric::axis_nearest_depth(entry, leave, clip)? else {
            return Ok(false);
        };
        pick.set_depth(depth);
        Ok(true)
    }
}

fn rectangular_nearest_box_point(point: Point3, min: &Vec3<f64>, max: &Vec3<f64>) -> Point3 {
    Point3::new(
        std_max(std_min(point.x(), max.x), min.x),
        std_max(std_min(point.y(), max.y), min.y),
        std_max(std_min(point.z(), max.z), min.z),
    )
}
