//! Rectangular quadric picks preserve scaled depths and writes after clipped hits.
use super::axis_quadric::{axis_local_ray, axis_quadric_normal};
use super::*;
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis1, Point3, Transform3, Vector3};
impl SelectMgrRectangularFrustum {
    pub(super) fn rectangular_sphere_pick(
        &self,
        center: &Point3,
        radius: f64,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.rectangular_initialized("OverlapsSphere")?;
        let ray = *self.ray.read();
        let Some((entry, leave)) =
            self.ray_sphere_intersection(center, radius, &ray.near, &ray.direction)
        else {
            return Ok(false);
        };
        pick.set_depth(entry * ray.scale);
        if clip.is_clipped(pick.depth()) {
            pick.set_depth(leave * ray.scale);
        }
        let point = Point3::from(ray.near.xyz() + ray.direction.xyz() * pick.depth() / ray.scale);
        pick.set_picked_point(point);
        pick.set_surface_normal_vec(&Vector3::from(point.xyz() - center.xyz()));
        Ok(!clip.is_clipped(pick.depth()))
    }
    pub(super) fn rectangular_cylinder_pick(
        &self,
        radii: (f64, f64),
        height: f64,
        transform: Transform3,
        hollow: bool,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.rectangular_initialized("OverlapsCylinder")?;
        let state = *self.ray.read();
        let local = axis_local_ray(Axis1::new(state.near, state.direction), transform)?;
        let Some((entry, leave)) = self.ray_cylinder_intersection(
            radii.0,
            radii.1,
            height,
            &local.location(),
            &local.direction(),
            hollow,
        )?
        else {
            return Ok(false);
        };
        let time = rectangular_clipped_time(entry, leave.unwrap_or(0.), state.scale, clip, pick);
        rectangular_store_quadric(local, time, transform, Some(height), pick);
        Ok(!clip.is_clipped(pick.depth()))
    }
    pub(super) fn rectangular_circle_pick(
        &self,
        radius: f64,
        transform: Transform3,
        filled: bool,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.rectangular_initialized("OverlapsCircle")?;
        let state = *self.ray.read();
        let local = axis_local_ray(Axis1::new(state.near, state.direction), transform)?;
        let Some(time) = self.rectangular_circle_time(radius, transform, filled, local)? else {
            return Ok(false);
        };
        pick.set_depth(time * state.scale);
        rectangular_store_quadric(local, time, transform, None, pick);
        Ok(!clip.is_clipped(pick.depth()))
    }
    fn rectangular_circle_time(
        &self,
        radius: f64,
        transform: Transform3,
        filled: bool,
        local: Axis1,
    ) -> Result<Option<f64>, Failure> {
        if filled {
            return Ok(self.ray_circle_intersection(
                radius,
                &local.location(),
                &local.direction(),
                true,
            ));
        }
        if !self.overlaps_circle(radius, &transform, false, None)? {
            return Ok(None);
        }
        Ok(Some(if local.direction().z() != 0. {
            (0. - local.location().z()) / local.direction().z()
        } else {
            0.
        }))
    }
}
fn rectangular_store_quadric(
    ray: Axis1,
    depth: f64,
    transform: Transform3,
    top: Option<f64>,
    pick: &mut SelectBasicsPickResult,
) {
    let point = Point3::from(ray.location().xyz() + ray.direction().xyz() * depth);
    pick.set_surface_normal_vec(&axis_quadric_normal(point, top, transform));
    pick.set_picked_point(point.transformed(transform));
}

fn rectangular_clipped_time(
    entry: f64,
    leave: f64,
    scale: f64,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> f64 {
    pick.set_depth(entry * scale);
    if !clip.is_clipped(pick.depth()) {
        return entry;
    }
    pick.set_depth(leave * scale);
    leave
}
