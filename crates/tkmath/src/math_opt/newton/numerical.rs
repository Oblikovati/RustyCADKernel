//! Numerical derivative adapters from MathOpt_Newton.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use crate::math_utils::{numerical_gradient_adaptive, numerical_hessian};
struct HessianAdapter<'a, F> {
    function: &'a mut F,
    step: f64,
}
impl<F: MultipleVarFunctionWithGradient> MultipleVarFunction for HessianAdapter<'_, F> {
    fn variables(&self) -> i32 {
        self.function.variables()
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        self.function.value(x, value)
    }
}
impl<F: MultipleVarFunctionWithGradient> MultipleVarFunctionWithGradient for HessianAdapter<'_, F> {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        self.function.gradient(x, g)
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
impl<F: MultipleVarFunctionWithGradient> HessianFunction for HessianAdapter<'_, F> {
    fn hessian(&mut self, x: &Vector<'_>, h: &mut Matrix<'_>) -> bool {
        let mut copy: Vector<'static> = zero_based(x);
        numerical_hessian(self.function, &mut copy, h, self.step)
    }
}
/// Uses direct objective differences for the Hessian; `newton_numerical_hessian(&mut f,&start,1e-6,&config)`.
pub fn newton_numerical_hessian(
    f: &mut impl MultipleVarFunctionWithGradient,
    start: &Vector<'_>,
    step: f64,
    cfg: &NewtonConfig,
) -> VectorResult {
    if !step.is_finite() || step <= 0.0 {
        return VectorResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    newton(&mut HessianAdapter { function: f, step }, start, cfg)
}
struct DerivativesAdapter<'a, F> {
    function: &'a mut F,
    gradient_step: f64,
    hessian_step: f64,
}
impl<F: MultipleVarFunction> MultipleVarFunction for DerivativesAdapter<'_, F> {
    fn variables(&self) -> i32 {
        self.function.variables()
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        self.function.value(x, value)
    }
}
impl<F: MultipleVarFunction> MultipleVarFunctionWithGradient for DerivativesAdapter<'_, F> {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        let mut copy: Vector<'static> = zero_based(x);
        numerical_gradient_adaptive(self.function, &mut copy, g, self.gradient_step)
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
fn symmetrize(h: &mut Matrix<'_>) {
    for i in 0..h.row_number() {
        for j in i + 1..h.col_number() {
            let average: f64 = 0.5 * (h[(i, j)] + h[(j, i)]);
            h[(i, j)] = average;
            h[(j, i)] = average;
        }
    }
}
impl<F: MultipleVarFunction> HessianFunction for DerivativesAdapter<'_, F> {
    fn hessian(&mut self, x: &Vector<'_>, h: &mut Matrix<'_>) -> bool {
        let mut copy: Vector<'static> = zero_based(x);
        let n: i32 = x.length();
        let mut plus: Vector<'static> = Vector::with_size(n as usize);
        let mut minus: Vector<'static> = Vector::with_size(n as usize);
        for j in 0..n {
            let original: f64 = copy[j];
            copy[j] = original + self.hessian_step;
            if !numerical_gradient_adaptive(self.function, &mut copy, &mut plus, self.gradient_step)
            {
                return false;
            }
            copy[j] = original - self.hessian_step;
            if !numerical_gradient_adaptive(
                self.function,
                &mut copy,
                &mut minus,
                self.gradient_step,
            ) {
                return false;
            }
            copy[j] = original;
            for i in 0..n {
                *h.change_at(i as usize, j as usize).unwrap() =
                    (plus[i] - minus[i]) / (2.0 * self.hessian_step);
            }
        }
        symmetrize(h);
        true
    }
}
/// Differences numerical gradients for the Hessian; `newton_numerical(&mut f,&start,1e-8,1e-6,&config)`.
pub fn newton_numerical(
    f: &mut impl MultipleVarFunction,
    start: &Vector<'_>,
    gradient_step: f64,
    hessian_step: f64,
    cfg: &NewtonConfig,
) -> VectorResult {
    if !gradient_step.is_finite()
        || gradient_step <= 0.0
        || !hessian_step.is_finite()
        || hessian_step <= 0.0
    {
        return VectorResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    newton(
        &mut DerivativesAdapter {
            function: f,
            gradient_step,
            hessian_step,
        },
        start,
        cfg,
    )
}
