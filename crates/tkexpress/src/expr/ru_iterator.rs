// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_RUIterator.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_RUIterator`: the named unknowns of a relation.
use super::errors::no_more_object;
use super::general_relation::ExprRelationHandle;
use super::handles::{ExprHandle, deref};
use super::named_unknown::ExprNamedUnknown;
use super::relation_iterator::ExprRelationIterator;
use super::unknown_iterator::{ExprUnknownIterator, UnknownSet};
use rustycad_tkernel::standard::{Failure, Handle};

/// Iterates over the unknowns of a relation, each once: those of the first member of
/// each single relation, then of its second member;
/// `let mut unknowns = ExprRUIterator::new(&equal)?;`.
pub struct ExprRUIterator {
    map: UnknownSet,
    current: i32,
}
impl ExprRUIterator {
    /// The unknowns of `rel` (`Expr_RUIterator(rel)`); `ExprRUIterator::new(&equal)?`.
    pub fn new(rel: &ExprRelationHandle) -> Result<Self, Failure> {
        let mut map: UnknownSet = UnknownSet::default();
        let mut relations: ExprRelationIterator = ExprRelationIterator::new(rel)?;
        while relations.more() {
            let single = relations.value()?;
            relations.next()?;
            let single = deref(&single, "Expr_RUIterator")?;
            collect(&mut map, &single.first_member())?;
            collect(&mut map, &single.second_member())?;
        }
        Ok(Self { map, current: 1 })
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
                "Expr_RUIterator::Next: position {} past {} unknowns",
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

/// Add the unknowns of `member` to `map`, in iteration order.
fn collect(map: &mut UnknownSet, member: &ExprHandle) -> Result<(), Failure> {
    let mut unknowns: ExprUnknownIterator = ExprUnknownIterator::new(member)?;
    while unknowns.more() {
        let unknown: Handle<ExprNamedUnknown> = unknowns.value()?;
        unknowns.next()?;
        map.add(&unknown);
    }
    Ok(())
}
