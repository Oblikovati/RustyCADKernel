//! Message_ProgressRange: one step of a scope that advances the indicator once.
use super::ProgressScope;
use std::cell::Cell;

/// Message_ProgressRange: the part `[start, start + delta]` of the global scale that
/// a scope's `next` allocated.
///
/// Either a [`ProgressScope`] created from it takes over the advance, or the range
/// advances the indicator by `delta` when closed or dropped. A default range is not
/// bound to any indicator and is safe to pass to any algorithm.
/// Example: `lu_decompose_with_progress(&mut a, &mut indx, &mut d, 1e-20, &ProgressRange::default())`.
pub struct ProgressRange<'a> {
    parent: Option<&'a ProgressScope<'a>>,
    start: f64,
    delta: f64,
    // Native `mutable bool myWasUsed`: disarmed through shared references by copies
    // and by scopes built from `const Message_ProgressRange&`.
    was_used: Cell<bool>,
}
impl Default for ProgressRange<'_> {
    /// The empty range: no parent, start 0, delta 0; `ProgressRange::default()`.
    fn default() -> Self {
        Self {
            parent: None,
            start: 0.,
            delta: 0.,
            was_used: Cell::new(false),
        }
    }
}
impl<'a> ProgressRange<'a> {
    /// Private native constructor used by `ProgressScope::next`.
    pub(super) fn new(parent: &'a ProgressScope<'a>, start: f64, delta: f64) -> Self {
        Self {
            parent: Some(parent),
            start,
            delta,
            was_used: Cell::new(false),
        }
    }
    /// True when the indicator signals a user break; `range.user_break()`.
    pub fn user_break(&self) -> bool {
        self.parent.is_some_and(ProgressScope::indicator_break)
    }
    /// False when the indicator signals a user break; `while range.more() {}`.
    pub fn more(&self) -> bool {
        !self.user_break()
    }
    /// True while unused and bound to an indicator; `range.is_active()`.
    pub fn is_active(&self) -> bool {
        !self.was_used.get() && self.parent.is_some_and(ProgressScope::has_indicator)
    }
    /// Advances the indicator by `delta` once, then detaches; `range.close()`.
    pub fn close(&mut self) {
        if !self.is_active() {
            return;
        }
        if let Some(parent) = self.parent.take() {
            parent.increment_indicator(self.delta);
        }
        self.was_used.set(true);
    }
    /// Consumes the range for a new scope: returns `(parent, start, delta, was_used)`
    /// and disarms it, as the native scope constructors do.
    pub(super) fn disarm_for_scope(&self) -> (Option<&'a ProgressScope<'a>>, f64, f64, bool) {
        let used = self.was_used.replace(true);
        (self.parent, self.start, self.delta, used)
    }
}
impl Clone for ProgressRange<'_> {
    /// The native copy constructor: the copy takes the responsibility to advance and
    /// the source is disarmed (`myWasUsed = true`); `let task = range.clone();`.
    fn clone(&self) -> Self {
        let copy = Self {
            parent: self.parent,
            start: self.start,
            delta: self.delta,
            was_used: Cell::new(self.was_used.get()),
        };
        self.was_used.set(true);
        copy
    }
    /// The native copy assignment: overwrites this range WITHOUT closing it (its
    /// pending advance is dropped, a preserved quirk) and disarms the source;
    /// `target.clone_from(&range)`.
    fn clone_from(&mut self, source: &Self) {
        self.parent = source.parent;
        self.start = source.start;
        self.delta = source.delta;
        self.was_used.set(source.was_used.get());
        source.was_used.set(true);
    }
}
impl Drop for ProgressRange<'_> {
    /// Native destructor: `Close()`.
    fn drop(&mut self) {
        self.close();
    }
}
