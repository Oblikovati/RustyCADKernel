// Rust translation of the relation, sum and product actions of
// ModelingAlgorithms/TKExpress/ExprIntrp/ExprIntrp_yaccintrf.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The semantic actions of relations (`a = b; c = d`) and of the `Sum[...]` and
//! `Prod[...]` series.
use super::context::{ExprIntrpParseContext, c_int_of};
use crate::expr::ExprGeneralExpression;
use crate::expr::handles::deref;
use crate::expr::operators::expr_add;
use crate::expr::package::copy_share;
use crate::expr::{
    ExprEqual, ExprHandle, ExprNamedUnknown, ExprNumericValue, ExprProduct, ExprRelationHandle,
    ExprSum, ExprSystemRelation,
};
use rustycad_tkernel::standard::{Failure, Handle};

/// The pieces of `Sum[e, x, first, step, count]`, popped in reverse order.
struct SeriesTerms {
    expression: ExprHandle,
    variable: Handle<ExprNamedUnknown>,
    first: ExprHandle,
    step: ExprHandle,
    count: i32,
}

impl ExprIntrpParseContext<'_> {
    /// `ExprIntrp_EndOfEqual`: the equation of the two stacked members.
    pub(super) fn end_of_equal(&mut self) -> Result<(), Failure> {
        let second: ExprHandle = self.recept.pop();
        let first: ExprHandle = self.recept.pop();
        self.recept
            .push_relation(&ExprEqual::new(&first, &second).cast());
        Ok(())
    }
    /// `ExprIntrp_EndOfRelation`: the stacked relations, the last first, gathered in a
    /// system when there are several.
    pub(super) fn end_of_relation(&mut self) -> Result<(), Failure> {
        let mut system: Handle<ExprSystemRelation> = Handle::default();
        let mut current: ExprRelationHandle = Handle::default();
        let mut previous: ExprRelationHandle = Handle::default();
        while !self.recept.is_rel_stack_empty() {
            current = self.recept.pop_relation();
            if let Some(gathered) = system.get() {
                gathered.add(&current);
            } else if !previous.is_null() {
                system = ExprSystemRelation::new(&previous);
                deref(&system, "ExprIntrp_EndOfRelation")?.add(&current);
            } else {
                previous = current.clone();
            }
        }
        let result: ExprRelationHandle = if system.is_null() {
            current
        } else {
            system.cast()
        };
        self.recept.push_relation(&result);
        Ok(())
    }
    /// Pop the five pieces of a series; the count is `(int)` of the stacked value.
    fn pop_series(&mut self) -> Result<SeriesTerms, Failure> {
        let number: Handle<ExprNumericValue> = self.recept.pop().cast();
        let count: i32 = c_int_of(deref(&number, "ExprIntrp_Sumator")?.get_value());
        let step: ExprHandle = self.recept.pop();
        let first: ExprHandle = self.recept.pop();
        let variable: Handle<ExprNamedUnknown> = self.recept.pop().cast();
        let expression: ExprHandle = self.recept.pop();
        Ok(SeriesTerms {
            expression,
            variable,
            first,
            step,
            count,
        })
    }
    /// `ExprIntrp_Sumator`: `Sum[e, x, first, step, n]`, the sum of `e` at `x = first`,
    /// `first + step`, ..., simplified one level.
    pub(super) fn sumator(&mut self) -> Result<(), Failure> {
        let members: Vec<ExprHandle> = series_members(&self.pop_series()?)?;
        let sum: ExprHandle =
            deref(&ExprSum::new(&members), "ExprIntrp_Sumator")?.shallow_simplified()?;
        self.recept.push(&sum);
        Ok(())
    }
    /// `ExprIntrp_Productor`: `Prod[e, x, first, step, n]`, the product of the same
    /// members, simplified one level.
    pub(super) fn productor(&mut self) -> Result<(), Failure> {
        let members: Vec<ExprHandle> = series_members(&self.pop_series()?)?;
        let product: ExprHandle =
            deref(&ExprProduct::new(&members), "ExprIntrp_Productor")?.shallow_simplified()?;
        self.recept.push(&product);
        Ok(())
    }
}

/// The `count` members of a series: `e` with `x` replaced by the running value, which
/// starts at a copy of `first` and grows by `step` (simplified one level each time);
/// when `e` is `x` itself the running values are the members.
fn series_members(terms: &SeriesTerms) -> Result<Vec<ExprHandle>, Failure> {
    let same: bool = terms.expression == terms.variable;
    let mut current: ExprHandle = copy_share(&terms.first)?;
    let mut members: Vec<ExprHandle> = Vec::new();
    for _ in 0..terms.count.max(0) {
        let member: ExprHandle = if same {
            current.clone()
        } else {
            let member: ExprHandle = copy_share(&terms.expression)?;
            deref(&member, "ExprIntrp_Sumator")?.replace(&terms.variable, &current)?;
            member
        };
        members.push(member);
        current =
            deref(&expr_add(&current, &terms.step), "ExprIntrp_Sumator")?.shallow_simplified()?;
    }
    Ok(members)
}
