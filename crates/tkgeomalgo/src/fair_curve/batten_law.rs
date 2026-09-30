// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_BattenLaw.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_BattenLaw`: the height of a linear batten along its neutral fibre.
use rustycad_tkmath::math::Function;

/// The height `H(t) = middle + (t - 0.5) * slope * sliding` of a linear batten.
///
/// `FairCurveBattenLaw::new(1.0, 0.1, 2.0).height(0.0)` is `(true, 0.9)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FairCurveBattenLaw {
    middle_heigth: f64,
    geometric_slope: f64,
    length_sliding: f64,
}

impl FairCurveBattenLaw {
    /// A law of height `heigth` at the middle, geometric `slope` and active length
    /// `sliding`; `FairCurveBattenLaw::new(1.0, 0.0, 2.0)`.
    pub fn new(heigth: f64, slope: f64, sliding: f64) -> Self {
        Self {
            middle_heigth: heigth,
            geometric_slope: slope,
            length_sliding: sliding,
        }
    }
    /// Changes the active length; `law.set_sliding(3.0)`.
    pub fn set_sliding(&mut self, sliding: f64) {
        self.length_sliding = sliding;
    }
    /// Changes the height at the middle point; `law.set_heigth(2.0)`.
    pub fn set_heigth(&mut self, heigth: f64) {
        self.middle_heigth = heigth;
    }
    /// Changes the geometric slope; `law.set_slope(0.5)`.
    pub fn set_slope(&mut self, slope: f64) {
        self.geometric_slope = slope;
    }
    /// `Value(T, THeigth)`: the height at `t` and whether it is positive;
    /// `let (positive, h) = law.height(0.5);`.
    pub fn height(&self, t: f64) -> (bool, f64) {
        let theigth: f64 =
            self.middle_heigth + (t - 0.5) * self.geometric_slope * self.length_sliding;
        (theigth > 0.0, theigth)
    }
}

impl Function for FairCurveBattenLaw {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let (positive, height) = self.height(x);
        *result = height;
        positive
    }
}
