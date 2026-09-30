//! Explicit derivative capabilities replace the native dynamic_cast dispatch.
use super::{
    Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, MultipleVarFunctionWithHessian,
    Vector,
};
/// Selects the native objective capabilities; `GlobalObjective::Hessian(&mut function)`.
pub enum GlobalObjective<'a> {
    /// Objective without derivatives; `GlobalObjective::Value(&mut f)`.
    Value(&'a mut dyn MultipleVarFunction),
    /// Objective with first derivatives; `GlobalObjective::Gradient(&mut f)`.
    Gradient(&'a mut dyn MultipleVarFunctionWithGradient),
    /// Objective with first and second derivatives; `GlobalObjective::Hessian(&mut f)`.
    Hessian(&'a mut dyn MultipleVarFunctionWithHessian),
}
impl GlobalObjective<'_> {
    pub(super) fn has_hessian(&self) -> bool {
        matches!(self, Self::Hessian(_))
    }
    pub(super) fn has_gradient(&self) -> bool {
        !matches!(self, Self::Value(_))
    }
}
impl MultipleVarFunction for GlobalObjective<'_> {
    fn variables(&self) -> i32 {
        match self {
            Self::Value(f) => f.variables(),
            Self::Gradient(f) => f.variables(),
            Self::Hessian(f) => f.variables(),
        }
    }
    fn value(&mut self, p: &Vector<'_>, v: &mut f64) -> bool {
        match self {
            Self::Value(f) => f.value(p, v),
            Self::Gradient(f) => f.value(p, v),
            Self::Hessian(f) => f.value(p, v),
        }
    }
    fn state_number(&mut self) -> i32 {
        match self {
            Self::Value(f) => f.state_number(),
            Self::Gradient(f) => f.state_number(),
            Self::Hessian(f) => f.state_number(),
        }
    }
}
impl MultipleVarFunctionWithGradient for GlobalObjective<'_> {
    fn gradient(&mut self, p: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        match self {
            Self::Value(_) => false,
            Self::Gradient(f) => f.gradient(p, g),
            Self::Hessian(f) => f.gradient(p, g),
        }
    }
    fn values(&mut self, p: &Vector<'_>, v: &mut f64, g: &mut Vector<'_>) -> bool {
        match self {
            Self::Value(_) => false,
            Self::Gradient(f) => f.values(p, v, g),
            Self::Hessian(f) => f.values(p, v, g),
        }
    }
}
impl MultipleVarFunctionWithHessian for GlobalObjective<'_> {
    fn values_hessian(
        &mut self,
        p: &Vector<'_>,
        v: &mut f64,
        g: &mut Vector<'_>,
        h: &mut Matrix<'_>,
    ) -> bool {
        match self {
            Self::Hessian(f) => f.values_hessian(p, v, g, h),
            _ => false,
        }
    }
}
