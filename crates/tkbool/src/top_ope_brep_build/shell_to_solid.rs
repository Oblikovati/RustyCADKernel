// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_ShellToSolid
// (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The solids of a set of shells (`TopOpeBRepBuild_ShellToSolid`).
use super::{
    TopOpeBRepBuildBuilder, TopOpeBRepBuildShapeSet, TopOpeBRepBuildShellFaceSet,
    TopOpeBRepBuildSolidBuilder,
};
use crate::top_ope_brep_ds::TopOpeBRepDSBuildTool;
use rustycad_tkbrep::topods::{Shape, Shell, Solid};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;

/// The solids of shells (`TopOpeBRepBuild_ShellToSolid`);
/// `let mut sts = TopOpeBRepBuildShellToSolid::new();`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildShellToSolid {
    my_lsh: List<Shape>,
}

impl TopOpeBRepBuildShellToSolid {
    /// `TopOpeBRepBuild_ShellToSolid()`; `TopOpeBRepBuildShellToSolid::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Init()`: no shell; `sts.init()`.
    pub fn init(&mut self) {
        self.my_lsh.clear();
    }

    /// `AddShell(Sh)`; `sts.add_shell(&sh)`.
    pub fn add_shell(&mut self, sh: &Shell) {
        self.my_lsh.append(sh.clone().into_shape());
    }

    /// `MakeSolids(So, LSo)`: the solids of the shells classified (forced) in the shell
    /// face set of `so`, made by a builder with a default build tool;
    /// `sts.make_solids(&so, &mut l)?`.
    pub fn make_solids(&self, so: &Solid, lso: &mut List<Shape>) -> Result<(), Failure> {
        lso.clear();
        let mut sfs = TopOpeBRepBuildShellFaceSet::with_solid(so.as_ref())?;
        for sh in self.my_lsh.iter() {
            sfs.add_shape(sh)?;
        }
        let mut sb = TopOpeBRepBuildSolidBuilder::new();
        sb.init_solid_builder(&mut sfs, true)?;
        let b = TopOpeBRepBuildBuilder::new(&TopOpeBRepDSBuildTool::new());
        b.make_solids(&mut sb, lso)
    }
}
