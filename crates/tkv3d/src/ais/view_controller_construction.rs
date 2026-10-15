//! Fallible ownership construction preserves native presentation initialization failures.
use super::view_controller_camera_action_state::ViewControllerCameraActionState;
use super::view_controller_clicks::ViewControllerClickDispatch;
use super::view_controller_dispatch_state::{ViewControllerDispatchState, ViewControllerXrState};
use super::view_controller_input_state::{ViewControllerClickState, ViewControllerNavigationState};
use super::view_controller_mouse_state::ViewControllerMouseDispatch;
use super::view_controller_parameters::ViewControllerParameters;
use super::view_controller_rotation_state::ViewControllerRotationState;
use super::view_controller_touch::ViewControllerTouchDispatch;
use super::{AisNavigationMode, AisViewController, AisViewInputBuffer};
use rustycad_tkernel::{ncollection::Vec2, osd::Timer, standard::Failure};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::AspectWindowInputListenerBase;
struct ControllerOwnedParts {
    clicks: ViewControllerClickState,
    touches: ViewControllerTouchDispatch,
    input: AspectWindowInputListenerBase,
    actions: ViewControllerCameraActionState,
    dispatch: ViewControllerDispatchState,
}
impl ControllerOwnedParts {
    fn new(timer: Timer) -> Result<Self, Failure> {
        let clicks = ViewControllerClickState::with_timer(timer.clone());
        let touches = ViewControllerTouchDispatch::with_timer(timer.clone());
        let input = AspectWindowInputListenerBase::with_timer(timer);
        let actions = ViewControllerCameraActionState::new()?;
        let dispatch = ViewControllerDispatchState::new()?;
        Ok(Self {
            clicks,
            touches,
            input,
            actions,
            dispatch,
        })
    }
}
impl AisViewController {
    /// Own native input and real anchor presentations; `let controller=AisViewController::new()?`.
    pub fn new() -> Result<Self, Failure> {
        // Native AIS_Point construction can fail; callers must receive that failure.
        Self::with_event_timer(Timer::new(false))
    }
    /// Inject event clocks while constructing real anchors; `AisViewController::with_event_timer(timer)?`.
    pub fn with_event_timer(timer: Timer) -> Result<Self, Failure> {
        let parts = ControllerOwnedParts::new(timer)?;
        Ok(Self::from_owned_parts(parts))
    }
    fn from_owned_parts(parts: ControllerOwnedParts) -> Self {
        Self {
            ui_input: AisViewInputBuffer::new(),
            render_input: AisViewInputBuffer::new(),
            parameters: ViewControllerParameters::default(),
            navigation_mode: AisNavigationMode::Orbit,
            rotation_state: ViewControllerRotationState::default(),
            camera_action_state: parts.actions,
            panning_anchor: Point3::new(rustycad_tkernel::precision::infinite(), 0.0, 0.0),
            previous_move_to: Vec2::new(-1, -1),
            input_base: parts.input,
            click_state: parts.clicks,
            navigation_state: ViewControllerNavigationState::default(),
            click_dispatch: ViewControllerClickDispatch::new(),
            touch_dispatch: parts.touches,
            mouse_dispatch: ViewControllerMouseDispatch::default(),
            dispatch: parts.dispatch,
            xr: ViewControllerXrState::default(),
        }
    }
}
