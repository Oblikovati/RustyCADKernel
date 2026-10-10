//! Native protected ray/linear intersection helpers (OCCT 3d097a0).
use super::SelectMgrAxisIntersector;
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::{
    ncollection::Vec3,
    precision::{angular, confusion},
    standard::Failure,
};
use rustycad_tkmath::{
    bvh::{BvhRay, tools::ray_box_intersection_by_corners},
    gp::{Axis1, Direction3, Point3, Vector3, Xyz},
};

fn axis_vector(point: Xyz) -> Vec3<f64> {
    Vec3::new(point.x(), point.y(), point.z())
}
fn axis_segment_point(axis: Axis1, first: Point3, second: Point3) -> Option<Point3> {
    let segment = second.xyz() - first.xyz();
    let ray = axis.direction().xyz();
    let offset = first.xyz() - axis.location().xyz();
    let ray_cross = ray.crossed(segment);
    let offset_cross = offset.crossed(segment);
    if ray_cross.modulus() <= confusion() || offset_cross.modulus() <= confusion() {
        return None;
    }
    let parameter = offset_cross.dot(ray_cross) / ray_cross.square_modulus();
    if parameter < 0. {
        return None;
    }
    let point = axis.location().xyz() + ray * parameter;
    if (point - first.xyz()).modulus() + (point - second.xyz()).modulus()
        > segment.modulus() + confusion()
    {
        return None;
    }
    Some(Point3::from(point))
}
fn axis_plane_point(axis: Axis1, normal: Xyz, point: Point3) -> Option<Point3> {
    let direction = axis.direction().xyz();
    let offset = axis.location().xyz() - point.xyz();
    let denominator = normal.dot(direction);
    let numerator = -normal.dot(offset);
    if denominator.abs() < confusion() {
        return None;
    }
    let parameter = numerator / denominator;
    if parameter < 0. {
        return None;
    }
    Some(Point3::from(axis.location().xyz() + direction * parameter))
}
fn axis_store_linear_pick(
    axis: Axis1,
    point: Option<Point3>,
    pick: &mut SelectBasicsPickResult,
) -> bool {
    let Some(point) = point else {
        pick.invalidate();
        return false;
    };
    pick.set_depth(axis.location().distance(&point));
    pick.set_picked_point(point);
    true
}
impl SelectMgrAxisIntersector {
    /// Protected slab test without Init assertions; untouched outputs are None.
    /// Example: `volume.has_intersection_box(&minimum, &maximum)`.
    pub fn has_intersection_box(
        &self,
        minimum: &Vec3<f64>,
        maximum: &Vec3<f64>,
    ) -> Option<(f64, f64)> {
        let axis = *self.axis.read();
        let ray = BvhRay::new(
            axis_vector(axis.location().xyz()),
            axis_vector(axis.direction().xyz()),
        );
        ray_box_intersection_by_corners(&ray, minimum, maximum)
    }
    /// Return point depth only for a co-directed ray; the origin raises a direction failure.
    /// Example: `volume.has_intersection_point(&point)?`.
    pub fn has_intersection_point(&self, point: &Point3) -> Result<Option<f64>, Failure> {
        let axis = *self.axis.read();
        let direction = Direction3::from_xyz(point.xyz() - axis.location().xyz())?;
        Ok(axis
            .direction()
            .is_equal(&direction, angular())
            .then(|| axis.location().distance(point)))
    }
    /// Intersect a segment, invalidating the pick on rejection; `volume.ray_segment_distance(&a, &b, &mut pick)`.
    pub fn ray_segment_distance(
        &self,
        first: &Point3,
        second: &Point3,
        pick: &mut SelectBasicsPickResult,
    ) -> bool {
        let axis = *self.axis.read();
        axis_store_linear_pick(axis, axis_segment_point(axis, *first, *second), pick)
    }
    /// Intersect a forward plane, invalidating the pick on rejection; `volume.ray_plane_intersection(&normal, &point, &mut pick)`.
    pub fn ray_plane_intersection(
        &self,
        normal: &Vector3,
        point: &Point3,
        pick: &mut SelectBasicsPickResult,
    ) -> bool {
        let axis = *self.axis.read();
        axis_store_linear_pick(axis, axis_plane_point(axis, *normal.xyz(), *point), pick)
    }
}
