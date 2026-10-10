//! Native local-coordinate quadric picks; depths deliberately remain in local units.
use super::{SelectMgrAxisIntersector, SelectMgrBaseIntersector, SelectMgrViewClipRange};
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::{precision::confusion, standard::Failure};
use rustycad_tkmath::{
    bnd::Range1,
    bvh::std_max,
    gp::{Axis1, Direction3, Point3, Transform3, Vector3},
};

pub(super) fn axis_inside_entry(hit: Option<(f64, f64)>, inside: Option<&mut bool>) -> bool {
    let Some((entry, _)) = hit else {
        return false;
    };
    if let Some(inside) = inside {
        *inside &= entry >= 0.;
    }
    true
}
pub(super) fn axis_nearest_depth(
    entry: f64,
    leave: f64,
    clip: &SelectMgrViewClipRange,
) -> Result<Option<f64>, Failure> {
    // Reversed native Bnd_Range construction raises; it must not become an empty range.
    let range = Range1::new(std_max(entry, 0.), leave)?;
    let mut depth = 0.;
    range.get_min(&mut depth);
    Ok(clip
        .get_nearest_depth_into(&range, &mut depth)
        .then_some(depth))
}
pub(super) fn axis_local_ray(axis: Axis1, transform: Transform3) -> Result<Axis1, Failure> {
    let inverse = transform.inverted()?;
    Ok(Axis1::new(
        axis.location().transformed(inverse),
        axis.direction().transformed(inverse),
    ))
}
fn axis_local_point(axis: Axis1, depth: f64) -> Point3 {
    Point3::from(axis.location().xyz() + axis.direction().xyz() * depth)
}
pub(super) fn axis_quadric_normal(
    point: Point3,
    top: Option<f64>,
    transform: Transform3,
) -> Vector3 {
    let normal = Direction3::Z.transformed(transform);
    if point.z().abs() < confusion() {
        return Vector3::from(normal.xyz().reversed());
    }
    if top.is_some_and(|height| (point.z() - height).abs() < confusion()) {
        return Vector3::from(normal.xyz());
    }
    Vector3::new(point.x(), point.y(), 0.).transformed(transform)
}
fn axis_store_quadric_pick(
    ray: Axis1,
    depth: f64,
    transform: Transform3,
    top: Option<f64>,
    pick: &mut SelectBasicsPickResult,
) {
    let point = axis_local_point(ray, depth);
    pick.set_depth(depth);
    pick.set_picked_point(point.transformed(transform));
    pick.set_surface_normal_vec(&axis_quadric_normal(point, top, transform));
}
impl SelectMgrAxisIntersector {
    pub(super) fn axis_sphere_pick(
        &self,
        center: &Point3,
        radius: f64,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let axis = self.initialized_axis("OverlapsSphere")?;
        let Some((entry, leave)) =
            self.ray_sphere_intersection(center, radius, &axis.location(), &axis.direction())
        else {
            return Ok(false);
        };
        let Some(depth) = axis_nearest_depth(entry, leave, clip)? else {
            return Ok(false);
        };
        let point = axis_local_point(axis, depth);
        pick.set_depth(depth);
        pick.set_picked_point(point);
        pick.set_surface_normal_vec(&Vector3::from(point.xyz() - center.xyz()));
        Ok(true)
    }
    pub(super) fn axis_cylinder_times(
        &self,
        radii: (f64, f64),
        height: f64,
        transform: Transform3,
        hollow: bool,
    ) -> Result<(Axis1, Option<(f64, f64)>), Failure> {
        let axis = self.initialized_axis("OverlapsCylinder")?;
        let ray = axis_local_ray(axis, transform)?;
        let hit = self.ray_cylinder_intersection(
            radii.0,
            radii.1,
            height,
            &ray.location(),
            &ray.direction(),
            hollow,
        )?;
        // Native initializes the leave output to zero even when only entry is written.
        Ok((ray, hit.map(|(entry, leave)| (entry, leave.unwrap_or(0.)))))
    }
    pub(super) fn axis_cylinder_pick(
        &self,
        radii: (f64, f64),
        height: f64,
        transform: Transform3,
        hollow: bool,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let (ray, hit) = self.axis_cylinder_times(radii, height, transform, hollow)?;
        let Some((entry, leave)) = hit else {
            return Ok(false);
        };
        let Some(depth) = axis_nearest_depth(entry, std_max(entry, leave), clip)? else {
            return Ok(false);
        };
        axis_store_quadric_pick(ray, depth, transform, Some(height), pick);
        Ok(true)
    }
    pub(super) fn axis_circle_time(
        &self,
        radius: f64,
        transform: Transform3,
        filled: bool,
    ) -> Result<(Axis1, Option<f64>), Failure> {
        let axis = self.initialized_axis("OverlapsCircle")?;
        let ray = axis_local_ray(axis, transform)?;
        Ok((
            ray,
            self.ray_circle_intersection(radius, &ray.location(), &ray.direction(), filled),
        ))
    }
    pub(super) fn axis_circle_pick(
        &self,
        radius: f64,
        transform: Transform3,
        filled: bool,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let (ray, time) = self.axis_circle_time(radius, transform, filled)?;
        let Some(time) = time else {
            return Ok(false);
        };
        let depth = std_max(time, 0.);
        if clip.is_clipped(depth) {
            return Ok(false);
        }
        axis_store_quadric_pick(ray, depth, transform, None, pick);
        Ok(true)
    }
}
