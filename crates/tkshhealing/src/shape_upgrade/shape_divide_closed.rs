//! Whole-shape division restricted to closed faces.
use super::{ShapeDivider, ShapeUpgradeClosedFaceDivide, ShapeUpgradeShapeDivide};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use std::ops::Deref;

/// Divides closed faces and leaves free wires to the inherited whole-shape policy.
///
/// ```
/// use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis};
/// use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeShapeDivideClosed;
/// let mut maker = BRepPrimAPIMakeCylinder::new(2.0, 5.0);
/// let cylinder = maker.face()?.into_shape();
/// let divider = ShapeUpgradeShapeDivideClosed::new(cylinder);
/// divider.set_nb_split_points(2);
/// assert!(divider.perform(true)?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct ShapeUpgradeShapeDivideClosed {
    parent: ShapeUpgradeShapeDivide,
}

impl Deref for ShapeUpgradeShapeDivideClosed {
    type Target = ShapeUpgradeShapeDivide;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl ShapeUpgradeShapeDivideClosed {
    /// Initialize the whole-shape divider with its native one-cut closed-face tool.
    /// `ShapeUpgradeShapeDivideClosed::new(shape)`.
    pub fn new(shape: Shape) -> Self {
        let divider = Self {
            parent: ShapeUpgradeShapeDivide::new(shape),
        };
        divider.set_nb_split_points(1);
        divider
    }

    /// Replace the closed-face policy; nonpositive values retain the child default of one.
    /// `divider.set_nb_split_points(2)`.
    pub fn set_nb_split_points(&self, count: i32) {
        let face_tool = ShapeUpgradeClosedFaceDivide::empty();
        face_tool.set_nb_split_points(count);
        face_tool.set_wire_divide_tool(&Handle::default());
        self.parent
            .set_split_face_tool(&Handle::new(face_tool).cast());
    }

    /// Perform closed-face division with the inherited context policy.
    /// `divider.perform(true)?`.
    pub fn perform(&self, new_context: bool) -> Result<bool, rustycad_tkernel::standard::Failure> {
        ShapeDivider::perform(self, new_context)
    }
}

impl ShapeDivider for ShapeUpgradeShapeDivideClosed {
    fn shape_divide_base(&self) -> &ShapeUpgradeShapeDivide {
        &self.parent
    }
}
