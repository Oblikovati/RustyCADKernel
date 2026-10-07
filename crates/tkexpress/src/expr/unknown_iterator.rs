// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_UnknownIterator.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_UnknownIterator`: the named unknowns of an expression.
use super::errors::{no_more_object, out_of_range};
use super::handles::{ExprHandle, deref};
use super::named_unknown::ExprNamedUnknown;
use rustycad_tkernel::standard::{Failure, Handle};

/// Unknowns kept once each, in order of first occurrence (the native
/// `NCollection_IndexedMap` of handles, keyed by identity); `UnknownSet::default()`.
#[derive(Default)]
pub struct UnknownSet {
    unknowns: Vec<Handle<ExprNamedUnknown>>,
}
impl UnknownSet {
    /// Add `unknown` unless this very object is already there (`Contains`/`Add`);
    /// `set.add(&x)`.
    pub fn add(&mut self, unknown: &Handle<ExprNamedUnknown>) {
        if !self.unknowns.contains(unknown) {
            self.unknowns.push(unknown.clone());
        }
    }
    /// Number of unknowns (`Extent`); `set.extent()`.
    pub fn extent(&self) -> i32 {
        i32::try_from(self.unknowns.len()).unwrap_or(i32::MAX)
    }
    /// The `index`-th unknown, 1-based (`FindKey`); Standard_OutOfRange outside the
    /// set; `set.find_key(1)?`.
    pub fn find_key(&self, index: i32) -> Result<Handle<ExprNamedUnknown>, Failure> {
        usize::try_from(index)
            .ok()
            .and_then(|position| position.checked_sub(1))
            .and_then(|slot| self.unknowns.get(slot))
            .cloned()
            .ok_or_else(|| {
                out_of_range(&format!(
                    "NCollection_IndexedMap::FindKey: index = {index}; expected in [1, {}]",
                    self.unknowns.len()
                ))
            })
    }
}

/// Iterates over the unknowns of an expression, assigned ones and the unknowns of
/// their expressions included, each once, in depth-first order;
/// `let mut unknowns = ExprUnknownIterator::new(&sum)?; while unknowns.more() { ... }`.
pub struct ExprUnknownIterator {
    map: UnknownSet,
    current: i32,
}
impl ExprUnknownIterator {
    /// The unknowns of `exp` (`Expr_UnknownIterator(exp)`); `ExprUnknownIterator::new(&sum)?`.
    pub fn new(exp: &ExprHandle) -> Result<Self, Failure> {
        let mut iterator: Self = Self {
            map: UnknownSet::default(),
            current: 1,
        };
        iterator.perform(exp)?;
        Ok(iterator)
    }
    /// `Perform`: record `exp` when it is an unknown, then visit its sub-expressions.
    fn perform(&mut self, exp: &ExprHandle) -> Result<(), Failure> {
        let expression = deref(exp, "Expr_UnknownIterator::Perform")?;
        let unknown: Handle<ExprNamedUnknown> = exp.cast();
        if !unknown.is_null() {
            self.map.add(&unknown);
        }
        for index in 1..=expression.nb_sub_expressions() {
            self.perform(&expression.sub_expression(index)?)?;
        }
        Ok(())
    }
    /// True while an unknown remains (`More`); `unknowns.more()`.
    pub fn more(&self) -> bool {
        self.current <= self.map.extent()
    }
    /// Go to the next unknown (`Next`); Standard_NoMoreObject past the end;
    /// `unknowns.next()?`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, fallible.
    pub fn next(&mut self) -> Result<(), Failure> {
        if !self.more() {
            return Err(no_more_object(&format!(
                "Expr_UnknownIterator::Next: position {} past {} unknowns",
                self.current,
                self.map.extent()
            )));
        }
        self.current += 1;
        Ok(())
    }
    /// The current unknown (`Value`); Standard_OutOfRange past the end;
    /// `unknowns.value()?`.
    pub fn value(&self) -> Result<Handle<ExprNamedUnknown>, Failure> {
        self.map.find_key(self.current)
    }
}
