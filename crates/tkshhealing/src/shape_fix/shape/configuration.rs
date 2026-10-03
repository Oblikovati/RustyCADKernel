use super::*;
use crate::shape_extend::BasicMsgRegistrator;
use crate::shape_fix::{ShapeFixEdge, ShapeFixFace, ShapeFixShell, ShapeFixSolid, ShapeFixWire};
pub(super) struct ShapeFixModes {
    pub solid: i32,
    pub shell: i32,
    pub face: i32,
    pub wire: i32,
    pub same_parameter: i32,
    pub vertex_position: i32,
    pub vertex_tolerance: i32,
}
impl Default for ShapeFixShape {
    fn default() -> Self {
        Self {
            parent: super::super::ShapeFixRoot::new(),
            result: SharedState::new(Shape::default()),
            status: SharedState::new(0),
            solid_tool: Handle::new(ShapeFixSolid::new()),
            visited: SharedState::new(MapOfShape::default()),
            modes: SharedState::new(ShapeFixModes {
                solid: -1,
                shell: -1,
                face: -1,
                wire: -1,
                same_parameter: -1,
                vertex_position: 0,
                vertex_tolerance: -1,
            }),
        }
    }
}
impl ShapeFixShape {
    /// Borrow the shared solid repair tool; `tool.fix_solid_tool()`.
    pub fn fix_solid_tool(&self) -> Handle<ShapeFixSolid> {
        self.solid_tool.clone()
    }
    /// Borrow the shared shell repair tool; `tool.fix_shell_tool()`.
    pub fn fix_shell_tool(&self) -> Handle<ShapeFixShell> {
        self.fix_solid_tool().fix_shell_tool()
    }
    /// Borrow the shared face repair tool; `tool.fix_face_tool()`.
    pub fn fix_face_tool(&self) -> Handle<ShapeFixFace> {
        self.fix_shell_tool().fix_face_tool()
    }
    /// Borrow the shared wire repair tool; `tool.fix_wire_tool()`.
    pub fn fix_wire_tool(&self) -> Handle<ShapeFixWire> {
        self.fix_face_tool().fix_wire_tool()
    }
    /// Borrow the shared edge repair tool; `tool.fix_edge_tool()`.
    pub fn fix_edge_tool(&self) -> Handle<ShapeFixEdge> {
        self.fix_wire_tool().fix_edge_tool()
    }
    /// Read the native repair mode; `tool.fix_solid_mode()`.
    pub fn fix_solid_mode(&self) -> i32 {
        self.modes.read().solid
    }
    /// Set the native repair mode; `tool.set_fix_solid_mode(0)`.
    pub fn set_fix_solid_mode(&self, mode: i32) {
        self.modes.write().solid = mode;
    }
    /// Read the native repair mode; `tool.fix_free_shell_mode()`.
    pub fn fix_free_shell_mode(&self) -> i32 {
        self.modes.read().shell
    }
    /// Set the native repair mode; `tool.set_fix_free_shell_mode(0)`.
    pub fn set_fix_free_shell_mode(&self, mode: i32) {
        self.modes.write().shell = mode;
    }
    /// Read the native repair mode; `tool.fix_free_face_mode()`.
    pub fn fix_free_face_mode(&self) -> i32 {
        self.modes.read().face
    }
    /// Set the native repair mode; `tool.set_fix_free_face_mode(0)`.
    pub fn set_fix_free_face_mode(&self, mode: i32) {
        self.modes.write().face = mode;
    }
    /// Read the native repair mode; `tool.fix_free_wire_mode()`.
    pub fn fix_free_wire_mode(&self) -> i32 {
        self.modes.read().wire
    }
    /// Set the native repair mode; `tool.set_fix_free_wire_mode(0)`.
    pub fn set_fix_free_wire_mode(&self, mode: i32) {
        self.modes.write().wire = mode;
    }
    /// Read the native repair mode; `tool.fix_same_parameter_mode()`.
    pub fn fix_same_parameter_mode(&self) -> i32 {
        self.modes.read().same_parameter
    }
    /// Set the native repair mode; `tool.set_fix_same_parameter_mode(0)`.
    pub fn set_fix_same_parameter_mode(&self, mode: i32) {
        self.modes.write().same_parameter = mode;
    }
    /// Read the native repair mode; `tool.fix_vertex_position_mode()`.
    pub fn fix_vertex_position_mode(&self) -> i32 {
        self.modes.read().vertex_position
    }
    /// Set the native repair mode; `tool.set_fix_vertex_position_mode(0)`.
    pub fn set_fix_vertex_position_mode(&self, mode: i32) {
        self.modes.write().vertex_position = mode;
    }
    /// Read the native repair mode; `tool.fix_vertex_tol_mode()`.
    pub fn fix_vertex_tol_mode(&self) -> i32 {
        self.modes.read().vertex_tolerance
    }
    /// Set the native repair mode; `tool.set_fix_vertex_tol_mode(0)`.
    pub fn set_fix_vertex_tol_mode(&self, mode: i32) {
        self.modes.write().vertex_tolerance = mode;
    }
}
impl FixRoot for ShapeFixShape {
    fn fix_root_data(&self) -> &FixRootData {
        self.parent.fix_root_data()
    }
    fn set_msg_registrator(&self, messages: &Handle<dyn BasicMsgRegistrator>) {
        self.parent.set_msg_registrator(messages);
        self.solid_tool.set_msg_registrator(messages);
    }
    fn set_precision(&self, precision: f64) {
        self.parent.set_precision(precision);
        self.solid_tool.set_precision(precision);
    }
    fn set_min_tolerance(&self, tolerance: f64) {
        self.parent.set_min_tolerance(tolerance);
        self.solid_tool.set_min_tolerance(tolerance);
    }
    fn set_max_tolerance(&self, tolerance: f64) {
        self.parent.set_max_tolerance(tolerance);
        self.solid_tool.set_max_tolerance(tolerance);
    }
}
