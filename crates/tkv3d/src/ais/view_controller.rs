//! `AIS_ViewController`: the queued input state of the 3D view controller.
use super::view_controller_camera_action_state::ViewControllerCameraActionState;
use super::view_controller_clicks::ViewControllerClickDispatch;
use super::view_controller_dispatch_state::{ViewControllerDispatchState, ViewControllerXrState};
use super::view_controller_input_state::{ViewControllerClickState, ViewControllerNavigationState};
use super::view_controller_mouse_state::ViewControllerMouseDispatch;
use super::view_controller_parameters::ViewControllerParameters;
use super::view_controller_rotation_state::ViewControllerRotationState;
use super::view_controller_touch::ViewControllerTouchDispatch;
use super::{AisNavigationMode, AisViewInputBuffer, AisViewInputBufferType};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::AspectWindowInputListenerBase;
/// Queues native viewer gestures; `AisViewController::new()?.update_z_rotation(0.1)`.
pub struct AisViewController {
    pub(super) ui_input: AisViewInputBuffer,
    pub(super) render_input: AisViewInputBuffer,
    pub(super) parameters: ViewControllerParameters,
    pub(super) panning_anchor: Point3,
    pub(super) camera_action_state: ViewControllerCameraActionState,
    pub(super) rotation_state: ViewControllerRotationState,
    pub(super) navigation_mode: AisNavigationMode,
    pub(super) previous_move_to: Vec2<i32>,
    pub(super) input_base: AspectWindowInputListenerBase,
    pub(super) click_state: ViewControllerClickState,
    pub(super) navigation_state: ViewControllerNavigationState,
    pub(super) click_dispatch: ViewControllerClickDispatch,
    pub(super) touch_dispatch: ViewControllerTouchDispatch,
    pub(super) mouse_dispatch: ViewControllerMouseDispatch,
    pub(super) dispatch: ViewControllerDispatchState,
    pub(super) xr: ViewControllerXrState,
}
impl AisViewController {
    /// Borrow the UI or rendering queue; `controller.input_buffer(AisViewInputBufferType::Ui)`.
    pub fn input_buffer(&self, role: AisViewInputBufferType) -> &AisViewInputBuffer {
        match role {
            AisViewInputBufferType::Ui => &self.ui_input,
            AisViewInputBufferType::Gl => &self.render_input,
        }
    }
    /// Mutate one queue independently; `controller.change_input_buffer(AisViewInputBufferType::Ui).reset()`.
    pub fn change_input_buffer(&mut self, role: AisViewInputBufferType) -> &mut AisViewInputBuffer {
        match role {
            AisViewInputBufferType::Ui => &mut self.ui_input,
            AisViewInputBufferType::Gl => &mut self.render_input,
        }
    }
    /// Read the navigation policy; `controller.navigation_mode()`.
    pub fn navigation_mode(&self) -> AisNavigationMode {
        self.navigation_mode
    }
    /// Abort queued UI rotations when choosing navigation, even for the same mode; `controller.set_navigation_mode(AisNavigationMode::Orbit)`.
    pub fn set_navigation_mode(&mut self, mode: AisNavigationMode) {
        self.navigation_mode = mode;
        self.ui_input.orbit_rotation.to_start = false;
        self.ui_input.orbit_rotation.to_rotate = false;
        self.ui_input.view_rotation.to_start = false;
        self.ui_input.view_rotation.to_rotate = false;
    }
    /// Check the native undefined pixel sentinel; `controller.has_previous_move_to()`.
    pub fn has_previous_move_to(&self) -> bool {
        self.previous_move_to != Vec2::new(-1, -1)
    }
    /// Borrow the last dispatched highlighting pixel; `controller.previous_move_to()`.
    pub fn previous_move_to(&self) -> &Vec2<i32> {
        &self.previous_move_to
    }
    /// Restore the native undefined pixel; `controller.reset_previous_move_to()`.
    pub fn reset_previous_move_to(&mut self) {
        self.previous_move_to = Vec2::new(-1, -1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dispatched_highlight_pixel_and_reset_match_native() {
        let mut controller = AisViewController::new().unwrap();
        controller.previous_move_to = Vec2::new(4, 8);
        assert_previous(&controller, "previous-defined");
        controller.reset_previous_move_to();
        assert_previous(&controller, "previous-cleared");
    }
    fn assert_previous(c: &AisViewController, name: &str) {
        let row = include_str!("../../tests/fixtures/view-controller-events-native.txt")
            .lines()
            .find(|row| row.starts_with(&format!("{name} ")))
            .unwrap();
        let expected: Vec<i32> = row
            .split_whitespace()
            .skip(1)
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(
            expected,
            [
                c.has_previous_move_to() as i32,
                c.previous_move_to().x,
                c.previous_move_to().y
            ]
        );
    }
}
