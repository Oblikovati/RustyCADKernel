// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_SingleRelation.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_SingleRelation`: a relation between two members.
//!
//! The members (private natively) live in an [`ExprSingleCore`] that each concrete
//! class embeds; [`single_relation_members!`] writes the inherited members.
use super::errors::out_of_range;
use super::general_relation::{ExprGeneralRelation, ExprRelationHandle};
use super::handles::{ExprHandle, deref};
use super::named_unknown::ExprNamedUnknown;
use super::unary_expression::replace_inside;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};

/// The two members of a single relation; `ExprSingleCore::default()`.
#[derive(Default)]
pub struct ExprSingleCore {
    members: SharedState<[ExprHandle; 2]>,
}

/// A relation between a first and a second member; `equal.first_member()`.
pub trait ExprSingleRelation: ExprGeneralRelation {
    /// The embedded members of the concrete object; `equal.single_core()`.
    fn single_core(&self) -> &ExprSingleCore;
    /// Replace the first member, unchecked (`SetFirstMember`); `equal.set_first_member(&y)`.
    fn set_first_member(&self, exp: &ExprHandle) {
        self.single_core().members.write()[0] = exp.clone();
    }
    /// Replace the second member, unchecked (`SetSecondMember`); `equal.set_second_member(&y)`.
    fn set_second_member(&self, exp: &ExprHandle) {
        self.single_core().members.write()[1] = exp.clone();
    }
    /// The first member (`FirstMember`); `equal.first_member()`.
    fn first_member(&self) -> ExprHandle {
        self.single_core().members.read()[0].clone()
    }
    /// The second member (`SecondMember`); `equal.second_member()`.
    fn second_member(&self) -> ExprHandle {
        self.single_core().members.read()[1].clone()
    }
}
impl_transient_interface!(
    dyn ExprSingleRelation,
    "Expr_SingleRelation",
    dyn ExprGeneralRelation
);

/// `Expr_SingleRelation::IsLinear`: both members linear, the first asked first;
/// `single_is_linear(&equal)?`.
pub fn single_is_linear<T: ExprSingleRelation + ?Sized>(me: &T) -> Result<bool, Failure> {
    if !deref(&me.first_member(), "Expr_SingleRelation::IsLinear")?.is_linear()? {
        return Ok(false);
    }
    deref(&me.second_member(), "Expr_SingleRelation::IsLinear")?.is_linear()
}

/// `Expr_SingleRelation::Contains`; `single_contains(&equal, &x)?`.
pub fn single_contains<T: ExprSingleRelation + ?Sized>(
    me: &T,
    exp: &ExprHandle,
) -> Result<bool, Failure> {
    let first: ExprHandle = me.first_member();
    let second: ExprHandle = me.second_member();
    if first == *exp || second == *exp {
        return Ok(true);
    }
    if deref(&first, "Expr_SingleRelation::Contains")?.contains(exp)? {
        return Ok(true);
    }
    deref(&second, "Expr_SingleRelation::Contains")?.contains(exp)
}

/// `Expr_SingleRelation::Replace`: the first member, then the second;
/// `single_replace(&equal, &x, &y)?`.
pub fn single_replace<T: ExprSingleRelation + ?Sized>(
    me: &T,
    var: &Handle<ExprNamedUnknown>,
    with: &ExprHandle,
) -> Result<(), Failure> {
    let first: ExprHandle = me.first_member();
    if first == *var {
        me.set_first_member(with);
    } else {
        replace_inside(&first, var, with)?;
    }
    let second: ExprHandle = me.second_member();
    if second == *var {
        me.set_second_member(with);
        return Ok(());
    }
    replace_inside(&second, var, with)
}

/// `Expr_SingleRelation::SubRelation`: a single relation has none;
/// `single_sub_relation(1)` fails with Standard_OutOfRange.
pub fn single_sub_relation(index: i32) -> Result<ExprRelationHandle, Failure> {
    Err(out_of_range(&format!(
        "Expr_SingleRelation::SubRelation: index = {index}; a single relation has no sub-relation"
    )))
}

/// The members every single relation inherits from `Expr_SingleRelation`: inside an
/// `impl ExprGeneralRelation for Class` block, `single_relation_members!();`.
macro_rules! single_relation_members {
    () => {
        fn is_linear(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
            $crate::expr::single_relation::single_is_linear(self)
        }
        fn nb_of_sub_relations(&self) -> i32 {
            0
        }
        fn nb_of_single_relations(&self) -> Result<i32, rustycad_tkernel::standard::Failure> {
            Ok(1)
        }
        fn sub_relation(
            &self,
            index: i32,
        ) -> Result<$crate::expr::ExprRelationHandle, rustycad_tkernel::standard::Failure> {
            $crate::expr::single_relation::single_sub_relation(index)
        }
        fn contains(
            &self,
            exp: &$crate::expr::ExprHandle,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            $crate::expr::single_relation::single_contains(self, exp)
        }
        fn replace(
            &self,
            var: &rustycad_tkernel::standard::Handle<$crate::expr::ExprNamedUnknown>,
            with: &$crate::expr::ExprHandle,
        ) -> Result<(), rustycad_tkernel::standard::Failure> {
            $crate::expr::single_relation::single_replace(self, var, with)
        }
    };
}
pub(crate) use single_relation_members;
