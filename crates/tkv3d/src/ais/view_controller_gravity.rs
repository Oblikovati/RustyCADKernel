//! Martensite orbit pivots preserve native picking and scene-bound fallback policy.
use super::{AisInteractiveContext, AisRotationMode, AisViewController};
use crate::v3d::V3dView;
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Failure, exceptions::null_object},
};
use rustycad_tkmath::gp::Point3;

impl AisInteractiveContext {
    /// Obtain the view's visible-structure gravity point; `context.gravity_point(&view)?`.
    pub fn gravity_point(&self, view: &V3dView) -> Result<Point3, Failure> {
        view.gravity_point()
    }
}

impl AisViewController {
    /// Resolve the configured orbit pivot with native context fallback; `controller.gravity_point(&context, &view)?`.
    pub fn gravity_point(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
    ) -> Result<Point3, Failure> {
        if let Some(point) = self.controller_rotation_pivot(context, view)? {
            return Ok(point);
        }
        context.gravity_point(view)
    }

    fn controller_rotation_pivot(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
    ) -> Result<Option<Point3>, Failure> {
        match self.rotation_mode() {
            AisRotationMode::PickLast | AisRotationMode::PickCenter => {
                return self.controller_pick_pivot(context, view);
            }
            AisRotationMode::CameraAt => return Ok(Some(view.camera().center())),
            AisRotationMode::BndBoxScene => return controller_scene_pivot(view),
            AisRotationMode::BndBoxActive => {}
        }
        Ok(None)
    }

    fn controller_pick_pivot(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
    ) -> Result<Option<Point3>, Failure> {
        let cursor = self.controller_gravity_cursor(view)?;
        let mut point = Point3::default();
        let stick = self.to_stick_to_ray_on_rotation();
        let hit = self.pick_point(&mut point, context, view, &cursor, stick)?;
        Ok(hit.then_some(point))
    }

    fn controller_gravity_cursor(&self, view: &V3dView) -> Result<Vec2<i32>, Failure> {
        if self.rotation_mode() != AisRotationMode::PickCenter {
            let start = self.render_input.orbit_rotation.point_start;
            return Ok(Vec2::new(start.x() as i32, start.y() as i32));
        }
        let window = view.window();
        let window = window.get().ok_or_else(|| {
            null_object("GravityPoint window=null; expected an attached view window")
        })?;
        let (width, height) = window.size();
        Ok(Vec2::new(width / 2, height / 2))
    }
}

fn controller_scene_pivot(view: &V3dView) -> Result<Option<Point3>, Failure> {
    let bounds = view.view().min_max_values(false)?;
    if bounds.is_void() {
        return Ok(None);
    }
    let center = (bounds.corner_min()?.xyz() + bounds.corner_max()?.xyz()) * 0.5;
    Ok(Some(Point3::from(center)))
}
