//! Closed-edge subdivision policy from OCCT 3d097a0328e71b8.
use super::{
    ShapeDivider, ShapeUpgradeClosedEdgeDivide, ShapeUpgradeFaceDivide, ShapeUpgradeShapeDivide,
    ShapeUpgradeWireDivide,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;

/// Split closed edges with the native midpoint criterion; `ShapeUpgradeShapeDivideClosedEdges::new(shape)`.
pub struct ShapeUpgradeShapeDivideClosedEdges {
    parent: ShapeUpgradeShapeDivide,
}
impl ShapeDivider for ShapeUpgradeShapeDivideClosedEdges {
    fn shape_divide_base(&self) -> &ShapeUpgradeShapeDivide {
        &self.parent
    }
}
impl std::ops::Deref for ShapeUpgradeShapeDivideClosedEdges {
    type Target = ShapeUpgradeShapeDivide;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
impl ShapeUpgradeShapeDivideClosedEdges {
    /// Initialize the shape and closed-edge criterion; `ShapeUpgradeShapeDivideClosedEdges::new(shape)`.
    pub fn new(shape: Shape) -> Self {
        let divider = Self {
            parent: ShapeUpgradeShapeDivide::new(shape),
        };
        divider.set_nb_split_points(1);
        divider
    }
    /// Recreate the criterion; native intentionally ignores the count; `divider.set_nb_split_points(3)`.
    pub fn set_nb_split_points(&self, _count: i32) {
        let edge = Handle::new(ShapeUpgradeClosedEdgeDivide::new());
        let wire = Handle::new(ShapeUpgradeWireDivide::new());
        wire.set_edge_divide_tool(&edge.cast());
        let face = Handle::new(ShapeUpgradeFaceDivide::empty());
        face.set_wire_divide_tool(&wire);
        face.set_split_surface_tool(&Handle::default());
        self.set_split_face_tool(&face);
    }
}
