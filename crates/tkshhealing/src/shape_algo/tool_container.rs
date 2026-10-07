// Rust translation of ModelingAlgorithms/TKShHealing/ShapeAlgo/ShapeAlgo_ToolContainer at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeAlgo_ToolContainer`: the factory of the tools the algorithm container uses.
use crate::shape_fix::{ShapeFixEdgeProjAux, ShapeFixShape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Handle, StandardTransient};

/// `ShapeAlgo_ToolContainer`: returns new Shape Healing tools. The native members are
/// virtual, but no OCCT class overrides them.
///
/// ```
/// use rustycad_tkshhealing::shape_algo::ShapeAlgoToolContainer;
/// let tools = ShapeAlgoToolContainer::new();
/// assert!(!tools.fix_shape().is_null());
/// assert!(!tools.edge_proj_aux().is_null());
/// ```
#[derive(Default)]
pub struct ShapeAlgoToolContainer {
    parent: StandardTransient,
}
impl_transient!(
    ShapeAlgoToolContainer,
    "ShapeAlgo_ToolContainer",
    StandardTransient,
    parent
);

impl ShapeAlgoToolContainer {
    /// `ShapeAlgo_ToolContainer()`; `ShapeAlgoToolContainer::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `FixShape()`: a new empty `ShapeFix_Shape`; `tools.fix_shape()`.
    pub fn fix_shape(&self) -> Handle<ShapeFixShape> {
        Handle::new(ShapeFixShape::empty())
    }

    /// `EdgeProjAux()`: a new empty `ShapeFix_EdgeProjAux`; `tools.edge_proj_aux()`.
    pub fn edge_proj_aux(&self) -> Handle<ShapeFixEdgeProjAux> {
        Handle::new(ShapeFixEdgeProjAux::new())
    }
}
