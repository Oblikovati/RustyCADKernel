//! Gauss-Kronrod integration, native error estimates and adaptive interval splitting.
use super::{Function, Vector, kronrod_points_and_weights, ordered_gauss_points_and_weights};
mod adaptive;
mod results;
mod rule;
struct Quadrature {
    gp: Vector<'static>,
    gw: Vector<'static>,
    kp: Vector<'static>,
    kw: Vector<'static>,
}
/// Native integration result and error estimates; `KronrodSingleIntegration::new(&mut f,0.0,1.0,15)`.
#[derive(Default)]
pub struct KronrodSingleIntegration {
    done: bool,
    value: f64,
    error: f64,
    absolute: f64,
    order: i32,
    iterations: i32,
}
impl Quadrature {
    fn new(order: i32) -> Option<Self> {
        let mut q: Self = Self {
            gp: Vector::with_bounds(1, order / 2).ok()?,
            gw: Vector::with_bounds(1, order / 2).ok()?,
            kp: Vector::with_bounds(1, order).ok()?,
            kw: Vector::with_bounds(1, order).ok()?,
        };
        if !kronrod_points_and_weights(order, &mut q.kp, &mut q.kw)
            || !ordered_gauss_points_and_weights(order / 2, &mut q.gp, &mut q.gw)
        {
            return None;
        }
        Some(q)
    }
    fn evaluate(
        &self,
        f: &mut impl Function,
        bounds: [f64; 2],
        value: &mut f64,
        error: &mut f64,
    ) -> bool {
        KronrodSingleIntegration::gk_rule(
            f, bounds[0], bounds[1], &self.gp, &self.gw, &self.kp, &self.kw, value, error,
        )
    }
}
impl KronrodSingleIntegration {
    /// Performs a single rule with odd point count; `KronrodSingleIntegration::new(&mut f,0.0,1.0,15)`.
    pub fn new(f: &mut impl Function, lower: f64, upper: f64, order: i32) -> Self {
        let mut result: Self = Self::default();
        result.perform(f, lower, upper, order);
        result
    }
    /// Runs the native relative-error refinement; `KronrodSingleIntegration::with_tolerance(&mut f,0.0,1.0,15,1e-8,100)`.
    pub fn with_tolerance(
        f: &mut impl Function,
        lower: f64,
        upper: f64,
        order: i32,
        tolerance: f64,
        limit: i32,
    ) -> Self {
        let mut result: Self = Self::default();
        result.perform_with_tolerance(f, lower, upper, order, tolerance, limit);
        result
    }
    /// Reuses the result and rejects reversed or too-short intervals; `integral.perform(&mut f,0.0,1.0,15)`.
    pub fn perform(&mut self, f: &mut impl Function, lower: f64, upper: f64, order: i32) {
        self.iterations = 0;
        if order < 3 || upper - lower < 1e-9 {
            self.done = false;
            return;
        }
        self.order = if order % 2 == 0 { order + 1 } else { order };
        self.error = f64::MAX;
        let Some(q) = Quadrature::new(self.order) else {
            self.done = false;
            return;
        };
        self.done = q.evaluate(f, [lower, upper], &mut self.value, &mut self.error);
        if !self.done {
            return;
        }
        self.absolute = self.error;
        self.iterations += 1;
    }
    /// Refines the largest-error interval, preserving native early returns; `integral.perform_with_tolerance(&mut f,0.0,1.0,15,1e-8,100)`.
    pub fn perform_with_tolerance(
        &mut self,
        f: &mut impl Function,
        lower: f64,
        upper: f64,
        order: i32,
        tolerance: f64,
        limit: i32,
    ) {
        self.iterations = 0;
        if order < 3 || tolerance <= 0.0 {
            self.done = false;
            return;
        }
        self.order = if order % 2 == 0 { order + 1 } else { order };
        let Some(q) = Quadrature::new(self.order) else {
            self.done = false;
            return;
        };
        self.done = q.evaluate(f, [lower, upper], &mut self.value, &mut self.error);
        if !self.done {
            return;
        }
        self.absolute = self.error;
        if self.value.abs() > super::real_epsilon(1.0) {
            self.error /= self.value.abs();
        }
        self.iterations += 1;
        if self.error <= tolerance || self.iterations >= limit {
            return;
        }
        self.refine(f, &q, [lower, upper], tolerance, limit);
    }
}
