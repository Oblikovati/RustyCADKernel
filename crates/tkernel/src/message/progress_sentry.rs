//! Message_ProgressSentry: the deprecated scope constructor with min/step/span checks.
use super::{ProgressRange, ProgressScope};
use crate::{standard::Failure, tcollection::AsciiString};
use std::ops::Deref;

/// Message_ProgressSentry (deprecated upstream): a [`ProgressScope`] built through the
/// legacy `(min, max, step)` signature, which only accepts `min == 0`, `step == 1`
/// and a zero new-scope span. Public inheritance becomes `Deref`.
/// Example: `let sentry = ProgressSentry::new(&range, "Legacy", 0., 10., 1., false, 0.)?;`.
pub struct ProgressSentry<'a> {
    scope: ProgressScope<'a>,
}
impl<'a> ProgressSentry<'a> {
    /// Builds the scope (copying `name`), then refuses legacy parameters with
    /// `Standard_ProgramError`.
    ///
    /// Preserved quirk: natively the base scope is constructed before the check, so
    /// a refused sentry has already disarmed `range` and its destruction during
    /// unwinding closes the scope, advancing the indicator by the whole range.
    /// `ProgressSentry::new(&range, "Legacy", 0., 10., 1., false, 0.)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        range: &ProgressRange<'a>,
        name: &str,
        min: f64,
        max: f64,
        step: f64,
        infinite: bool,
        new_scope_span: f64,
    ) -> Result<Self, Failure> {
        let scope = ProgressScope::new(range, &AsciiString::new(name), max, infinite);
        if min != 0.0 || step != 1.0 || new_scope_span != 0.0 {
            drop(scope);
            return Err(Failure::with_type(
                "Standard_ProgramError",
                &format!(
                    "Message_ProgressSentry, invalid parameters: min {min}, step {step}, \
                     new scope span {new_scope_span}; expected 0, 1 and 0"
                ),
                "",
            ));
        }
        Ok(Self { scope })
    }
    /// Legacy name of `close`; `sentry.relieve()`.
    pub fn relieve(&self) {
        self.scope.close();
    }
}
impl<'a> Deref for ProgressSentry<'a> {
    type Target = ProgressScope<'a>;
    fn deref(&self) -> &ProgressScope<'a> {
        &self.scope
    }
}
