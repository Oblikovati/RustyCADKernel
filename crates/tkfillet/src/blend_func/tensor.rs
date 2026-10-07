// Rust translation of BlendFunc_Tensor.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A three-index table of reals: the "gradient of gradient" of a blend function.
use super::linalg::mset;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Matrix, Vector};

/// `BlendFunc_Tensor`: `NbRow x NbCol x NbMat` reals indexed from 1, stored in one flat
/// table with `Mat` varying fastest.
///
/// ```
/// use rustycad_tkfillet::blend_func::Tensor;
/// let mut tensor = Tensor::new(2, 2, 2);
/// *tensor.change_value(1, 2, 2)? = 3.0;
/// assert_eq!(tensor.value(1, 2, 2)?, 3.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Tensor {
    tab: Vec<f64>,
    nbrow: i32,
    nbcol: i32,
    nbmat: i32,
    nbmtcl: i32,
}

impl Tensor {
    /// `BlendFunc_Tensor(NbRow, NbCol, NbMat)`: a zero tensor (the native table is left
    /// uninitialized); `Tensor::new(4, 4, 4)`.
    pub fn new(nb_row: i32, nb_col: i32, nb_mat: i32) -> Self {
        let size = (nb_row * nb_mat * nb_col).max(0) as usize;
        Self {
            tab: vec![0.0; size],
            nbrow: nb_row,
            nbcol: nb_col,
            nbmat: nb_mat,
            nbmtcl: nb_mat * nb_col,
        }
    }

    /// `Init(InitialValue)`: every element set to `value`; `tensor.init(0.0)`.
    pub fn init(&mut self, value: f64) {
        self.tab.fill(value);
    }

    /// The flat 0-based offset of `(row, col, mat)`, Standard_OutOfRange outside the table.
    fn offset(&self, row: i32, col: i32, mat: i32) -> Result<usize, Failure> {
        let index = mat + self.nbmat * (col - 1) + self.nbmtcl * (row - 1);
        if index < 1 || index as usize > self.tab.len() {
            return Err(Failure::out_of_range(&format!(
                "BlendFunc_Tensor: ({row}, {col}, {mat}) gives the index {index}; expected one in [1, {}]",
                self.tab.len()
            )));
        }
        Ok((index - 1) as usize)
    }

    /// `Value(Row, Col, Mat)` (and `operator()`); `tensor.value(1, 1, 1)?`.
    pub fn value(&self, row: i32, col: i32, mat: i32) -> Result<f64, Failure> {
        Ok(self.tab[self.offset(row, col, mat)?])
    }

    /// `ChangeValue(Row, Col, Mat)` (and `operator()`); `*tensor.change_value(1, 1, 1)? = 2.0`.
    pub fn change_value(&mut self, row: i32, col: i32, mat: i32) -> Result<&mut f64, Failure> {
        let offset = self.offset(row, col, mat)?;
        Ok(&mut self.tab[offset])
    }

    /// `Multiply(Right, Product)`: `Product(i, j) = sum_k T(i, j, k) * Right(k)`, summed in
    /// increasing `k` from 0; `tensor.multiply(&right, &mut product)?`.
    pub fn multiply(&self, right: &Vector<'_>, product: &mut Matrix<'_>) -> Result<(), Failure> {
        for i in 1..=self.nbrow {
            for j in 1..=self.nbcol {
                let mut somme = 0.0;
                for k in 1..=self.nbmat {
                    somme += self.value(i, j, k)? * *right.value(k)?;
                }
                mset(product, i, j, somme)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiply_contracts_the_last_index() {
        let mut tensor = Tensor::new(2, 2, 3);
        tensor.init(1.0);
        *tensor.change_value(2, 1, 3).unwrap() = 4.0;
        let mut right = Vector::with_bounds(1, 3).unwrap();
        right[1] = 1.0;
        right[2] = 2.0;
        right[3] = 3.0;
        let mut product = Matrix::with_bounds(1, 2, 1, 2).unwrap();
        tensor.multiply(&right, &mut product).unwrap();
        assert_eq!(product[(1, 1)], 6.0);
        assert_eq!(product[(2, 1)], 15.0);
    }

    #[test]
    fn indices_outside_the_table_raise_out_of_range() {
        let tensor = Tensor::new(2, 2, 2);
        assert_eq!(
            tensor.value(3, 1, 1).unwrap_err().exception_type(),
            "Standard_OutOfRange"
        );
    }
}
