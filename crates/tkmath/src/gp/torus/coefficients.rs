// Rust translation of FoundationClasses/TKMath/gp/gp_Torus.cxx Coefficients at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The 35 quartic coefficients of gp_Torus::Coefficients, grouped by the product shapes the
//! native code repeats; every group keeps the native multiplication and summation order.
use super::super::placement::column_product;

/// Column dot products `c[i][j] = Σ T(r,i)·T(r,j)` over the three frame-change rows.
struct Columns {
    all: [[f64; 4]; 4],
    planar: [[f64; 4]; 4],
    rows: [[f64; 4]; 3],
    sum: f64,
    difference: f64,
}
impl Columns {
    fn new(rows: [[f64; 4]; 3], major: f64, minor: f64) -> Self {
        Self {
            all: std::array::from_fn(|i| std::array::from_fn(|j| column_product(&rows, i, j))),
            planar: std::array::from_fn(|i| {
                std::array::from_fn(|j| column_product(&rows[..2], i, j))
            }),
            rows,
            sum: major * major + minor * minor,
            difference: major * major - minor * minor,
        }
    }
    /// `4·c[a][a]·c[a][b]` (entries 3-8 and 15-17).
    fn linear(&self, a: usize, b: usize) -> f64 {
        4.0 * self.all[a][a] * self.all[a][b]
    }
    /// `c[a][a]·c[b][c] + 2·c[a][b]·c[a][c]` (entries 9-14 and 18-23 before scaling).
    fn mixed(&self, a: usize, b: usize, c: usize) -> f64 {
        self.all[a][a] * self.all[b][c] + 2.0 * self.all[a][b] * self.all[a][c]
    }
    /// Quadratic radius terms of entries 25-30 before scaling.
    fn radial(&self, i: usize, j: usize) -> f64 {
        self.difference * self.rows[2][i] * self.rows[2][j] - self.sum * self.planar[i][j]
            + self.all[3][3] * self.all[i][j]
            + 2.0 * self.all[i][3] * self.all[j][3]
    }
    /// Linear radius terms of entries 31-33 before scaling.
    fn axial(&self, i: usize) -> f64 {
        self.all[3][3] * self.all[i][3] + self.difference * self.rows[2][i] * self.rows[2][3]
            - self.sum * self.planar[i][3]
    }
    fn constant(&self) -> f64 {
        let t34 = self.rows[2][3];
        2.0 * self.difference * t34 * t34 - 2.0 * self.sum * self.planar[3][3]
            + self.all[3][3] * self.all[3][3]
            + self.difference * self.difference
    }
    fn cross_terms(&self) -> f64 {
        let c = &self.all;
        8.0 * (c[0][1] * c[2][3] + c[1][2] * c[0][3] + c[1][3] * c[0][2])
    }
}

/// Evaluates gp_Torus::Coefficients for the frame-change rows; `quartic(rows,5.0,1.0)`.
pub(super) fn quartic(rows: [[f64; 4]; 3], major: f64, minor: f64) -> [f64; 35] {
    let c = Columns::new(rows, major, minor);
    let mut out = Coefficients {
        values: [0.0; 35],
        next: 0,
    };
    out.extend(&[0, 1, 2].map(|a| c.all[a][a] * c.all[a][a]));
    out.extend(&CROSS_PAIRS.map(|(a, b)| c.linear(a, b)));
    out.extend(&[(0, 1), (0, 2), (1, 2)].map(|(a, b)| 2.0 * c.mixed(a, b, b)));
    out.extend(&[(0, 1, 2), (1, 0, 2), (2, 0, 1)].map(|(a, b, d)| 4.0 * c.mixed(a, b, d)));
    out.extend(&[0, 1, 2].map(|a| c.linear(a, 3)));
    out.extend(&CROSS_PAIRS.map(|(a, d)| 4.0 * c.mixed(a, 3, d)));
    out.extend(&[c.cross_terms()]);
    out.extend(&[0, 1, 2].map(|i| 2.0 * c.radial(i, i)));
    out.extend(&[(0, 1), (0, 2), (1, 2)].map(|(i, j)| 4.0 * c.radial(i, j)));
    out.extend(&[0, 1, 2].map(|i| 4.0 * c.axial(i)));
    out.extend(&[c.constant()]);
    out.values
}
/// Ordered pairs of distinct spatial columns in the native entry order (3-8 and 18-23).
const CROSS_PAIRS: [(usize, usize); 6] = [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)];
/// Sequential writer over the 35 entries.
struct Coefficients {
    values: [f64; 35],
    next: usize,
}
impl Coefficients {
    fn extend(&mut self, group: &[f64]) {
        self.values[self.next..self.next + group.len()].copy_from_slice(group);
        self.next += group.len();
    }
}
