// Rust translation of ModelingAlgorithms/TKShHealing/ShapeBuild/ShapeBuild_ReShape.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeBuild_ReShape`: the Shape Healing re-shaper (docs/shhealing-design.md section 3).
mod apply;
mod build_mode;

use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape, ReShapeData};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `ShapeBuild_ReShape`: records requests to replace or remove shapes, then applies them
/// to any shape that contains them, rebuilding it (`BRepTools_ReShape` with its own
/// `Apply`, a build-mode `Apply` for solids and shells, and the ShapeExtend status of the
/// last `Apply`).
///
/// ```
/// use rustycad_tkbrep::brep_tools::ReShape;
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
/// use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
/// let reshape = ShapeBuildReShape::new();
/// assert!(reshape.apply(&Shape::default(), ShapeEnum::Shape).unwrap().is_null());
/// assert!(reshape.status_flag(ShapeExtendStatus::Ok));
/// ```
#[derive(Default)]
pub struct ShapeBuildReShape {
    parent: BRepToolsReShape,
}
impl_transient!(
    ShapeBuildReShape,
    "ShapeBuild_ReShape",
    BRepToolsReShape,
    parent,
    interfaces: [dyn ReShape]
);

impl ShapeBuildReShape {
    /// `ShapeBuild_ReShape()`: no request, status -1 (as the base class);
    /// `ShapeBuildReShape::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Status(ShapeExtend_Status)`: the flags of the last `Apply(shape, until)`: OK when
    /// nothing was replaced or removed, DONE1 the shape itself replaced, DONE2 removed,
    /// DONE3 sub-shapes replaced, DONE4 sub-shapes removed, FAIL1 a replacement of an
    /// incompatible type ignored; `reshape.status_flag(ShapeExtendStatus::Done3)`.
    pub fn status_flag(&self, status: ShapeExtendStatus) -> bool {
        let bits = self.reshape_data().with(|fields| fields.status);
        ShapeExtend::decode_status(bits, status)
    }

    /// Writes `myStatus`.
    fn set_status(&self, bits: i32) {
        self.reshape_data().with_mut(|fields| fields.status = bits);
    }

    /// Reads `myStatus`.
    fn status_bits(&self) -> i32 {
        self.reshape_data().with(|fields| fields.status)
    }
}

impl ReShape for ShapeBuildReShape {
    fn reshape_data(&self) -> &ReShapeData {
        ReShape::reshape_data(&self.parent)
    }

    /// `Apply(shape, until)` (override): the recorded replacements applied recursively
    /// down to the type `until`; a sub-shape can be replaced by a shape of its type or by
    /// a container of such shapes (their items are added one by one; an item of another
    /// type, or an empty container, is ignored with FAIL1). The status of the last call
    /// is kept for [`ShapeBuildReShape::status_flag`].
    fn apply(&self, shape: &Shape, until: ShapeEnum) -> Result<Shape, Failure> {
        let mut in_flight = Vec::new();
        apply::apply_impl(self, shape, until, &mut in_flight)
    }
}
