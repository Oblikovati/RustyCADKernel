// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_RelationIterator.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_RelationIterator`: the single relations of a relation, systems flattened.
use super::errors::{no_more_object, no_such_object};
use super::general_relation::ExprRelationHandle;
use super::handles::deref;
use super::single_relation::ExprSingleRelation;
use rustycad_tkernel::standard::{Failure, Handle};

/// Iterates over the single relations of a relation in order, nested systems
/// flattened; `let mut relations = ExprRelationIterator::new(&system)?;`.
pub struct ExprRelationIterator {
    relations: Vec<Handle<dyn ExprSingleRelation>>,
    current: i32,
}
impl ExprRelationIterator {
    /// The single relations of `rel` (`Expr_RelationIterator(rel)`); the array is sized
    /// by `NbOfSingleRelations` first, as natively; `ExprRelationIterator::new(&system)?`.
    pub fn new(rel: &ExprRelationHandle) -> Result<Self, Failure> {
        // The native array is sized by this count; the walk below fills it exactly.
        deref(rel, "Expr_RelationIterator")?.nb_of_single_relations()?;
        let single: Handle<dyn ExprSingleRelation> = rel.cast();
        let relations: Vec<Handle<dyn ExprSingleRelation>> = if single.is_null() {
            Self::flattened(rel)?
        } else {
            vec![single]
        };
        Ok(Self {
            relations,
            current: 1,
        })
    }
    /// The single relations of the sub-relations of `rel`, in order.
    fn flattened(rel: &ExprRelationHandle) -> Result<Vec<Handle<dyn ExprSingleRelation>>, Failure> {
        let relation = deref(rel, "Expr_RelationIterator")?;
        let mut relations: Vec<Handle<dyn ExprSingleRelation>> = Vec::new();
        for index in 1..=relation.nb_of_sub_relations() {
            let current: ExprRelationHandle = relation.sub_relation(index)?;
            let single: Handle<dyn ExprSingleRelation> = current.cast();
            if !single.is_null() {
                relations.push(single);
                continue;
            }
            let mut nested: Self = Self::new(&current)?;
            while nested.more() {
                relations.push(nested.value()?);
                nested.next()?;
            }
        }
        Ok(relations)
    }
    /// True while a relation remains (`More`); `relations.more()`.
    pub fn more(&self) -> bool {
        usize::try_from(self.current).is_ok_and(|current| current <= self.relations.len())
    }
    /// Go to the next relation (`Next`); Standard_NoMoreObject past the end;
    /// `relations.next()?`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, fallible.
    pub fn next(&mut self) -> Result<(), Failure> {
        if !self.more() {
            return Err(no_more_object(&format!(
                "Expr_RelationIterator::Next: position {} past {} relations",
                self.current,
                self.relations.len()
            )));
        }
        self.current += 1;
        Ok(())
    }
    /// The current single relation (`Value`); Standard_NoSuchObject past the end;
    /// `relations.value()?`.
    pub fn value(&self) -> Result<Handle<dyn ExprSingleRelation>, Failure> {
        usize::try_from(self.current - 1)
            .ok()
            .and_then(|slot| self.relations.get(slot))
            .cloned()
            .ok_or_else(|| {
                no_such_object(&format!(
                    "Expr_RelationIterator::Value: position {} past {} relations",
                    self.current,
                    self.relations.len()
                ))
            })
    }
}
