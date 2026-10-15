//! Read access to the protected state of `AIS_ViewController` that subclasses inspect: the
//! anchor presentations, the rotation start camera, the mouse and touch bookkeeping and the
//! gesture thresholds.
use super::{AisMouseGesture, AisPoint, AisViewController};
use rustycad_tkernel::ncollection::{DataMap, Vec2};
use rustycad_tkernel::osd::Timer;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
use rustycad_tkservice::aspect::AspectTouch;

/// `myScrollZoomRatio`: the zoom of one mouse scroll step.
pub(super) const SCROLL_ZOOM_RATIO: f32 = 15.0;

impl AisViewController {
    /// `myAnchorPointPrs1`: the rotation center presentation (top layer); `c.anchor_point_prs1()`.
    pub fn anchor_point_prs1(&self) -> &Handle<AisPoint> {
        &self.camera_action_state.rotate_anchor
    }
    /// `myAnchorPointPrs2`: the panning anchor presentation (topmost layer); `c.anchor_point_prs2()`.
    pub fn anchor_point_prs2(&self) -> &Handle<AisPoint> {
        &self.camera_action_state.pan_anchor
    }
    /// `myRotatePnt3d`: the rotation center of the current gesture; `c.rotate_pnt_3d()`.
    pub fn rotate_pnt_3d(&self) -> Point3 {
        self.rotation_state.pivot
    }
    /// `myCamStartOpUp`: the camera up direction at the rotation start; `c.cam_start_op_up()`.
    pub fn cam_start_op_up(&self) -> Direction3 {
        self.rotation_state.start_up
    }
    /// `myCamStartOpDir`: the camera direction at the rotation start; `c.cam_start_op_dir()`.
    pub fn cam_start_op_dir(&self) -> Direction3 {
        self.rotation_state.start_direction
    }
    /// `myCamStartOpEye`: the camera eye at the rotation start; `c.cam_start_op_eye()`.
    pub fn cam_start_op_eye(&self) -> Point3 {
        self.rotation_state.start_eye
    }
    /// `myCamStartOpCenter`: the camera center at the rotation start; `c.cam_start_op_center()`.
    pub fn cam_start_op_center(&self) -> Point3 {
        self.rotation_state.start_center
    }
    /// `myCamStartOpToCenter`: declared but never assigned natively (the null vector).
    /// Example: `c.cam_start_op_to_center()`.
    pub fn cam_start_op_to_center(&self) -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }
    /// `myCamStartOpToEye`: declared but never assigned natively (the null vector).
    /// Example: `c.cam_start_op_to_eye()`.
    pub fn cam_start_op_to_eye(&self) -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }
    /// `myCurrentRollAngle`: the roll of the first-person rotation; `c.current_roll_angle()`.
    pub fn current_roll_angle(&self) -> f64 {
        self.rotation_state.current_roll
    }
    /// `myHasHlrOnBeforeRotation`: the computed mode suspended by a rotation.
    /// Example: `c.has_hlr_on_before_rotation()`.
    pub fn has_hlr_on_before_rotation(&self) -> bool {
        self.camera_action_state.had_computed_hlr
    }
    /// `myHasThrust`: a walk thrust is active; `c.has_thrust()`.
    pub fn has_thrust(&self) -> bool {
        self.navigation_state.has_thrust
    }
    /// `myLastEventsTime`: the time of the last processed events; `c.last_events_time()`.
    pub fn last_events_time(&self) -> f64 {
        self.navigation_state.last_events_time
    }
    /// `myMouseActiveGesture`; `c.mouse_active_gesture()`.
    pub fn mouse_active_gesture(&self) -> AisMouseGesture {
        self.click_state.active_gesture
    }
    /// `myMouseActiveIdleRotation`; `c.mouse_active_idle_rotation()`.
    pub fn mouse_active_idle_rotation(&self) -> bool {
        self.mouse_dispatch.idle_rotation
    }
    /// `myMouseClickCounter`: the clicks of the current multi-click; `c.mouse_click_counter()`.
    pub fn mouse_click_counter(&self) -> i32 {
        self.click_state.counter
    }
    /// `myMouseClickThreshold`: the pixel tolerance of a click; `c.mouse_click_threshold()`.
    pub fn mouse_click_threshold(&self) -> f64 {
        self.mouse_dispatch.click_threshold
    }
    /// `myMouseClickTimer`: the timer of multi-clicks; `c.mouse_click_timer()`.
    pub fn mouse_click_timer(&self) -> &Timer {
        &self.click_state.timer
    }
    /// `myMouseGestureMapDrag`: the gestures of dragging buttons; `c.mouse_gesture_map_drag()`.
    pub fn mouse_gesture_map_drag(&self) -> &DataMap<u32, AisMouseGesture> {
        &self.mouse_dispatch.drag_map
    }
    /// `myMousePressPoint`; `c.mouse_press_point()`.
    pub fn mouse_press_point(&self) -> Vec2<i32> {
        self.mouse_dispatch.press_point
    }
    /// `myMouseProgressPoint`; `c.mouse_progress_point()`.
    pub fn mouse_progress_point(&self) -> Vec2<i32> {
        self.mouse_dispatch.progress_point
    }
    /// `myMouseSingleButton`: the single pressed button (-1 when none); `c.mouse_single_button()`.
    pub fn mouse_single_button(&self) -> i32 {
        self.click_state.single_button
    }
    /// `myMouseStopDragOnUnclick`; `c.mouse_stop_drag_on_unclick()`.
    pub fn mouse_stop_drag_on_unclick(&self) -> bool {
        self.mouse_dispatch.stop_drag_on_unclick
    }
    /// `myNbTouchesLast`: the touches of the previous frame; `c.nb_touches_last()`.
    pub fn nb_touches_last(&self) -> usize {
        self.touch_dispatch.last_touch_count
    }
    /// `myScrollZoomRatio`; `c.scroll_zoom_ratio()`.
    pub fn scroll_zoom_ratio(&self) -> f32 {
        SCROLL_ZOOM_RATIO
    }
    /// `myStartPanCoord`: the pan start of a touch gesture; `c.start_pan_coord()`.
    pub fn start_pan_coord(&self) -> Vec2<f64> {
        self.touch_dispatch.start_pan
    }
    /// `myStartRotCoord`: the rotation start of a touch gesture; `c.start_rot_coord()`.
    pub fn start_rot_coord(&self) -> Vec2<f64> {
        self.touch_dispatch.start_rotation
    }
    /// `myTouchClick`: the touch of a tap; `c.touch_click()`.
    pub fn touch_click(&self) -> &AspectTouch {
        &self.touch_dispatch.click
    }
    /// `myTouchClickThresholdPx`; `c.touch_click_threshold_px()`.
    pub fn touch_click_threshold_px(&self) -> f32 {
        self.touch_dispatch.click_threshold_px
    }
    /// `myTouchDoubleTapTimer`; `c.touch_double_tap_timer()`.
    pub fn touch_double_tap_timer(&self) -> &Timer {
        &self.touch_dispatch.double_tap_timer
    }
    /// `myTouchRotationThresholdPx`; `c.touch_rotation_threshold_px()`.
    pub fn touch_rotation_threshold_px(&self) -> f32 {
        self.mouse_dispatch.rotation_threshold
    }
    /// `myTouchPanThresholdPx`; `c.touch_pan_threshold_px()`.
    pub fn touch_pan_threshold_px(&self) -> f32 {
        self.mouse_dispatch.pan_threshold
    }
    /// `myTouchZoomThresholdPx`; `c.touch_zoom_threshold_px()`.
    pub fn touch_zoom_threshold_px(&self) -> f32 {
        self.mouse_dispatch.zoom_threshold
    }
    /// `myTouchDraggingThresholdPx`; `c.touch_dragging_threshold_px()`.
    pub fn touch_dragging_threshold_px(&self) -> f32 {
        self.mouse_dispatch.dragging_threshold
    }
    /// `myTouchZoomRatio`; `c.touch_zoom_ratio()`.
    pub fn touch_zoom_ratio(&self) -> f32 {
        self.touch_dispatch.zoom_ratio
    }
    /// `myTouchZRotationThreshold`: two degrees; `c.touch_z_rotation_threshold()`.
    pub fn touch_z_rotation_threshold(&self) -> f32 {
        self.touch_dispatch.z_rotation_threshold
    }
    /// `myUpdateStartPointPan`; `c.update_start_point_pan()`.
    pub fn update_start_point_pan(&self) -> bool {
        self.touch_dispatch.update_start_pan
    }
    /// `myUpdateStartPointRot`; `c.update_start_point_rot()`.
    pub fn update_start_point_rot(&self) -> bool {
        self.touch_dispatch.update_start_rotation
    }
    /// `myUpdateStartPointZRot`: declared true natively and never read; `c.update_start_point_z_rot()`.
    pub fn update_start_point_z_rot(&self) -> bool {
        true
    }
}
