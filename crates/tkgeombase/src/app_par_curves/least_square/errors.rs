// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_LeastSquare.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_LeastSquare::Error` and `ErrorGradient`: the squared distances of
//! the fitted points to the curves and their derivative along the parameters.
use super::LeastSquare;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;

/// `(F, MaxE3d, MaxE2d)`: the sum of the squared distances and the largest 3D and
/// 2D distances.
pub type FitErrors = (f64, f64, f64);

impl LeastSquare {
    /// `Error(F, MaxE3d, MaxE2d)`: stores each squared distance for Distance();
    /// StdFail_NotDone before a successful Perform; `fit.error()?`.
    pub fn error(&mut self) -> Result<FitErrors, Failure> {
        self.errors(None)
    }

    /// `ErrorGradient(Grad, F, MaxE3d, MaxE2d)`: also writes into `grad(i)` the
    /// derivative of the squared distances of point i along its parameter (zeroing
    /// `grad` first); `fit.error_gradient(&mut grad)?`.
    pub fn error_gradient(&mut self, grad: &mut Vector<'_>) -> Result<FitErrors, Failure> {
        grad.init(0.);
        self.errors(Some(grad))
    }

    fn errors(&mut self, mut grad: Option<&mut Vector<'_>>) -> Result<FitErrors, Failure> {
        self.check_done(if grad.is_some() {
            "ErrorGradient"
        } else {
            "Error"
        })?;
        let (mut f, mut max3d, mut max2d): (f64, f64, f64) = (0., 0., 0.);
        let mut i2: i32 = 1;
        for k in 1..=self.nb_p + self.nb_p2d {
            let width: i32 = if k <= self.nb_p { 3 } else { 2 };
            let poles: Vec<Vec<f64>> = (0..width)
                .map(|c| {
                    (1..=self.nbpoles)
                        .map(|i| self.mypoles.value(i, i2 + c).copied())
                        .collect::<Result<_, _>>()
                })
                .collect::<Result<_, _>>()?;
            for i in self.first_p..=self.last_p {
                let (fi, gr) = self.point_error(i, i2, &poles, grad.is_some())?;
                if width == 3 {
                    max3d = max3d.max_native(fi);
                } else {
                    max2d = max2d.max_native(fi);
                }
                self.the_error[(i, k)] = fi;
                if let Some(grad) = grad.as_deref_mut() {
                    *grad.change_value(i)? += gr;
                }
                f += fi;
            }
            i2 += width;
        }
        Ok((f, max3d.sqrt(), max2d.sqrt()))
    }

    // The squared distance of point i to curve column block i2 and its derivative:
    // `(Fi, 2 * sum(dC . (C - P)))`, summing on the non-null span of row i.
    fn point_error(
        &self,
        i: i32,
        i2: i32,
        poles: &[Vec<f64>],
        with_gradient: bool,
    ) -> Result<(f64, f64), Failure> {
        let width: usize = poles.len();
        let (mut value, mut derivative) = ([0.; 3], [0.; 3]);
        let indexdeb: i32 = *self.myindex.value(i)? + 1;
        for j in indexdeb..=indexdeb + self.deg {
            let aij: f64 = *self.a.value(i, j)?;
            let daij: f64 = if with_gradient {
                *self.da.value(i, j)?
            } else {
                0.
            };
            for c in 0..width {
                let pole: f64 = pole_at(&poles[c], j)?;
                value[c] += aij * pole;
                derivative[c] += daij * pole;
            }
        }
        let offsets: Vec<f64> = (0..width)
            .map(|c| Ok(value[c] - *self.mypoints.value(i, i2 + c as i32)?))
            .collect::<Result<_, Failure>>()?;
        let mut fi: f64 = offsets[0] * offsets[0] + offsets[1] * offsets[1];
        let mut gr: f64 = 2.0 * (derivative[0] * offsets[0] + derivative[1] * offsets[1]);
        if width == 3 {
            fi += offsets[2] * offsets[2];
            gr += 2.0 * derivative[2] * offsets[2];
        }
        Ok((fi, gr))
    }
}

// Px(j) of the 1-based pole column (math_Vector Px(1, nbpoles)).
fn pole_at(column: &[f64], j: i32) -> Result<f64, Failure> {
    usize::try_from(j - 1)
        .ok()
        .and_then(|slot| column.get(slot).copied())
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "AppParCurves_LeastSquare::Error: pole {j}; expected 1..={}",
                column.len()
            ))
        })
}

/// `if (Fi > MaxE) MaxE = Fi` (NaN never replaces the maximum).
trait NativeMax {
    fn max_native(self, value: f64) -> f64;
}

impl NativeMax for f64 {
    fn max_native(self, value: f64) -> f64 {
        if value > self { value } else { self }
    }
}
