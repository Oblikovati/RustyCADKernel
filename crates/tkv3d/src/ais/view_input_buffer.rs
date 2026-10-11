//! Native AIS event reset policy, independent of a renderer or input backend.
use super::{view_input_motion::*, view_input_types::*};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkservice::aspect::AspectScrollDelta;
/// Pending viewer events; `AisViewInputBuffer::new().reset()` acknowledges one frame's requests.
#[derive(Clone, Default)]
pub struct AisViewInputBuffer {
    /// Transition between gestures.
    pub is_new_gesture: bool,
    /// One-based queue of scroll actions.
    pub zoom_actions: Sequence<AspectScrollDelta>,
    /// Fit and orientation requests.
    pub orientation: AisViewInputOrientation,
    /// Dynamic highlight request.
    pub move_to: AisViewInputHighlighting,
    /// Selection tool and retained points.
    pub selection: AisViewInputSelection,
    /// Panning request.
    pub panning: AisViewInputPanning,
    /// Object dragging request.
    pub dragging: AisViewInputDragging,
    /// Orbit rotation request.
    pub orbit_rotation: AisViewInputOrbitRotation,
    /// View rotation request.
    pub view_rotation: AisViewInputRotation,
    /// Z rotation request.
    pub z_rotate: AisViewInputZRotation,
}
impl AisViewInputBuffer {
    /// Start with no pending events and native defaults; `let events = AisViewInputBuffer::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Acknowledge requests while retaining gesture coordinates; `events.reset()` keeps selection points.
    pub fn reset(&mut self) {
        // OCCT 3d097a0 keeps persistent gesture state; replacing the buffer with Default loses it.
        self.orientation.to_fit_all = false;
        self.orientation.to_set_view_orient = false;
        self.move_to.to_hilight = false;
        self.selection.to_apply_tool = false;
        self.is_new_gesture = false;
        self.zoom_actions.clear(None);
        self.reset_motion_requests();
    }
    fn reset_motion_requests(&mut self) {
        self.panning.to_start = false;
        self.panning.to_pan = false;
        self.dragging.to_start = false;
        self.dragging.to_confirm = false;
        self.dragging.to_move = false;
        self.dragging.to_stop = false;
        self.dragging.to_abort = false;
        self.orbit_rotation.to_start = false;
        self.orbit_rotation.to_rotate = false;
        self.view_rotation.to_start = false;
        self.view_rotation.to_rotate = false;
        self.z_rotate.to_rotate = false;
    }
}
