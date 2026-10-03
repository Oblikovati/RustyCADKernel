use super::*;
use crate::shape_extend::BasicMsgRegistrator;
impl ShapeFixSolid {
    /// Read shell repair mode; `tool.fix_shell_mode()`.
    pub fn fix_shell_mode(&self) -> i32 {
        self.fields.read().shell_mode
    }
    /// Set shell repair mode; `tool.set_fix_shell_mode(0)`.
    pub fn set_fix_shell_mode(&self, mode: i32) {
        self.fields.write().shell_mode = mode;
    }
    /// Read orientation repair mode; `tool.fix_shell_orientation_mode()`.
    pub fn fix_shell_orientation_mode(&self) -> i32 {
        self.fields.read().orientation_mode
    }
    /// Set orientation repair mode; `tool.set_fix_shell_orientation_mode(0)`.
    pub fn set_fix_shell_orientation_mode(&self, mode: i32) {
        self.fields.write().orientation_mode = mode;
    }
    /// Read open-solid creation mode; `tool.create_open_solid_mode()`.
    pub fn create_open_solid_mode(&self) -> bool {
        self.fields.read().create_open
    }
    /// Set open-solid creation mode; `tool.set_create_open_solid_mode(true)`.
    pub fn set_create_open_solid_mode(&self, enabled: bool) {
        self.fields.write().create_open = enabled;
    }
}
impl FixRoot for ShapeFixSolid {
    fn fix_root_data(&self) -> &FixRootData {
        self.parent.fix_root_data()
    }
    fn set_msg_registrator(&self, messages: &Handle<dyn BasicMsgRegistrator>) {
        self.parent.set_msg_registrator(messages);
        self.shell_tool.set_msg_registrator(messages);
    }
    fn set_precision(&self, precision: f64) {
        self.parent.set_precision(precision);
        self.shell_tool.set_precision(precision);
    }
    fn set_min_tolerance(&self, tolerance: f64) {
        self.parent.set_min_tolerance(tolerance);
        self.shell_tool.set_min_tolerance(tolerance);
    }
    fn set_max_tolerance(&self, tolerance: f64) {
        self.parent.set_max_tolerance(tolerance);
        self.shell_tool.set_max_tolerance(tolerance);
    }
}
