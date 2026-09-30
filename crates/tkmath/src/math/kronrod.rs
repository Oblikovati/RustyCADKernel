//! Kronrod quadrature generated from the native Jacobi-Kronrod recurrence.
use super::gauss_quadrature::sorted_nodes;
use super::{EigenValuesSearcher, Vector};
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
mod recurrence;
/// Computed quadrature nodes and weights; `KronrodPointsAndWeights::new(125)?`.
pub struct KronrodPointsAndWeights {
    points: Vector<'static>,
    weights: Vector<'static>,
    done: bool,
}
impl KronrodPointsAndWeights {
    /// Computes 2*n+1 nodes on [-1,1]; `KronrodPointsAndWeights::new(7)?`.
    pub fn new(order: i32) -> Result<Self, Failure> {
        // Order 0 reads b[1] and a[-1] outside the native coefficient arrays and negative
        // orders fail the array allocation, so both are refused instead of reproducing UB.
        if order < 1 {
            return Err(Failure::out_of_range(&format!(
                "Kronrod order {order}; expected a positive order"
            )));
        }
        let size: i32 = order
            .checked_mul(2)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| Failure::out_of_range("Kronrod order overflows i32"))?;
        let (diagonal, subdiagonal): (Array1<'static, f64>, Array1<'static, f64>) =
            recurrence::coefficients(order, size);
        let mut result: Self = Self {
            points: Vector::with_bounds(1, size)?,
            weights: Vector::with_bounds(1, size)?,
            done: false,
        };
        // The native constructor catches every Standard_Failure and stays not done.
        let Ok(eigensolver) = EigenValuesSearcher::new(&diagonal, &subdiagonal) else {
            return Ok(result);
        };
        if eigensolver.is_done() {
            result.collect(&eigensolver)?;
        }
        Ok(result)
    }
    fn collect(&mut self, eigensolver: &EigenValuesSearcher) -> Result<(), Failure> {
        sorted_nodes(eigensolver, &mut self.points, &mut self.weights)?;
        self.done = true;
        Ok(())
    }
    /// Reports convergence of the Jacobi-Kronrod eigenproblem; `quadrature.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Copies ascending nodes with one-based bounds; `quadrature.points()`.
    pub fn points(&self) -> Vector<'static> {
        self.points.clone()
    }
    /// Copies the corresponding quadrature weights; `quadrature.weights()`.
    pub fn weights(&self) -> Vector<'static> {
        self.weights.clone()
    }
}
