//! Motion event records preserve coordinates when their request flags reset.
use rustycad_tkernel::ncollection::Vec2;
/// Pending panning gesture; `AisViewInputPanning::default()` starts idle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AisViewInputPanning {
    /// Start panning.
    pub to_start: bool,
    /// Retained pixel at the start.
    pub point_start: Vec2<i32>,
    /// Perform a pan.
    pub to_pan: bool,
    /// Retained pixel delta.
    pub delta: Vec2<i32>,
}
/// Pending object drag lifecycle; `AisViewInputDragging::default()` has no actions.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AisViewInputDragging {
    /// Start the drag.
    pub to_start: bool,
    /// Confirm a started drag.
    pub to_confirm: bool,
    /// Move the dragged object.
    pub to_move: bool,
    /// Finish the drag.
    pub to_stop: bool,
    /// Abort and restore the original position.
    pub to_abort: bool,
    /// Retained start pixel.
    pub point_start: Vec2<i32>,
    /// Retained end pixel.
    pub point_to: Vec2<i32>,
}
// Both upstream records expose the same state; generate distinct Rust types without duplicating policy.
macro_rules! view_rotation_record {
    ($name:ident, $intent:literal) => {
        #[doc = $intent]
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $name {
            /// Start the rotation.
            pub to_start: bool,
            /// Retained starting coordinate.
            pub point_start: Vec2<f64>,
            /// Perform the rotation.
            pub to_rotate: bool,
            /// Retained ending coordinate.
            pub point_to: Vec2<f64>,
        }
    };
}
view_rotation_record!(
    AisViewInputOrbitRotation,
    "Orbit gesture; `AisViewInputOrbitRotation::default()` starts idle."
);
view_rotation_record!(
    AisViewInputRotation,
    "View gesture; `AisViewInputRotation::default()` starts idle."
);
/// Pending Z rotation; `AisViewInputZRotation::default()` retains a zero angle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AisViewInputZRotation {
    /// Retained starting pixel.
    pub point: Vec2<i32>,
    /// Retained angle in radians.
    pub angle: f64,
    /// Request the rotation.
    pub to_rotate: bool,
}
