// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr`: the package functions.
use super::general_relation::ExprGeneralRelation;
use super::handles::{ExprHandle, deref};
use super::named_unknown::ExprNamedUnknown;
use super::ru_iterator::ExprRUIterator;
use super::unknown_iterator::ExprUnknownIterator;
use rustycad_tkernel::standard::{Failure, Handle};

/// `Expr::CopyShare`: `exp` itself when it is shareable (a named expression), else
/// its copy; `let shared: ExprHandle = copy_share(&x)?;`.
pub fn copy_share(exp: &ExprHandle) -> Result<ExprHandle, Failure> {
    let expression = deref(exp, "Expr::CopyShare")?;
    if expression.is_shareable() {
        return Ok(exp.clone());
    }
    expression.copy()
}

/// `Expr::NbOfFreeVariables(expression)`: the unknowns of `exp` that are not
/// assigned; `nb_of_free_variables(&sum)? == 2`.
pub fn nb_of_free_variables(exp: &ExprHandle) -> Result<i32, Failure> {
    let mut unknowns: ExprUnknownIterator = ExprUnknownIterator::new(exp)?;
    let mut count: i32 = 0;
    while unknowns.more() {
        count += free_count(&unknowns.value()?)?;
        unknowns.next()?;
    }
    Ok(count)
}

/// `Expr::NbOfFreeVariables(relation)`: the unknowns of `rel` that are not assigned;
/// `nb_of_free_variables_relation(&equal)? == 1`.
pub fn nb_of_free_variables_relation(
    rel: &Handle<dyn ExprGeneralRelation>,
) -> Result<i32, Failure> {
    let mut unknowns: ExprRUIterator = ExprRUIterator::new(rel)?;
    let mut count: i32 = 0;
    while unknowns.more() {
        count += free_count(&unknowns.value()?)?;
        unknowns.next()?;
    }
    Ok(count)
}

/// 1 for a free unknown, 0 for an assigned one.
fn free_count(unknown: &Handle<ExprNamedUnknown>) -> Result<i32, Failure> {
    Ok(i32::from(
        !deref(unknown, "Expr::NbOfFreeVariables")?.is_assigned(),
    ))
}

/// `Expr::Sign`: `copysign(1.0, val)`, so -0.0 and negative NaN give -1;
/// `sign(-0.0) == -1.0`.
pub fn sign(val: f64) -> f64 {
    1.0_f64.copysign(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_follows_copysign() {
        assert_eq!(sign(2.0), 1.0);
        assert_eq!(sign(-0.0), -1.0);
        assert_eq!(sign(0.0), 1.0);
        assert_eq!(sign(-f64::NAN), -1.0);
    }
}
