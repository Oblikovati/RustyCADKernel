//! `FillInitialVectorOfSolution`: the start point of the solver on the parametric
//! surface, brought into its bounds by periods.
use super::ApproxIntImpPrmSvSurfaces;
use crate::approx_int::surface_tool::ApproxIntSurfaceTool;
use crate::approx_int::sv_surfaces::ApproxIntSvSurfaces;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::math::Vector;

/// The margin of the bound tests (`0.0000000001`).
const BOUND_EPS: f64 = 0.0000000001;

/// The bounds of the parametric surface of `FillInitialVectorOfSolution`.
///
/// ```
/// use rustycad_tkgeomalgo::approx_int::PatchBounds;
/// let b = PatchBounds { binfu: 0.0, bsupu: 1.0, binfv: 0.0, bsupv: 1.0 };
/// assert_eq!(b.bsupu, 1.0);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PatchBounds {
    /// `binfu`.
    pub binfu: f64,
    /// `bsupu`.
    pub bsupu: f64,
    /// `binfv`.
    pub binfv: f64,
    /// `bsupv`.
    pub bsupv: f64,
}

/// The translation by periods that brings `value` into `[low, high]` (with the margin):
/// Some(0) inside, None outside a non-periodic direction. `period` is asked only for a
/// value outside, as the native code asks the surface. The negated tests keep the
/// native `do ... while` exits for a NaN.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn translation(
    value: f64,
    (low, high): (f64, f64),
    period: impl FnOnce() -> Result<Option<f64>, Failure>,
) -> Result<Option<f64>, Failure> {
    let mut shift: f64 = 0.0;
    if value < low - BOUND_EPS {
        let Some(d) = period()? else {
            return Ok(None);
        };
        loop {
            shift += d;
            if !(value + shift < low) {
                break;
            }
        }
    } else if value > high + BOUND_EPS {
        let Some(d) = period()? else {
            return Ok(None);
        };
        loop {
            shift -= d;
            if !(value + shift > high) {
                break;
            }
        }
    }
    Ok(Some(shift))
}

impl<T: ApproxIntSurfaceTool> ApproxIntImpPrmSvSurfaces<T> {
    /// `FillInitialVectorOfSolution(u1, v1, u2, v2, binfu, bsupu, binfv, bsupv, X,
    /// TranslationU, TranslationV)`: `X` gets the parameters on the parametric surface
    /// (`u2, v2` when the implicit surface is the first one) translated into its bounds,
    /// moved off the bounds when the solver is used; the translations, or None when a
    /// parameter is outside a non-periodic direction.
    pub fn fill_initial_vector_of_solution(
        &self,
        uv: [f64; 4],
        bounds: PatchBounds,
        x: &mut Vector<'_>,
    ) -> Result<Option<(f64, f64)>, Failure> {
        let ps = self.p_surface()?;
        let (u, v) = if self.my_implicit_first {
            (uv[2], uv[3])
        } else {
            (uv[0], uv[1])
        };
        let u_period = || -> Result<Option<f64>, Failure> {
            Ok(if T::is_u_periodic(ps)? {
                Some(T::u_period(ps)?)
            } else {
                None
            })
        };
        let Some(tu) = translation(u, (bounds.binfu, bounds.bsupu), u_period)? else {
            return Ok(None);
        };
        let v_period = || -> Result<Option<f64>, Failure> {
            Ok(if T::is_v_periodic(ps)? {
                Some(T::v_period(ps)?)
            } else {
                None
            })
        };
        let Some(tv) = translation(v, (bounds.binfv, bounds.bsupv), v_period)? else {
            return Ok(None);
        };
        x[1] = u + tu;
        x[2] = v + tv;
        if self.get_use_solver() {
            self.step_off_bounds(bounds, x)?;
        }
        Ok(Some((tu, tv)))
    }

    /// A small step from the bounds, so that the solver does not find an
    /// "outboundaried" solution (Rsnld -> NotDone).
    fn step_off_bounds(&self, b: PatchBounds, x: &mut Vector<'_>) -> Result<(), Failure> {
        let ps = self.p_surface()?;
        let conf: f64 = precision::confusion();
        let du: f64 = std_max(conf, T::u_resolution(ps, conf)?);
        let dv: f64 = std_max(conf, T::v_resolution(ps, conf)?);
        if x[1] - BOUND_EPS <= b.binfu {
            x[1] += du;
        }
        if x[1] + BOUND_EPS >= b.bsupu {
            x[1] -= du;
        }
        if x[2] - BOUND_EPS <= b.binfv {
            x[2] += dv;
        }
        if x[2] + BOUND_EPS >= b.bsupv {
            x[2] -= dv;
        }
        Ok(())
    }
}
