//! Walking projects forward without normalization and recomputes pitch side after yaw.
use super::{
    AisNavigationMode, AisViewController, AisWalkDelta, AisWalkPart, AisWalkRotation as R,
    AisWalkTranslation as T,
};
use crate::v3d::V3dView;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3, Vector3, Xyz};
use rustycad_tkservice::{aspect::AspectScrollDelta, graphic3d::Graphic3dCamera};
impl AisViewController {
    pub(super) fn translate_navigation_walk(
        &mut self,
        walk: &AisWalkDelta,
        view: &V3dView,
        camera: &Graphic3dCamera,
        up: Direction3,
        speed: f64,
    ) -> Result<(), Failure> {
        if !walk.to_move() || !self.to_allow_panning() {
            return Ok(());
        }
        let displacement = navigation_displacement(camera, walk, up, speed)?;
        self.zoom_navigation_forward(walk, view, camera)?;
        translate_navigation_camera(camera, displacement);
        Ok(())
    }
    fn zoom_navigation_forward(
        &mut self,
        walk: &AisWalkDelta,
        view: &V3dView,
        camera: &Graphic3dCamera,
    ) -> Result<(), Failure> {
        let forward = walk[T::Forward];
        if !camera.is_orthographic() || forward.is_empty() {
            return Ok(());
        }
        // This calls actual view zoom before translating the selected (possibly XR base) camera.
        let delta = forward.value * forward.pressure * f64::from(self.walk_speed_relative());
        self.handle_zoom(view, &AspectScrollDelta::with_delta(delta * 100.0, 0), None)
    }
    pub(super) fn rotate_navigation_walk(
        &self,
        walk: &AisWalkDelta,
        camera: &Graphic3dCamera,
        up: Direction3,
    ) -> Result<(), Failure> {
        if self.navigation_mode() != AisNavigationMode::Orbit || !self.to_allow_rotation() {
            return Ok(());
        }
        rotate_navigation_component(camera, camera.eye(), up, walk[R::Yaw], 0.5)?;
        if !walk[R::Pitch].is_empty() {
            let side = camera.side_right()?.reversed();
            rotate_navigation_component(camera, camera.eye(), side, walk[R::Pitch], -0.5)?;
        }
        self.roll_navigation_walk(walk, camera)
    }
    fn roll_navigation_walk(
        &self,
        walk: &AisWalkDelta,
        camera: &Graphic3dCamera,
    ) -> Result<(), Failure> {
        if !self.to_lock_orbit_z_up() {
            let axis = Axis1::new(camera.center(), camera.direction());
            rotate_navigation_component(
                camera,
                axis.location(),
                axis.direction(),
                walk[R::Roll],
                0.5,
            )?;
        }
        Ok(())
    }
}
fn navigation_component(axis: Xyz, part: AisWalkPart, speed: f64) -> Xyz {
    if part.is_empty() {
        return Xyz::default();
    }
    axis * part.value * part.pressure * speed
}
fn translate_navigation_camera(camera: &Graphic3dCamera, displacement: Xyz) {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(
        displacement.x(),
        displacement.y(),
        displacement.z(),
    ));
    camera.transform(&transform);
}
fn rotate_navigation_component(
    camera: &Graphic3dCamera,
    origin: Point3,
    direction: Direction3,
    part: AisWalkPart,
    speed: f64,
) -> Result<(), Failure> {
    if part.is_empty() {
        return Ok(());
    }
    // Native rotation intentionally ignores pressure; translation uses it.
    let mut transform = Transform3::default();
    transform.set_rotation(Axis1::new(origin, direction), part.value * speed)?;
    camera.transform(&transform);
    Ok(())
}

fn navigation_displacement(
    camera: &Graphic3dCamera,
    walk: &AisWalkDelta,
    up: Direction3,
    speed: f64,
) -> Result<Xyz, Failure> {
    let side = camera.side_right()?.reversed();
    let forward = camera.direction().xyz();
    let forward = forward - up.xyz() * up.xyz().dot(forward);
    let mut displacement = Xyz::default();
    if !camera.is_orthographic() {
        displacement += navigation_component(forward, walk[T::Forward], speed);
    }
    displacement += navigation_component(side.xyz(), walk[T::Side], speed);
    displacement += navigation_component(up.xyz(), walk[T::Up], speed);
    Ok(displacement)
}
