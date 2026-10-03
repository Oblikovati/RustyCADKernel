//! ShapeFix_Shell from OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod config;
mod groups;
mod orientation;
use super::{FixRoot, FixRootData, ShapeFixFace, ShapeFixRoot};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::topods::{Compound, Shape, Shell};
use rustycad_tkernel::{
    impl_transient,
    standard::{Handle, SharedState},
};
/// Shell face healing and orientation; `ShapeFixShell::from_shell(&shell)`.
pub struct ShapeFixShell {
    parent: ShapeFixRoot,
    state: SharedState<ShellRepairState>,
    face_tool: Handle<ShapeFixFace>,
}
struct ShellRepairState {
    shell: Shell,
    errors: Compound,
    status: i32,
    count: i32,
    face_mode: i32,
    orientation_mode: i32,
    non_manifold: bool,
}
impl Default for ShapeFixShell {
    fn default() -> Self {
        Self {
            parent: ShapeFixRoot::new(),
            face_tool: Handle::new(ShapeFixFace::new(Shape::default())),
            state: SharedState::new(ShellRepairState {
                shell: Shell::default(),
                errors: Compound::default(),
                status: 0,
                count: 0,
                face_mode: -1,
                orientation_mode: -1,
                non_manifold: false,
            }),
        }
    }
}
impl_transient!(ShapeFixShell,"ShapeFix_Shell",ShapeFixRoot,parent,interfaces:[dyn FixRoot]);
impl ShapeFixShell {
    /// Construct an unloaded shell repair tool; `ShapeFixShell::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Initialize a shell repair tool; `ShapeFixShell::from_shell(&shell)`.
    pub fn from_shell(shell: &Shell) -> Self {
        let tool = Self::new();
        tool.init(shell);
        tool
    }
    /// Load the shell without clearing status; `tool.init(&shell)`.
    pub fn init(&self, shell: &Shell) {
        self.parent
            .fix_root_data()
            .with_mut(|fields| fields.shape = shell.clone().into_shape());
        let mut fields = self.state.write();
        fields.shell = shell.clone();
        fields.count = 0;
    }
    /// Return the latest shell; `let shell=tool.shell()`.
    pub fn shell(&self) -> Shell {
        self.state.read().shell.clone()
    }
    /// Return the shell or compound result; `let result=tool.shape()`.
    pub fn shape(&self) -> Shape {
        self.parent
            .fix_root_data()
            .with(|fields| fields.shape.clone())
    }
    /// Return the number of resulting shells; `tool.nb_shells()`.
    pub fn nb_shells(&self) -> i32 {
        self.state.read().count
    }
    /// Return faces with inconsistent orientation; `tool.error_faces()`.
    pub fn error_faces(&self) -> Compound {
        self.state.read().errors.clone()
    }
    /// Query retained native status bits; `tool.status_flag(ShapeExtendStatus::Done2)`.
    pub fn status_flag(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().status, status)
    }
    /// Borrow the shared face-repair tool; `tool.fix_face_tool()`.
    pub fn fix_face_tool(&self) -> Handle<ShapeFixFace> {
        self.face_tool.clone()
    }
}

mod build_groups;

mod multiple_faces;

mod glue;
mod non_manifold;

mod perform;
