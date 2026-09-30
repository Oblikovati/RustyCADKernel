//! Native Gauss quadrature and uniform subdivision convergence.
use super::{Function, Vector, gauss_points, gauss_weights, stream_real};
use rustycad_tkernel::standard::Failure;
use std::fmt;
/// Single-variable Gauss integral; `GaussSingleIntegration::new(&mut f,0.0,1.0,10)`.
///
/// `Default` is the native empty constructor: not done.
#[derive(Default)]
pub struct GaussSingleIntegration {
    done: bool,
    value: f64,
}
impl GaussSingleIntegration {
    /// Caps order at 61 as OCCT does; `GaussSingleIntegration::new(&mut f,0.0,1.0,10)`.
    pub fn new(f: &mut impl Function, lower: f64, upper: f64, order: i32) -> Self {
        let mut result: Self = Self::default();
        result.perform(f, lower, upper, order.min(61));
        result
    }
    /// Subdivides at least once, up to native iteration limit; `GaussSingleIntegration::with_tolerance(&mut f,0.0,1.0,5,1e-8)`.
    pub fn with_tolerance(
        f: &mut impl Function,
        lower: f64,
        upper: f64,
        order: i32,
        tolerance: f64,
    ) -> Self {
        let order: i32 = order.min(61);
        let mut result: Self = Self::new(f, lower, upper, order);
        let mut sum: f64 = result.value;
        let mut count: i32 = 1;
        let mut iteration: i32 = 1;
        loop {
            let old: f64 = sum;
            sum = 0.0;
            count *= 2;
            let step: f64 = (upper - lower) / f64::from(count);
            for i in 1..=count {
                result.perform(
                    f,
                    lower + f64::from(i - 1) * step,
                    lower + f64::from(i) * step,
                    order,
                );
                if !result.done {
                    return result;
                }
                sum += result.value;
            }
            iteration += 1;
            if (old - sum).abs().partial_cmp(&tolerance) != Some(std::cmp::Ordering::Greater)
                || iteration > 13
            {
                break;
            }
        }
        result.value = sum;
        result
    }
    fn perform(&mut self, f: &mut impl Function, lower: f64, upper: f64, order: i32) {
        self.done = false;
        let Ok(mut points) = Vector::with_bounds(1, order) else {
            return;
        };
        let mut weights: Vector<'static> = Vector::with_bounds(1, order).unwrap();
        gauss_points(order, &mut points);
        gauss_weights(order, &mut weights);
        let middle: f64 = 0.5 * (upper + lower);
        let radius: f64 = 0.5 * (upper - lower);
        self.value = 0.0;
        if (order + 1) / 2 > order / 2 {
            if !f.value(middle, &mut self.value) {
                return;
            }
            self.value *= weights[(order + 1) / 2];
        }
        let (mut first, mut second): (f64, f64) = (0.0, 0.0);
        for index in 1..=order / 2 {
            let offset: f64 = radius * points[index];
            if !f.value(middle - offset, &mut first) || !f.value(middle + offset, &mut second) {
                return;
            }
            self.value += weights[index] * (first + second);
        }
        self.value *= radius;
        self.done = true;
    }
    /// Reports successful callbacks; `integral.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Reads a completed integral; `integral.value()?`.
    pub fn value(&self) -> Result<f64, Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(self.value)
    }
    /// Writes the native status text; `integral.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        write!(stream, "math_GaussSingleIntegration ")?;
        if !self.done {
            return writeln!(stream, "Status = not Done ");
        }
        writeln!(stream, " Status = Done ")?;
        writeln!(stream, "Integration Value = {}", stream_real(self.value))
    }
}
display_via_dump!(GaussSingleIntegration);
