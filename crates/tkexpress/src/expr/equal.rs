// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Equal.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Equal`: the equation `first = second`.
use super::general_relation::{ExprGeneralRelation, ExprRelationHandle};
use super::handles::{ExprHandle, deref};
use super::package::copy_share;
use super::single_relation::{ExprSingleCore, ExprSingleRelation, single_relation_members};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// An equation; `ExprEqual::new(&x, &y).string()? == "x = y"`.
pub struct ExprEqual {
    parent: StandardTransient,
    core: ExprSingleCore,
}
impl_transient!(
    ExprEqual,
    "Expr_Equal",
    dyn ExprSingleRelation,
    parent,
    interfaces: [dyn ExprGeneralRelation, dyn ExprSingleRelation]
);
impl ExprEqual {
    /// `new Expr_Equal(exp1, exp2)`; `ExprEqual::new(&x, &y)`.
    pub fn new(exp1: &ExprHandle, exp2: &ExprHandle) -> Handle<Self> {
        let equal: Self = Self {
            parent: StandardTransient::default(),
            core: ExprSingleCore::default(),
        };
        equal.set_first_member(exp1);
        equal.set_second_member(exp2);
        Handle::new(equal)
    }
    /// Both members simplified, the first first.
    fn simplified_members(&self) -> Result<(ExprHandle, ExprHandle), Failure> {
        let first: ExprHandle = deref(&self.first_member(), "Expr_Equal")?.simplified()?;
        let second: ExprHandle = deref(&self.second_member(), "Expr_Equal")?.simplified()?;
        Ok((first, second))
    }
}
impl ExprSingleRelation for ExprEqual {
    fn single_core(&self) -> &ExprSingleCore {
        &self.core
    }
}
impl ExprGeneralRelation for ExprEqual {
    single_relation_members!();
    fn is_satisfied(&self) -> Result<bool, Failure> {
        let (first, second) = self.simplified_members()?;
        deref(&first, "Expr_Equal::IsSatisfied")?.is_identical(&second)
    }
    fn simplified(&self) -> Result<ExprRelationHandle, Failure> {
        let (first, second) = self.simplified_members()?;
        Ok(Self::new(&first, &second).cast())
    }
    fn simplify(&self) -> Result<(), Failure> {
        let first: ExprHandle =
            deref(&self.first_member(), "Expr_Equal::Simplify")?.simplified()?;
        self.set_first_member(&first);
        let second: ExprHandle =
            deref(&self.second_member(), "Expr_Equal::Simplify")?.simplified()?;
        self.set_second_member(&second);
        Ok(())
    }
    fn copy(&self) -> Result<ExprRelationHandle, Failure> {
        let first: ExprHandle = copy_share(&self.first_member())?;
        Ok(Self::new(&first, &copy_share(&self.second_member())?).cast())
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        let mut text: AsciiString = deref(&self.first_member(), "Expr_Equal::String")?.string()?;
        text += " = ";
        text += &deref(&self.second_member(), "Expr_Equal::String")?.string()?;
        Ok(text)
    }
}
