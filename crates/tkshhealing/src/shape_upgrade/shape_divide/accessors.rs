use super::{ShapeUpgradeFaceDivide, ShapeUpgradeShapeDivide};
use crate::{
    shape_build::ShapeBuildReShape,
    shape_extend::{BasicMsgRegistrator, ShapeExtend, ShapeExtendStatus},
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    message::{Gravity, Msg},
    standard::Handle,
};
impl ShapeUpgradeShapeDivide {
    /// Change the source without clearing prior result or flags; `divider.init(&shape)`.
    pub fn init(&self, shape: &Shape) {
        self.with_mut(|f| f.shape = shape.clone());
    }
    /// Set the spatial precision; `divider.set_precision(1e-5)`.
    pub fn set_precision(&self, value: f64) {
        self.with_mut(|f| f.precision = value);
    }
    /// Set the maximum tolerance; `divider.set_max_tolerance(1e-3)`.
    pub fn set_max_tolerance(&self, value: f64) {
        self.with_mut(|f| f.max_tolerance = value);
    }
    /// Set the minimum tolerance; `divider.set_min_tolerance(1e-7)`.
    pub fn set_min_tolerance(&self, value: f64) {
        self.with_mut(|f| f.min_tolerance = value);
    }
    /// Select surface trimming; `divider.set_surface_segment_mode(false)`.
    pub fn set_surface_segment_mode(&self, value: bool) {
        self.with_mut(|f| f.segment_mode = value);
    }
    /// Read the last result; `divider.result()`.
    pub fn result(&self) -> Shape {
        self.with(|f| f.result.clone())
    }
    /// Read shared substitutions; `divider.context()`.
    pub fn context(&self) -> Handle<ShapeBuildReShape> {
        self.with(|f| f.context.clone())
    }
    /// Replace substitutions; `divider.set_context(&context)`.
    pub fn set_context(&self, context: &Handle<ShapeBuildReShape>) {
        self.with_mut(|f| f.context = context.clone());
    }
    /// Read the optional message destination; `divider.msg_registrator()`.
    pub fn msg_registrator(&self) -> Handle<dyn BasicMsgRegistrator> {
        self.with(|f| f.msg_registrator.clone())
    }
    /// Attach a message through the selected registrator; `divider.send_msg(&shape, &message, Gravity::Info)`.
    pub fn send_msg(&self, shape: &Shape, message: &Msg, gravity: Gravity) {
        if let Some(registrator) = self.msg_registrator().get() {
            registrator.send_shape(shape, message, gravity);
        }
    }
    /// Query native status flags; `divider.status_flag(ShapeExtendStatus::Done)`.
    pub fn status_flag(&self, flag: ShapeExtendStatus) -> bool {
        self.with(|f| ShapeExtend::decode_status(f.status, flag))
    }
    /// Replace the face division criterion; `divider.set_split_face_tool(&tool)`.
    pub fn set_split_face_tool(&self, tool: &Handle<ShapeUpgradeFaceDivide>) {
        self.with_mut(|f| f.face_tool = tool.clone());
    }
    /// Choose free/shared/all 3D curve splitting; `divider.set_edge_mode(2)`.
    pub fn set_edge_mode(&self, value: i32) {
        self.with_mut(|f| f.edge_mode = value);
    }
    pub(super) fn add_status(&self, flag: ShapeExtendStatus) {
        self.with_mut(|f| f.status |= ShapeExtend::encode_status(flag));
    }
}
