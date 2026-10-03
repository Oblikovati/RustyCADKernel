use super::*;
use crate::shape_extend::BasicMsgRegistrator;
impl ShapeFixShell {
    /// Set face repair mode; `tool.set_fix_face_mode(0)`.
    pub fn set_fix_face_mode(&self, mode: i32) {
        self.state.write().face_mode = mode;
    }
    /// Read face repair mode; `tool.fix_face_mode()`.
    pub fn fix_face_mode(&self) -> i32 {
        self.state.read().face_mode
    }
    /// Set orientation repair mode; `tool.set_fix_orientation_mode(0)`.
    pub fn set_fix_orientation_mode(&self, mode: i32) {
        self.state.write().orientation_mode = mode;
    }
    /// Read orientation repair mode; `tool.fix_orientation_mode()`.
    pub fn fix_orientation_mode(&self) -> i32 {
        self.state.read().orientation_mode
    }
    /// Enable nonmanifold merging; `tool.set_non_manifold_flag(true)`.
    pub fn set_non_manifold_flag(&self, enabled: bool) {
        self.state.write().non_manifold = enabled;
    }
}
impl FixRoot for ShapeFixShell {
    fn fix_root_data(&self) -> &FixRootData {
        self.parent.fix_root_data()
    }
    fn set_msg_registrator(&self, messages: &Handle<dyn BasicMsgRegistrator>) {
        self.parent.set_msg_registrator(messages);
        self.face_tool.set_msg_registrator(messages);
    }
    fn set_precision(&self, precision: f64) {
        self.parent.set_precision(precision);
        self.face_tool.set_precision(precision);
    }
    fn set_min_tolerance(&self, tolerance: f64) {
        self.parent.set_min_tolerance(tolerance);
        self.face_tool.set_min_tolerance(tolerance);
    }
    fn set_max_tolerance(&self, tolerance: f64) {
        self.parent.set_max_tolerance(tolerance);
        self.face_tool.set_max_tolerance(tolerance);
    }
}
