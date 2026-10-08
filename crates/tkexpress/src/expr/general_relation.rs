// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_GeneralRelation.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_GeneralRelation`: a relation between expressions (an equation or a system).
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkernel::tcollection::AsciiString;

/// `occ::handle<Expr_GeneralRelation>`; `let relation: ExprRelationHandle = equal.cast();`.
pub type ExprRelationHandle = Handle<dyn ExprGeneralRelation>;

/// A relation between expressions; every member takes `&self`, and the mutating ones
/// (`simplify`, `replace`) change the relation for every handle that shares it.
///
/// `ExprEqual::new(&x, &y).string()? == "x = y"`
pub trait ExprGeneralRelation: Transient {
    /// True when the relation holds after simplification (`IsSatisfied`);
    /// `relation.is_satisfied()?`.
    fn is_satisfied(&self) -> Result<bool, Failure>;
    /// True when every member is linear (`IsLinear`); `relation.is_linear()?`.
    fn is_linear(&self) -> Result<bool, Failure>;
    /// A new relation on simplified members (`Simplified`); `relation.simplified()?`.
    fn simplified(&self) -> Result<ExprRelationHandle, Failure>;
    /// Simplify the members in place (`Simplify`); `relation.simplify()?`.
    fn simplify(&self) -> Result<(), Failure>;
    /// A copy sharing the shareable sub-expressions (`Copy`); `relation.copy()?`.
    fn copy(&self) -> Result<ExprRelationHandle, Failure>;
    /// Number of direct sub-relations (`NbOfSubRelations`); `relation.nb_of_sub_relations()`.
    fn nb_of_sub_relations(&self) -> i32;
    /// Number of single relations, nested ones included (`NbOfSingleRelations`);
    /// `relation.nb_of_single_relations()?`.
    fn nb_of_single_relations(&self) -> Result<i32, Failure>;
    /// The `index`-th sub-relation (`SubRelation`); Standard_OutOfRange outside the
    /// sub-relations; `relation.sub_relation(1)?`.
    fn sub_relation(&self, index: i32) -> Result<ExprRelationHandle, Failure>;
    /// True when `exp` occurs in the relation (`Contains`); `relation.contains(&x)?`.
    fn contains(&self, exp: &ExprHandle) -> Result<bool, Failure>;
    /// Replace every occurrence of `var` with `with` (`Replace`);
    /// `relation.replace(&x, &value)?`.
    fn replace(&self, var: &Handle<ExprNamedUnknown>, with: &ExprHandle) -> Result<(), Failure>;
    /// The readable text of the relation (`String`); `relation.string()?`.
    fn string(&self) -> Result<AsciiString, Failure>;
}
impl_transient_interface!(
    dyn ExprGeneralRelation,
    "Expr_GeneralRelation",
    StandardTransient
);
