//! Bounded numeric storage and mathematical algorithms.
mod double_tab;
pub use double_tab::DoubleTab;
mod matrix;
mod vector;
pub use matrix::Matrix;
pub use vector::Vector;
mod gauss;
pub use gauss::Gauss;
mod crout;
pub use crout::Crout;
mod least_squares;
pub use least_squares::GaussLeastSquares;
mod householder;
pub use householder::Householder;
mod jacobi;
pub use jacobi::Jacobi;
mod svd;
pub use svd::Svd;
mod eigen_values_searcher;
pub use eigen_values_searcher::EigenValuesSearcher;
mod kronrod;
pub use kronrod::KronrodPointsAndWeights;
mod function;
pub use function::Function;
mod bracketed_root;
pub use bracketed_root::BracketedRoot;
mod derivative;
pub use derivative::FunctionWithDerivative;
mod bissec_newton;
pub use bissec_newton::BissecNewton;
mod bracket_minimum;
pub use bracket_minimum::BracketMinimum;
mod brent_minimum;
pub use brent_minimum::BrentMinimum;
mod function_set;
pub use function_set::{FunctionSet, FunctionSetWithDerivatives};
mod newton_function_set_root;
pub use newton_function_set_root::NewtonFunctionSetRoot;
mod function_set_root;
pub use function_set_root::FunctionSetRoot;
mod function_root;
pub use function_root::FunctionRoot;
mod multiple_var_function;
pub use multiple_var_function::{MultipleVarFunction, MultipleVarFunctionWithGradient};
mod bfgs;
mod optimization;
pub use bfgs::Bfgs;
mod frpr;
pub use frpr::Frpr;
mod powell;
pub use multiple_var_function::MultipleVarFunctionWithHessian;
pub use powell::Powell;
mod status;
pub use status::Status;
mod newton_minimum;
pub use newton_minimum::NewtonMinimum;
mod newton_function_root;
pub use newton_function_root::NewtonFunctionRoot;
mod uzawa;
pub use uzawa::Uzawa;
mod direct_polynomial_roots;
pub use direct_polynomial_roots::DirectPolynomialRoots;
mod trigonometric_equation;
pub use trigonometric_equation::TrigonometricEquation;
mod trigonometric_roots;
pub use trigonometric_roots::TrigonometricRoots;
// Standard_Real::Epsilon moves toward the finite extreme, including from infinity.
pub(crate) fn real_epsilon(value: f64) -> f64 {
    let magnitude: f64 = value.abs();
    if magnitude == f64::MAX {
        return 0.0;
    }
    if magnitude == f64::INFINITY {
        return f64::NEG_INFINITY;
    }
    magnitude.next_up() - magnitude
}

mod function_roots;
pub use function_roots::FunctionRoots;

mod function_sample;
pub use function_sample::{FunctionSample, FunctionSampling};
mod function_all_roots;
pub use function_all_roots::FunctionAllRoots;

mod gauss_quadrature;
pub use gauss_quadrature::GaussPointsAndWeights;
mod quadrature_tables;
pub use quadrature_tables::{
    gauss_points, gauss_points_max, gauss_weights, kronrod_points_and_weights, kronrod_points_max,
    ordered_gauss_points_and_weights,
};
mod gauss_integration;
pub use gauss_integration::GaussSingleIntegration;

mod kronrod_integration;
pub use kronrod_integration::KronrodSingleIntegration;

mod bullard_generator;
pub use bullard_generator::BullardGenerator;
mod pso_particles;
pub use pso_particles::{PsoParticle, PsoParticlesPool};
mod pso;
pub use pso::Pso;

mod global_objective;
pub use global_objective::GlobalObjective;
mod global_optimum;
pub use global_optimum::GlobalOptimum;
