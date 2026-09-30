// Rust translation of BRepLib_Command.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_Command`: the done flag at the root of every BRepLib maker.
use super::errors::not_done;
use rustycad_tkernel::standard::Failure;

/// `BRepLib_Command` (docs/topalgo-design.md section 7): the done flag of a
/// construction. The native constructor is protected; the makers build it.
///
/// ```
/// use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;
/// let maker = BRepLibMakeEdge::new();
/// assert!(!maker.is_done());
/// assert_eq!(maker.check().unwrap_err().exception_type(), "StdFail_NotDone");
/// ```
#[derive(Clone, Debug, Default)]
pub struct BRepLibCommand {
    my_done: bool,
}

impl BRepLibCommand {
    /// `BRepLib_Command()`: not done.
    pub(crate) const fn new() -> Self {
        Self { my_done: false }
    }

    /// `IsDone()`; `maker.is_done()`.
    pub const fn is_done(&self) -> bool {
        self.my_done
    }

    /// `Check()`: StdFail_NotDone when the command is not done; `maker.check()?`.
    pub fn check(&self) -> Result<(), Failure> {
        if !self.my_done {
            return Err(not_done("BRep_API: command not done"));
        }
        Ok(())
    }

    /// The protected `Done()`: the command succeeded.
    pub(crate) fn set_done(&mut self) {
        self.my_done = true;
    }

    /// The protected `NotDone()`: the command failed.
    pub(crate) fn set_not_done(&mut self) {
        self.my_done = false;
    }
}
