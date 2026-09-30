// Rust translation of the private BRepAdaptor_CompCurve::Prepare and InvPrepare
// (ModelingData/TKBRep/BRepAdaptor/BRepAdaptor_CompCurve.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The mapping between the parameter of the wire and the parameters of its edges.
use super::CompState;
use crate::brep::BRepTool;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

/// The result of `Prepare(W, Delta, theCurIndex)`.
pub(super) struct Located {
    /// The parameter on the edge (`W` on output).
    pub(super) parameter: f64,
    /// The derivative scale from the edge to the wire parameter (`Delta`).
    pub(super) delta: f64,
    /// The index of the edge (`theCurIndex` on output).
    pub(super) index: i32,
}

impl CompState {
    /// Whether the edge of `index` is parsed against its own direction.
    fn is_reversed(&self, index: i32) -> Result<bool, Failure> {
        let reversed = self.curve(index)?.edge().orientation() == Orientation::Reversed;
        Ok((self.forward && reversed) || (!self.forward && !reversed))
    }

    /// `Prepare(W, Delta, theCurIndex)`: finds the edge holding `w`, starting the search
    /// from `start`; near a knot the edge before it is taken in the first half of the
    /// wire and the edge after it in the second half (the offset `PTol`); then maps `w`
    /// onto the 3D range of that edge.
    pub(super) fn prepare(&self, w: f64, start: i32) -> Result<Located, Failure> {
        let eps = if w - self.t_first < self.t_last - w {
            self.p_tol
        } else {
            -self.p_tol
        };
        let index = self.find_index(w + eps, start)?;
        let (first, last) = BRepTool::range(&self.curve(index)?.edge())?;
        let knot = self.knot(index)?;
        let mut delta = self.knot(index + 1)? - knot;
        if delta > self.p_tol * 1.0e-9 {
            delta = (last - first) / delta;
        }
        if self.is_reversed(index)? {
            delta *= -1.0;
            return Ok(Located {
                parameter: last + (w - knot) * delta,
                delta,
                index,
            });
        }
        Ok(Located {
            parameter: first + (w - knot) * delta,
            delta,
            index,
        })
    }

    /// The index of the knot interval holding `w_test`, searched down or up from
    /// `start`; the first or the last edge when out of limits.
    fn find_index(&self, w_test: f64, start: i32) -> Result<i32, Failure> {
        let count = self.curves()?.len() as i32;
        if self.knot(start)? > w_test {
            for index in (1..start).rev() {
                if self.knot(index)? <= w_test {
                    return Ok(index);
                }
            }
            return Ok(1);
        }
        if self.knot(start + 1)? <= w_test {
            for index in start + 1..=count {
                if self.knot(index + 1)? > w_test {
                    return Ok(index);
                }
            }
            return Ok(count);
        }
        Ok(start)
    }

    /// `InvPrepare(index, First, Delta)`: the parameters of the reparametrisation
    /// `T = Ti + (t - First) * Delta` of the edge of `index`.
    pub(super) fn inv_prepare(&self, index: i32) -> Result<(f64, f64), Failure> {
        let (first, last) = BRepTool::range(&self.curve(index)?.edge())?;
        let mut delta = self.knot(index + 1)? - self.knot(index)?;
        if last - first > self.p_tol * 1.0e-9 {
            delta /= last - first;
        }
        if self.is_reversed(index)? {
            return Ok((last, -delta));
        }
        Ok((first, delta))
    }
}
