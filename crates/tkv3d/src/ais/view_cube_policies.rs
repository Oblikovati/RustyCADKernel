//! Native widget policy setters keep animation identity and affect subsequent clicks.
use super::{AisAnimationCamera, AisAnimationMembers, AisViewCubeSource};
use rustycad_tkernel::standard::{Failure, Handle};
/// Nonvirtual native ViewCube policy members; `cube.set_fixed_animation_loop(false)`.
pub trait AisViewCubeMembers: AisViewCubeSource {
    /// Borrow the native live handle slot; `let alias=cube.view_animation_ref()`.
    fn view_animation_ref(&self) -> super::AisViewCubeAnimationRef<'_> {
        super::AisViewCubeAnimationRef::new(&self.interaction_base().fields)
    }
    /// Copy the current animation handle; replacements do not alter this snapshot.
    /// Example: `let owned=cube.view_animation()`; native ViewAnimation parity uses `view_animation_ref`.
    fn view_animation(&self) -> Handle<AisAnimationCamera> {
        self.interaction_base().fields.read().animation.clone()
    }
    /// Replace the retained native camera animation; `cube.set_view_animation(&animation)`.
    fn set_view_animation(&self, animation: &Handle<AisAnimationCamera>) {
        self.interaction_base().fields.write().animation = animation.clone();
    }
    /// Read own duration or diagnose a null animation; `cube.duration()?`.
    fn duration(&self) -> Result<f64, Failure> {
        let animation = self.view_animation();
        Ok(super::view_cube_animation::required_cube_animation(&animation)?.own_duration())
    }
    /// Set own duration without normalization, matching native; `cube.set_duration(0.5)?`.
    fn set_duration(&self, seconds: f64) -> Result<(), Failure> {
        let animation = self.view_animation();
        super::view_cube_animation::required_cube_animation(&animation)?.set_own_duration(seconds);
        Ok(())
    }
    /// Paused/stopped animations are finished; `cube.has_animation()?`.
    fn has_animation(&self) -> Result<bool, Failure> {
        let animation = self.view_animation();
        Ok(!super::view_cube_animation::required_cube_animation(&animation)?.is_stopped())
    }
    /// Read automatic owner-click behavior; `cube.to_auto_start_animation()`.
    fn to_auto_start_animation(&self) -> bool {
        self.interaction_base().fields.read().auto_start
    }
    /// Enable automatic owner clicks; `cube.set_auto_start_animation(false)`.
    fn set_auto_start_animation(&self, enabled: bool) {
        self.interaction_base().fields.write().auto_start = enabled;
    }
    /// Read the native uninterruptible-loop policy; `cube.is_fixed_animation_loop()`.
    fn is_fixed_animation_loop(&self) -> bool {
        self.interaction_base().fields.read().fixed_loop
    }
    /// Select application-driven stepping or fixed loop; `cube.set_fixed_animation_loop(false)`.
    fn set_fixed_animation_loop(&self, enabled: bool) {
        self.interaction_base().fields.write().fixed_loop = enabled;
    }
    /// Read the camera-up reset policy; `cube.to_reset_camera_up()`.
    fn to_reset_camera_up(&self) -> bool {
        self.interaction_base().fields.read().reset_up
    }
    /// Force the default view up direction on subsequent clicks; `cube.set_reset_camera(true)`.
    fn set_reset_camera(&self, reset: bool) {
        self.interaction_base().fields.write().reset_up = reset;
    }
    /// Read selected-first fit policy; `cube.to_fit_selected()`.
    fn to_fit_selected(&self) -> bool {
        self.interaction_base().fields.read().fit_selected
    }
    /// Use selected or displayed scene bounds; `cube.set_fit_selected(false)`.
    fn set_fit_selected(&self, selected: bool) {
        self.interaction_base().fields.write().fit_selected = selected;
    }
}
impl<C: AisViewCubeSource + ?Sized> AisViewCubeMembers for C {}
