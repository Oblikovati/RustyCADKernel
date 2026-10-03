//! Whole-shape division from OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod accessors;
mod boundaries;
mod compound;
mod perform;
use super::ShapeUpgradeFaceDivide;
use crate::{shape_build::ShapeBuildReShape, shape_extend::BasicMsgRegistrator};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    message::Msg,
    precision,
    standard::{Failure, Handle, SharedState},
    tcollection::AsciiString,
};

/// Native protected working state; `divider.shape_divide_base().with(|f| f.status)`.
pub struct ShapeDivideFields {
    /// Reshape substitutions shared across recursive calls.
    pub context: Handle<ShapeBuildReShape>,
    /// Optional message destination.
    pub msg_registrator: Handle<dyn BasicMsgRegistrator>,
    /// Current source, restored after successful compound traversal.
    pub shape: Shape,
    /// Last result, retained across Init and null-source failure.
    pub result: Shape,
    /// Working precision passed to child tools.
    pub precision: f64,
    /// Minimum tolerance passed to wire tools.
    pub min_tolerance: f64,
    /// Maximum tolerance passed to child tools.
    pub max_tolerance: f64,
    /// Whether supporting surfaces are segmented.
    pub segment_mode: bool,
    /// Native ShapeExtend status bits.
    pub status: i32,
    /// Curve splitting mode for free/shared edges.
    pub edge_mode: i32,
    face_tool: Handle<ShapeUpgradeFaceDivide>,
}
impl Default for ShapeDivideFields {
    fn default() -> Self {
        Self {
            context: Handle::new(ShapeBuildReShape::new()),
            msg_registrator: Handle::default(),
            shape: Shape::default(),
            result: Shape::default(),
            precision: precision::confusion(),
            min_tolerance: precision::confusion(),
            max_tolerance: 1.,
            segment_mode: true,
            status: 0,
            edge_mode: 2,
            face_tool: Handle::new(ShapeUpgradeFaceDivide::empty()),
        }
    }
}
/// Virtual shape division policy; implement `shape_divide_base()` and override criteria as needed.
pub trait ShapeDivider {
    /// Shared base and protected fields; `divider.shape_divide_base()`.
    fn shape_divide_base(&self) -> &ShapeUpgradeShapeDivide;
    /// Divide a shape and optionally discard previous substitutions; `divider.perform(true)?`.
    fn perform(&self, new_context: bool) -> Result<bool, Failure> {
        perform::perform(self, new_context)
    }
    /// Select the face criterion; `divider.split_face_tool()`.
    fn split_face_tool(&self) -> Handle<ShapeUpgradeFaceDivide> {
        self.shape_divide_base().with(|f| f.face_tool.clone())
    }
    /// Replace the virtual message destination; `divider.set_msg_registrator(&registrator)`.
    fn set_msg_registrator(&self, registrator: &Handle<dyn BasicMsgRegistrator>) {
        self.shape_divide_base()
            .with_mut(|f| f.msg_registrator = registrator.clone());
    }
    /// Native face-completion message; `divider.face_msg()`.
    fn face_msg(&self) -> Msg {
        Msg::from_key(&AsciiString::from("ShapeDivide.FaceDivide.MSG0"))
    }
    /// Native wire-completion message; `divider.wire_msg()`.
    fn wire_msg(&self) -> Msg {
        Msg::from_key(&AsciiString::from("ShapeDivide.WireDivide.MSG0"))
    }
    /// Native edge-completion message; `divider.edge_msg()`.
    fn edge_msg(&self) -> Msg {
        Msg::from_key(&AsciiString::from("ShapeDivide.EdgeDivide.MSG0"))
    }
}
/// Face, free-wire and free-edge subdivision; `ShapeUpgradeShapeDivide::new(shape)`.
#[derive(Default)]
pub struct ShapeUpgradeShapeDivide {
    fields: SharedState<ShapeDivideFields>,
}
impl ShapeDivider for ShapeUpgradeShapeDivide {
    fn shape_divide_base(&self) -> &ShapeUpgradeShapeDivide {
        self
    }
}
impl ShapeUpgradeShapeDivide {
    /// Construct without a source shape; `ShapeUpgradeShapeDivide::empty()`.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Construct for a shape; `ShapeUpgradeShapeDivide::new(shape)`.
    pub fn new(shape: Shape) -> Self {
        let divider = Self::empty();
        divider.init(&shape);
        divider
    }
    /// Read protected fields; `divider.with(|f| f.precision)`.
    pub fn with<R>(&self, read: impl FnOnce(&ShapeDivideFields) -> R) -> R {
        read(&self.fields.read())
    }
    /// Modify protected fields in derived policies; `divider.with_mut(|f| f.status = 0)`.
    pub fn with_mut<R>(&self, write: impl FnOnce(&mut ShapeDivideFields) -> R) -> R {
        write(&mut self.fields.write())
    }
    /// Divide with default policies; `divider.perform(true)?`.
    pub fn perform(&self, new_context: bool) -> Result<bool, Failure> {
        ShapeDivider::perform(self, new_context)
    }
}
