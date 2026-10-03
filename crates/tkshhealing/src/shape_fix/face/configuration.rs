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
