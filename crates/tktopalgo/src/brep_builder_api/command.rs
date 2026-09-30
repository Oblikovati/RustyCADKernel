// Rust translation of BRepBuilderAPI_Command.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_Command`: the done flag at the root of every BRepBuilderAPI command.
use rustycad_tkernel::standard::Failure;

/// `BRepBuilderAPI_Command` (docs/topalgo-design.md section 10): the done flag of a
/// command with the native bodies of `IsDone` and `Check`. The virtual `IsDone` is
/// [`crate::brep_builder_api::BRepBuilderAPIMakeShape::is_done`]; this struct keeps the
/// base body. The native constructor and `Done`/`NotDone` are protected; they are public
/// here because the derived classes live in other crates.
///
/// ```
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPICommand;
/// let mut command = BRepBuilderAPICommand::new();
/// assert!(!command.is_done());
/// assert_eq!(command.check().unwrap_err().exception_type(), "StdFail_NotDone");
/// command.set_done();
/// assert!(command.check().is_ok());
/// ```
#[derive(Clone, Debug, Default)]
pub struct BRepBuilderAPICommand {
    my_done: bool,
}

impl BRepBuilderAPICommand {
    /// `BRepBuilderAPI_Command()` (protected): not done; `BRepBuilderAPICommand::new()`.
    pub const fn new() -> Self {
        Self { my_done: false }
    }

    /// The base body of the virtual `IsDone()`: the flag; `command.is_done()`.
    pub const fn is_done(&self) -> bool {
        self.my_done
    }

    /// `Check()`: StdFail_NotDone when the flag is false (the flag, not the virtual
    /// `IsDone()`); `command.check()?`.
    pub fn check(&self) -> Result<(), Failure> {
        if !self.my_done {
            return Err(Failure::with_message(
                "StdFail_NotDone",
                "BRep_API: command not done",
            ));
        }
        Ok(())
    }

    /// The protected `Done()`: the command succeeded; `command.set_done()`.
    pub fn set_done(&mut self) {
        self.my_done = true;
    }

    /// The protected `NotDone()`: the command failed; `command.set_not_done()`.
    pub fn set_not_done(&mut self) {
        self.my_done = false;
    }
}
