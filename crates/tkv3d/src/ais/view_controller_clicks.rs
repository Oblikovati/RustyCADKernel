//! Martensite click dispatch preserves animation ownership and native selection bindings.
use super::{
    AisAnimation, AisAnimationCamera, AisAnimationMembers, AisAnimationSource, AisSelectionScheme,
    AisViewController,
};
use rustycad_tkernel::{
    ncollection::{DataMap, Vec2},
    standard::{Failure, Handle},
};
use rustycad_tkservice::aspect::{
    ASPECT_VKEY_FLAGS_ALT, ASPECT_VKEY_FLAGS_SHIFT, ASPECT_VKEY_MOUSE_LEFT_BUTTON, AspectVKeyFlags,
    AspectVKeyMouse,
};

pub(super) struct ViewControllerClickDispatch {
    view_animation: Handle<AisAnimationCamera>,
    objects_animation: Handle<dyn AisAnimationSource>,
    selection_schemes: DataMap<u32, AisSelectionScheme>,
}
impl ViewControllerClickDispatch {
    pub(super) fn new() -> Self {
        let view_animation = Handle::new(AisAnimationCamera::new(
            "AIS_ViewController_ViewAnimation",
            &Handle::default(),
        ));
        view_animation.set_own_duration(0.5);
        Self {
            view_animation,
            objects_animation: Handle::new(AisAnimation::new(
                "AIS_ViewController_ObjectsAnimation",
            ))
            .cast::<dyn AisAnimationSource>(),
            selection_schemes: native_click_selection_schemes(),
        }
    }
}
fn native_click_selection_schemes() -> DataMap<u32, AisSelectionScheme> {
    let mut schemes = DataMap::default();
    for modifiers in [
        0,
        ASPECT_VKEY_FLAGS_ALT,
        ASPECT_VKEY_FLAGS_SHIFT,
        ASPECT_VKEY_FLAGS_ALT | ASPECT_VKEY_FLAGS_SHIFT,
    ] {
        let scheme = if modifiers & ASPECT_VKEY_FLAGS_SHIFT != 0 {
            AisSelectionScheme::Xor
        } else {
            AisSelectionScheme::Replace
        };
        schemes.bind(ASPECT_VKEY_MOUSE_LEFT_BUTTON | modifiers, scheme);
    }
    schemes
}
impl AisViewController {
    /// Borrow the retained camera animation; `controller.view_animation().is_null()`.
    pub fn view_animation(&self) -> &Handle<AisAnimationCamera> {
        &self.click_dispatch.view_animation
    }
    /// Retain or detach a camera animation by identity; `controller.set_view_animation(&animation)`.
    pub fn set_view_animation(&mut self, animation: &Handle<AisAnimationCamera>) {
        self.click_dispatch.view_animation = animation.clone();
    }
    /// Stop and detach only a running camera animation; `controller.abort_view_animation()?`.
    pub fn abort_view_animation(&mut self) -> Result<(), Failure> {
        let Some(animation) = self.view_animation().get() else {
            return Ok(());
        };
        if animation.is_stopped() {
            return Ok(());
        }
        animation.stop()?;
        animation.set_view(&Handle::default());
        Ok(())
    }
    /// Borrow the shared object timeline; `controller.objects_animation().is_null()`.
    pub fn objects_animation(&self) -> &Handle<dyn AisAnimationSource> {
        &self.click_dispatch.objects_animation
    }
    /// Retain a polymorphic timeline or null; `controller.set_objects_animation(&timeline)`.
    pub fn set_objects_animation(&mut self, animation: &Handle<dyn AisAnimationSource>) {
        self.click_dispatch.objects_animation = animation.clone();
    }
    /// Borrow button/modifier selection bindings; `controller.mouse_selection_schemes().extent()`.
    pub fn mouse_selection_schemes(&self) -> &DataMap<u32, AisSelectionScheme> {
        &self.click_dispatch.selection_schemes
    }
    /// Edit the live selection bindings; `controller.change_mouse_selection_schemes().clear(false)`.
    pub fn change_mouse_selection_schemes(&mut self) -> &mut DataMap<u32, AisSelectionScheme> {
        &mut self.click_dispatch.selection_schemes
    }
    /// Pause an active timeline before dispatching a bound pick; `controller.update_mouse_click(point, button, flags, false)?`.
    pub fn update_mouse_click(
        &mut self,
        point: Vec2<i32>,
        button: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        _is_double_click: bool,
    ) -> Result<bool, Failure> {
        self.pause_objects_before_click()?;
        let Some(scheme) = self
            .mouse_selection_schemes()
            .seek(&(button | modifiers))
            .copied()
        else {
            return Ok(false);
        };
        self.select_in_viewer(point, scheme);
        Ok(true)
    }
    fn pause_objects_before_click(&self) -> Result<(), Failure> {
        if !self.to_pause_objects_animation() {
            return Ok(());
        }
        let Some(animation) = self.objects_animation().get() else {
            return Ok(());
        };
        if animation.is_stopped() {
            return Ok(());
        }
        animation.pause()
    }
}
