// Translation of ShapeFix_Edge parameter and vertex repair at
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod parameter;
mod range;
mod vertex;

use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Handle, StandardTransient};

/// Repair edge parameter consistency and vertex tolerances.
/// Missing-curve projection and removal operations remain separate porting work.
///
/// ```
/// use rustycad_tkshhealing::shape_fix::ShapeFixEdge;
/// use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
/// assert!(ShapeFixEdge::new().status_flag(ShapeExtendStatus::Ok));
/// ```
#[derive(Default)]
pub struct ShapeFixEdge {
    parent: StandardTransient,
    status: i32,
    context: Handle<ShapeBuildReShape>,
}
impl_transient!(ShapeFixEdge, "ShapeFix_Edge", StandardTransient, parent);

impl ShapeFixEdge {
    /// Construct a repair tool with no replacement context; `ShapeFixEdge::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Query flags from the latest repair; `tool.status_flag(ShapeExtendStatus::Done)`.
    pub fn status_flag(&self, flag: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.status, flag)
    }

    /// Share the vertex-replacement context; `tool.set_context(&context)`.
    pub fn set_context(&mut self, context: &Handle<ShapeBuildReShape>) {
        self.context = context.clone();
    }

    /// Return the current replacement context; `let context = tool.context()`.
    pub fn context(&self) -> Handle<ShapeBuildReShape> {
        self.context.clone()
    }

    fn add_status(&mut self, flag: ShapeExtendStatus) {
        self.status |= ShapeExtend::encode_status(flag);
    }
}
