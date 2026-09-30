//! Native math_Status discriminants.
/// Algorithm status independent of a solver's Done flag; `Status::TooManyIterations`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    /// Native math_OK.
    Ok = 0,
    /// Native math_TooManyIterations.
    TooManyIterations = 1,
    /// Native math_FunctionError.
    FunctionError = 2,
    /// Native math_DirectionSearchError.
    DirectionSearchError = 3,
    /// Native math_NotBracketed.
    NotBracketed = 4,
}
