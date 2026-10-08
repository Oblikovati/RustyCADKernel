// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_SystemRelation.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_SystemRelation`: a system of relations.
use super::errors::{dimension_mismatch, no_such_object, out_of_range};
use super::general_relation::{ExprGeneralRelation, ExprRelationHandle};
use super::handles::{ExprHandle, deref};
use super::named_unknown::ExprNamedUnknown;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// A system of relations, in order; `ExprSystemRelation::new(&equal).nb_of_sub_relations() == 1`.
pub struct ExprSystemRelation {
    parent: StandardTransient,
    relations: SharedState<Vec<ExprRelationHandle>>,
}
impl_transient!(
    ExprSystemRelation,
    "Expr_SystemRelation",
    dyn ExprGeneralRelation,
    parent,
    interfaces: [dyn ExprGeneralRelation]
);
impl ExprSystemRelation {
    /// `new Expr_SystemRelation(relation)`: a system of one relation;
    /// `ExprSystemRelation::new(&equal)`.
    pub fn new(relation: &ExprRelationHandle) -> Handle<Self> {
        Handle::new(Self {
            parent: StandardTransient::default(),
            relations: SharedState::new(vec![relation.clone()]),
        })
    }
    /// Append a relation (`Add`); `system.add(&equal)`.
    pub fn add(&self, relation: &ExprRelationHandle) {
        self.relations.write().push(relation.clone());
    }
    /// `Remove`, which never removes anything natively.
    ///
    /// Native quirk preserved: the test is inverted, so a relation of the system raises
    /// Standard_NoSuchObject; any other relation raises Standard_DimensionMismatch on a
    /// system of one relation and Standard_OutOfRange (position 0) otherwise;
    /// `system.remove(&other)` is always an error.
    pub fn remove(&self, relation: &ExprRelationHandle) -> Result<(), Failure> {
        let relations = self.relations.read();
        if let Some(position) = relations.iter().position(|member| member == relation) {
            return Err(no_such_object(&format!(
                "Expr_SystemRelation::Remove: the relation is at position {}",
                position + 1
            )));
        }
        if relations.len() <= 1 {
            return Err(dimension_mismatch(&format!(
                "Expr_SystemRelation::Remove: {} relation; expected more than 1",
                relations.len()
            )));
        }
        Err(out_of_range(&format!(
            "Expr_SystemRelation::Remove: position 0; expected in [1, {}]",
            relations.len()
        )))
    }
    /// The relations, copied out of the lock so that calls on them cannot meet it.
    fn relation_list(&self) -> Vec<ExprRelationHandle> {
        self.relations.read().clone()
    }
    /// The first relation, which `Simplified` and `Copy` read unconditionally.
    fn first(&self) -> Result<ExprRelationHandle, Failure> {
        self.sub_relation(1)
    }
}
/// Borrow a relation of the system, or fail as the native null dereference does.
fn relation_ref<'h>(
    relation: &'h ExprRelationHandle,
    member: &str,
) -> Result<&'h dyn ExprGeneralRelation, Failure> {
    deref(relation, member)
}
impl ExprGeneralRelation for ExprSystemRelation {
    fn is_satisfied(&self) -> Result<bool, Failure> {
        for relation in self.relation_list() {
            if !relation_ref(&relation, "Expr_SystemRelation::IsSatisfied")?.is_satisfied()? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        for relation in self.relation_list() {
            if !relation_ref(&relation, "Expr_SystemRelation::IsLinear")?.is_linear()? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn simplified(&self) -> Result<ExprRelationHandle, Failure> {
        let first: ExprRelationHandle =
            relation_ref(&self.first()?, "Expr_SystemRelation::Simplified")?.simplified()?;
        let result: Handle<ExprSystemRelation> = Self::new(&first);
        let system: &ExprSystemRelation = deref(&result, "Expr_SystemRelation::Simplified")?;
        for relation in self.relation_list().iter().skip(1) {
            system.add(&relation_ref(relation, "Expr_SystemRelation::Simplified")?.simplified()?);
        }
        Ok(result.cast())
    }
    fn simplify(&self) -> Result<(), Failure> {
        for relation in self.relation_list() {
            relation_ref(&relation, "Expr_SystemRelation::Simplify")?.simplify()?;
        }
        Ok(())
    }
    fn copy(&self) -> Result<ExprRelationHandle, Failure> {
        let first: ExprRelationHandle =
            relation_ref(&self.first()?, "Expr_SystemRelation::Copy")?.copy()?;
        let result: Handle<ExprSystemRelation> = Self::new(&first);
        let system: &ExprSystemRelation = deref(&result, "Expr_SystemRelation::Copy")?;
        for relation in self.relation_list().iter().skip(1) {
            system.add(&relation_ref(relation, "Expr_SystemRelation::Copy")?.copy()?);
        }
        Ok(result.cast())
    }
    fn nb_of_sub_relations(&self) -> i32 {
        i32::try_from(self.relations.read().len()).unwrap_or(i32::MAX)
    }
    fn nb_of_single_relations(&self) -> Result<i32, Failure> {
        let mut count: i32 = 0;
        for relation in self.relation_list() {
            count += relation_ref(&relation, "Expr_SystemRelation::NbOfSingleRelations")?
                .nb_of_single_relations()?;
        }
        Ok(count)
    }
    fn sub_relation(&self, index: i32) -> Result<ExprRelationHandle, Failure> {
        let relations = self.relations.read();
        usize::try_from(index)
            .ok()
            .and_then(|position| position.checked_sub(1))
            .and_then(|slot| relations.get(slot))
            .cloned()
            .ok_or_else(|| {
                out_of_range(&format!(
                    "Expr_SystemRelation::SubRelation: index = {index}; expected in [1, {}]",
                    relations.len()
                ))
            })
    }
    fn contains(&self, exp: &ExprHandle) -> Result<bool, Failure> {
        for relation in self.relation_list() {
            if relation_ref(&relation, "Expr_SystemRelation::Contains")?.contains(exp)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn replace(&self, var: &Handle<ExprNamedUnknown>, with: &ExprHandle) -> Result<(), Failure> {
        for relation in self.relation_list() {
            relation_ref(&relation, "Expr_SystemRelation::Replace")?.replace(var, with)?;
        }
        Ok(())
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        let mut text: AsciiString = AsciiString::new("");
        for (index, relation) in self.relation_list().iter().enumerate() {
            if index > 0 {
                text += b'\n';
            }
            text += &relation_ref(relation, "Expr_SystemRelation::String")?.string()?;
        }
        Ok(text)
    }
}
