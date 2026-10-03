// Translation of ShapeFix_Edge parameter and vertex repair at
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod add_pcurve;
mod curves;
mod parameter;
mod range;
mod seam;
mod vertex;

use crate::shape_build::ShapeBuildReShape;
use crate::shape_construct::ShapeConstructProjectCurveOnSurface;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Handle, StandardTransient};

/// Repair missing or inconsistent edge curves, parameterization and vertex tolerances.
///
/// ```
/// use rustycad_tkshhealing::shape_fix::ShapeFixEdge;
/// use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
/// assert!(ShapeFixEdge::new().status_flag(ShapeExtendStatus::Ok));
/// ```
pub struct ShapeFixEdge {
    parent: StandardTransient,
    status: i32,
    context: Handle<ShapeBuildReShape>,
    projector: Handle<ShapeConstructProjectCurveOnSurface>,
}
impl_transient!(ShapeFixEdge, "ShapeFix_Edge", StandardTransient, parent);

impl Default for ShapeFixEdge {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            status: 0,
            context: Handle::default(),
            projector: Handle::new(ShapeConstructProjectCurveOnSurface::new()),
        }
    }
}

impl ShapeFixEdge {
    /// Share the projector so callers can configure projection; `tool.projector()`.
    pub fn projector(&self) -> Handle<ShapeConstructProjectCurveOnSurface> {
        self.projector.clone()
    }

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
