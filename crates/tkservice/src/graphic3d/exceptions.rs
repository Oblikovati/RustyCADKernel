//! The exception classes of the Graphic3d package, as [`Failure`] values.
//!
//! Each `DEFINE_STANDARD_EXCEPTION(Graphic3d_*Error, ...)` becomes one constructor; the
//! failure keeps the most derived class name, which `Failure::exception_type` reports.
use rustycad_tkernel::standard::Failure;

/// `Graphic3d_GroupDefinitionError`; `group_definition_error("removed group")`.
pub fn group_definition_error(message: &str) -> Failure {
    Failure::with_type("Graphic3d_GroupDefinitionError", message, "")
}
/// `Graphic3d_PriorityDefinitionError`; `priority_definition_error("bad priority 12")`.
pub fn priority_definition_error(message: &str) -> Failure {
    Failure::with_type("Graphic3d_PriorityDefinitionError", message, "")
}
/// `Graphic3d_StructureDefinitionError`; `structure_definition_error("bad structure")`.
pub fn structure_definition_error(message: &str) -> Failure {
    Failure::with_type("Graphic3d_StructureDefinitionError", message, "")
}
