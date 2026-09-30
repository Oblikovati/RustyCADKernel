//! NCollection_Mat4::Inverted: explicit cofactor expansion in the native term order.
use super::Mat4;
use crate::ncollection::MatrixScalar;
use crate::standard::Failure;

/// For output element `i`: `[p, a, b, c, d, q, e, f, g, h, s, j, k, l, m]` encodes
/// `m[p]*(m[a]*m[b] - m[c]*m[d]) - m[q]*(m[e]*m[f] - m[g]*m[h]) - m[s]*(m[j]*m[k] - m[l]*m[m])`.
const COFACTORS: [[usize; 15]; 16] = [
    [5, 10, 15, 11, 14, 9, 6, 15, 7, 14, 13, 7, 10, 6, 11],
    [1, 11, 14, 10, 15, 9, 3, 14, 2, 15, 13, 2, 11, 3, 10],
    [1, 6, 15, 7, 14, 5, 2, 15, 3, 14, 13, 3, 6, 2, 7],
    [1, 7, 10, 6, 11, 5, 3, 10, 2, 11, 9, 2, 7, 3, 6],
    [4, 11, 14, 10, 15, 8, 7, 14, 6, 15, 12, 6, 11, 7, 10],
    [0, 10, 15, 11, 14, 8, 2, 15, 3, 14, 12, 3, 10, 2, 11],
    [0, 7, 14, 6, 15, 4, 3, 14, 2, 15, 12, 2, 7, 3, 6],
    [0, 6, 11, 7, 10, 4, 2, 11, 3, 10, 8, 3, 6, 2, 7],
    [4, 9, 15, 11, 13, 8, 5, 15, 7, 13, 12, 7, 9, 5, 11],
    [0, 11, 13, 9, 15, 8, 3, 13, 1, 15, 12, 1, 11, 3, 9],
    [0, 5, 15, 7, 13, 4, 1, 15, 3, 13, 12, 3, 5, 1, 7],
    [0, 7, 9, 5, 11, 4, 3, 9, 1, 11, 8, 1, 7, 3, 5],
    [4, 10, 13, 9, 14, 8, 6, 13, 5, 14, 12, 5, 10, 6, 9],
    [0, 9, 14, 10, 13, 8, 1, 14, 2, 13, 12, 2, 9, 1, 10],
    [0, 6, 13, 5, 14, 4, 2, 13, 1, 14, 12, 1, 6, 2, 5],
    [0, 5, 10, 6, 9, 4, 1, 10, 2, 9, 8, 2, 5, 1, 6],
];

fn cofactor<T: MatrixScalar>(m: &[T; 16], term: &[usize; 15]) -> T {
    let minor = |at: usize| m[term[at + 1]] * m[term[at + 2]] - m[term[at + 3]] * m[term[at + 4]];
    m[term[0]] * minor(0) - m[term[5]] * minor(5) - m[term[10]] * minor(10)
}

impl<T: MatrixScalar> Mat4<T> {
    /// Writes the adjugate-based inverse and determinant, as `Inverted(out, det)`.
    ///
    /// Returns false for a zero determinant; `out` then holds the unscaled cofactors,
    /// exactly as upstream leaves it. Example: `matrix.inverted_into(&mut out, &mut det)`.
    pub fn inverted_into(&self, output: &mut Self, determinant: &mut T) -> bool {
        let m = self.data();
        let inverse: [T; 16] = std::array::from_fn(|index| cofactor(&m, &COFACTORS[index]));
        *determinant =
            m[0] * inverse[0] + m[1] * inverse[4] + m[2] * inverse[8] + m[3] * inverse[12];
        *output = Self::from_data(inverse);
        if *determinant == T::from(0) {
            return false;
        }
        let divisor = T::from(1) / *determinant;
        *output = Self::from_data(inverse.map(|value| value * divisor));
        true
    }
    /// `Inverted(out)` without the determinant; `matrix.inverted_to(&mut out)`.
    pub fn inverted_to(&self, output: &mut Self) -> bool {
        let mut determinant: T = T::from(0);
        self.inverted_into(output, &mut determinant)
    }
    /// Returns the inverse or `Standard_ConstructionError`, as `Inverted()`; `matrix.inverted()?`.
    pub fn inverted(&self) -> Result<Self, Failure> {
        let mut output = Self::default();
        if self.inverted_to(&mut output) {
            return Ok(output);
        }
        Err(Failure::with_type(
            "Standard_ConstructionError",
            "NCollection_Mat4::Inverted() - matrix has zero determinant",
            "",
        ))
    }
}
