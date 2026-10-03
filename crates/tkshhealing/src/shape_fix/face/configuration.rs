use super::{FixRoot, FixRootData, Handle, ShapeFixFace};
use crate::{shape_extend::BasicMsgRegistrator, shape_fix::ShapeFixWire};
impl ShapeFixFace {
    /// Borrow the shared wire tool; `face.fix_wire_tool()`.
    pub fn fix_wire_tool(&self) -> Handle<ShapeFixWire> {
        self.wire_tool.clone()
    }
    /// Read the small-area-wire mode; `face.fix_small_area_wire_mode()`.
    pub fn fix_small_area_wire_mode(&self) -> i32 {
        self.modes.read().small_area_wire
    }
    /// Set the small-area-wire mode; `face.set_fix_small_area_wire_mode(1)`.
    pub fn set_fix_small_area_wire_mode(&self, mode: i32) {
        self.modes.write().small_area_wire = mode;
    }
}
impl FixRoot for ShapeFixFace {
    fn fix_root_data(&self) -> &FixRootData {
        self.parent.fix_root_data()
    }
    fn set_msg_registrator(&self, messages: &Handle<dyn BasicMsgRegistrator>) {
        self.parent.set_msg_registrator(messages);
        self.wire_tool.set_msg_registrator(messages);
    }
    fn set_precision(&self, precision: f64) {
        self.parent.set_precision(precision);
        self.wire_tool.set_precision(precision);
    }
    fn set_min_tolerance(&self, tolerance: f64) {
        self.parent.set_min_tolerance(tolerance);
        self.wire_tool.set_min_tolerance(tolerance);
    }
    fn set_max_tolerance(&self, tolerance: f64) {
        self.parent.set_max_tolerance(tolerance);
        self.wire_tool.set_max_tolerance(tolerance);
    }
}

#[derive(Clone, Copy)]
pub(super) struct FaceFixModes {
    pub wire: i32,
    pub orientation: i32,
    pub add_natural_bound: i32,
    pub missing_seam: i32,
    pub small_area_wire: i32,
    pub remove_small_area_face: i32,
    pub intersecting_wires: i32,
    pub loop_wires: i32,
    pub split_face: i32,
    pub auto_precision: i32,
    pub periodic_degenerated: i32,
}
impl Default for FaceFixModes {
    fn default() -> Self {
        Self {
            wire: -1,
            orientation: -1,
            add_natural_bound: -1,
            missing_seam: -1,
            small_area_wire: -1,
            remove_small_area_face: -1,
            intersecting_wires: -1,
            loop_wires: -1,
            split_face: -1,
            auto_precision: 1,
            periodic_degenerated: -1,
        }
    }
}
impl ShapeFixFace {
    /// Reset face repair modes without resetting child tools; `tool.clear_modes()`.
    pub fn clear_modes(&self) {
        *self.modes.write() = FaceFixModes::default();
    }
    /// Read the native face repair mode; `tool.fix_wire_mode()`.
    pub fn fix_wire_mode(&self) -> i32 {
        self.modes.read().wire
    }
    /// Set the native face repair mode; `tool.set_fix_wire_mode(0)`.
    pub fn set_fix_wire_mode(&self, mode: i32) {
        self.modes.write().wire = mode;
    }
    /// Read the native face repair mode; `tool.fix_missing_seam_mode()`.
    pub fn fix_missing_seam_mode(&self) -> i32 {
        self.modes.read().missing_seam
    }
    /// Set the native face repair mode; `tool.set_fix_missing_seam_mode(0)`.
    pub fn set_fix_missing_seam_mode(&self, mode: i32) {
        self.modes.write().missing_seam = mode;
    }
    /// Read the native face repair mode; `tool.remove_small_area_face_mode()`.
    pub fn remove_small_area_face_mode(&self) -> i32 {
        self.modes.read().remove_small_area_face
    }
    /// Set the native face repair mode; `tool.set_remove_small_area_face_mode(0)`.
    pub fn set_remove_small_area_face_mode(&self, mode: i32) {
        self.modes.write().remove_small_area_face = mode;
    }
    /// Read the native face repair mode; `tool.fix_intersecting_wires_mode()`.
    pub fn fix_intersecting_wires_mode(&self) -> i32 {
        self.modes.read().intersecting_wires
    }
    /// Set the native face repair mode; `tool.set_fix_intersecting_wires_mode(0)`.
    pub fn set_fix_intersecting_wires_mode(&self, mode: i32) {
        self.modes.write().intersecting_wires = mode;
    }
    /// Read the native face repair mode; `tool.fix_loop_wires_mode()`.
    pub fn fix_loop_wires_mode(&self) -> i32 {
        self.modes.read().loop_wires
    }
    /// Set the native face repair mode; `tool.set_fix_loop_wires_mode(0)`.
    pub fn set_fix_loop_wires_mode(&self, mode: i32) {
        self.modes.write().loop_wires = mode;
    }
    /// Read the native face repair mode; `tool.fix_split_face_mode()`.
    pub fn fix_split_face_mode(&self) -> i32 {
        self.modes.read().split_face
    }
    /// Set the native face repair mode; `tool.set_fix_split_face_mode(0)`.
    pub fn set_fix_split_face_mode(&self, mode: i32) {
        self.modes.write().split_face = mode;
    }
    /// Read the native face repair mode; `tool.auto_correct_precision_mode()`.
    pub fn auto_correct_precision_mode(&self) -> i32 {
        self.modes.read().auto_precision
    }
    /// Set the native face repair mode; `tool.set_auto_correct_precision_mode(0)`.
    pub fn set_auto_correct_precision_mode(&self, mode: i32) {
        self.modes.write().auto_precision = mode;
    }
    /// Read the native face repair mode; `tool.fix_periodic_degenerated_mode()`.
    pub fn fix_periodic_degenerated_mode(&self) -> i32 {
        self.modes.read().periodic_degenerated
    }
    /// Set the native face repair mode; `tool.set_fix_periodic_degenerated_mode(0)`.
    pub fn set_fix_periodic_degenerated_mode(&self, mode: i32) {
        self.modes.write().periodic_degenerated = mode;
    }
}
