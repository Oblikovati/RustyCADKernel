//! CSLib_NormalPolyDef, the trigonometric polynomial whose sign decides a singular normal.
use crate::math::{Function, FunctionWithDerivative};
use crate::plib::bin as plib_bin;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// `F(X) = Sum_{i=0}^{k0} C(k0,i) cos^i(X) sin^(k0-i)(X) li(i)` and its derivative.
///
/// `NormalPolyDef::new(1, &coefficients)?` with `coefficients` indexed `0..=1`.
#[derive(Clone, Debug, PartialEq)]
pub struct NormalPolyDef {
    k0: i32,
    coefficients: Vec<f64>,
    // PLib::Bin(k0, i), fetched at construction; see `new` for why.
    binomials: Vec<f64>,
}

impl NormalPolyDef {
    /// Copies `li(0..=k0)`; `NormalPolyDef::new(2, &li)?`.
    ///
    /// Native OCCT raises Standard_OutOfRange from `PLib::Bin` on the first
    /// evaluation when `k0 > 25`. The Rust evaluation callbacks return `bool`
    /// and cannot carry that exception, so the same failure is raised here.
    /// Reading `li` outside its bounds fails as NCollection_Array1 does.
    pub fn new(k0: i32, li: &Array1<'_, f64>) -> Result<Self, Failure> {
        if k0 < -1 {
            return Err(Failure::with_type(
                "Standard_RangeError",
                &format!("CSLib_NormalPolyDef degree {k0}: expected k0 >= -1"),
                "",
            ));
        }
        let coefficients: Vec<f64> = (0..=k0)
            .map(|index| read_coefficient(li, index))
            .collect::<Result<Vec<f64>, Failure>>()?;
        let binomials: Vec<f64> = (0..=k0)
            .map(|index| plib_bin(k0, index))
            .collect::<Result<Vec<f64>, Failure>>()?;
        Ok(Self {
            k0,
            coefficients,
            binomials,
        })
    }

    // The native term `C(k0,i) * cos^i * sin^(k0-i) * li(i)`, multiplied left to right.
    fn value_term(&self, index: usize, cos: f64, sin: f64) -> f64 {
        let i: f64 = index as f64;
        self.binomials[index]
            * cos.powf(i)
            * sin.powf(f64::from(self.k0) - i)
            * self.coefficients[index]
    }

    // The native derivative term `C(k0,i) * cos^(i-1) * sin^(k0-i-1) * (k0 cos^2 - i) * li(i)`.
    fn derivative_term(&self, index: usize, cos: f64, sin: f64) -> f64 {
        let i: f64 = index as f64;
        let k0: f64 = f64::from(self.k0);
        self.binomials[index]
            * cos.powf(i - 1.0)
            * sin.powf(k0 - i - 1.0)
            * (k0 * cos * cos - i)
            * self.coefficients[index]
    }
}

fn read_coefficient(li: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    li.value(index).copied().map_err(|error| {
        Failure::with_type(
            "Standard_OutOfRange",
            &format!("CSLib_NormalPolyDef coefficient {error}"),
            "",
        )
    })
}

/// Returns `(cos X, sin X)`, or `None` at the singular points where native OCCT
/// returns zero to avoid `0^n` terms (|cos| or |sin| <= RealSmall()).
fn regular_trigonometry(x: f64) -> Option<(f64, f64)> {
    let cos: f64 = x.cos();
    let sin: f64 = x.sin();
    if cos.abs() <= f64::MIN_POSITIVE || sin.abs() <= f64::MIN_POSITIVE {
        return None;
    }
    Some((cos, sin))
}

impl Function for NormalPolyDef {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = 0.0;
        let Some((cos, sin)) = regular_trigonometry(x) else {
            return true;
        };
        for index in 0..self.coefficients.len() {
            *result += self.value_term(index, cos, sin);
        }
        true
    }
}

impl FunctionWithDerivative for NormalPolyDef {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = 0.0;
        let Some((cos, sin)) = regular_trigonometry(x) else {
            return true;
        };
        for index in 0..self.coefficients.len() {
            *derivative += self.derivative_term(index, cos, sin);
        }
        true
    }

    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = 0.0;
        *derivative = 0.0;
        let Some((cos, sin)) = regular_trigonometry(x) else {
            return true;
        };
        // Native interleaves the two sums in one loop; each sum keeps its own order.
        for index in 0..self.coefficients.len() {
            *value += self.value_term(index, cos, sin);
            *derivative += self.derivative_term(index, cos, sin);
        }
        true
    }
}
