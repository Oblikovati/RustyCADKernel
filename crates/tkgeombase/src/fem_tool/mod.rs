// Rust translation of ModelingData/TKGeomBase/FEmTool (package overview)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! FEmTool: the finite-element tools of the variational approximation (AppDef):
//! the element curve in the Hermite-Jacobi basis ([`Curve`]), the quadratic
//! element criteria ([`LinearFlexion`], [`LinearJerk`], [`LinearTension`]), their
//! reference integrals ([`ElementsOfRefMatrix`]), the profile matrix
//! ([`ProfileMatrix`], a [`SparseMatrix`]) and the constrained global system
//! ([`Assembly`]).
//!
//! The transient classes (`Curve`, the criteria, `ProfileMatrix`) keep their
//! mutable state behind a lock, so every handle to one object sees the same
//! state and the native non-const members take `&self`.
mod assembly;
mod curve;
mod elementary_criterion;
mod elements_of_ref_matrix;
mod linear_criterion;
mod linear_kinds;
mod profile_matrix;
mod reference_matrix;
mod sparse_matrix;

pub use assembly::{Assembly, AssemblyTable};
pub use curve::Curve;
pub use elementary_criterion::{CriterionBase, CriterionFields, ElementaryCriterion};
pub use elements_of_ref_matrix::ElementsOfRefMatrix;
pub use linear_criterion::{LinearCriterion, LinearKind, NegativeClamp};
pub use linear_kinds::{Flexion, Jerk, LinearFlexion, LinearJerk, LinearTension, Tension};
pub use profile_matrix::ProfileMatrix;
pub use reference_matrix::ReferenceCache;
pub use sparse_matrix::{EntryRef, SparseMatrix};
