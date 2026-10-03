//! ShapeFix_Solid from OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod config;
mod from_shell;
use super::{FixRoot, FixRootData, ShapeFixRoot, ShapeFixShell};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::topods::{Shape, Solid};
use rustycad_tkernel::{
    impl_transient,
    standard::{Handle, SharedState},
};
/// Solid construction and shell orientation repair; `ShapeFixSolid::from_solid(&solid)`.
pub struct ShapeFixSolid {
    parent: ShapeFixRoot,
    fields: SharedState<SolidRepairState>,
    shell_tool: Handle<ShapeFixShell>,
}
struct SolidRepairState {
    solid: Shape,
    status: i32,
    shell_mode: i32,
    orientation_mode: i32,
    create_open: bool,
}
impl Default for ShapeFixSolid {
    fn default() -> Self {
        Self {
            parent: ShapeFixRoot::new(),
            shell_tool: Handle::new(ShapeFixShell::new()),
            fields: SharedState::new(SolidRepairState {
                solid: Shape::default(),
                status: 0,
                shell_mode: -1,
                orientation_mode: -1,
                create_open: false,
            }),
        }
    }
}
impl_transient!(ShapeFixSolid,"ShapeFix_Solid",ShapeFixRoot,parent,interfaces:[dyn FixRoot]);
impl ShapeFixSolid {
    /// Construct an unloaded repair tool; `ShapeFixSolid::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Load a solid; `ShapeFixSolid::from_solid(&solid)`.
    pub fn from_solid(solid: &Solid) -> Self {
        let tool = Self::new();
        tool.init(solid);
        tool
    }
    /// Load without resetting modes or status; `tool.init(&solid)`.
    pub fn init(&self, solid: &Solid) {
        self.fields.write().solid = solid.clone().into_shape();
        self.parent
            .fix_root_data()
            .with_mut(|fields| fields.shape = solid.clone().into_shape());
    }
    /// Read the retained diagnostic; `tool.status_flag(ShapeExtendStatus::Done2)`.
    pub fn status_flag(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status, status)
    }
    /// Return the stored solid result; `tool.solid()`.
    pub fn solid(&self) -> Shape {
        self.fields.read().solid.clone()
    }
    /// Return the context-adjusted shape; `tool.shape()`.
    pub fn shape(&self) -> Shape {
        self.parent
            .fix_root_data()
            .with(|fields| fields.shape.clone())
    }
    /// Borrow the shared shell tool; `tool.fix_shell_tool()`.
    pub fn fix_shell_tool(&self) -> Handle<ShapeFixShell> {
        self.shell_tool.clone()
    }
}
mod collect;
mod create;
mod perform;
