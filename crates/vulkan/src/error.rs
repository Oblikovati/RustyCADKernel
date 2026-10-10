//! Errors name the rejected input or operation and its expected contract.
use std::fmt;
/// A renderer input, capability or device failure; inspect with `error.to_string()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderError(pub(crate) String);
impl RenderError {
    /// Describe a rejected backend input; `RenderError::rejected(0, "positive extent")`.
    pub fn rejected(value: impl fmt::Debug, expected: &str) -> Self {
        rejected(value, expected)
    }
}
impl fmt::Display for RenderError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str(&self.0)
    }
}
impl std::error::Error for RenderError {}
pub(crate) fn rejected(value: impl fmt::Debug, expected: &str) -> RenderError {
    RenderError(format!("received {value:?}; expected {expected}"))
}
