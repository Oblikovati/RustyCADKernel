//! Modern linear algebra preserving the pinned OCCT algorithms and status fields.
mod eigen;
mod jacobi;
pub use eigen::{TridiagonalResult, eigen_tridiagonal, get_eigen_vector};
pub use jacobi::{
    eigen_values, jacobi, matrix_inv_sqrt, matrix_power, matrix_sqrt, spectral_decomposition,
};

mod gauss;
pub mod utils;
pub use gauss::{LuResult, determinant, invert, lu, solve, solve_multiple};

mod crout;
mod householder;
pub use crout::{CroutResult, crout, invert_crout, solve_crout};
pub use householder::{QrResult, qr, solve_qr, solve_qr_multiple};

mod svd;
pub use svd::{SvdResult, condition_number, numerical_rank, pseudo_inverse, solve_svd, svd};

mod least_squares;
pub use least_squares::{
    LeastSquaresMethod, LeastSquaresResult, least_squares, optimal_regularization,
    regularized_least_squares, weighted_least_squares,
};
