// Rust translation of ModelingAlgorithms/TKGeomAlgo/Plate at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package Plate: the variational plate `Plate_Plate` (the minimizer of an
//! iterated Laplacian energy under point, derivative and linear constraints) and its
//! constraint classes, used by GeomPlate and the filling algorithms.
//!
//! The constraint classes are values; the linear constraints share their arrays
//! through handles, as natively, so a copy loaded in a plate sees later `SetPPC` and
//! `SetCoeff` calls. `Plate_Plate::Load` overloads take distinct names
//! (`load_pinpoint`, `load_linear_xyz`...).
mod free_gtoc;
mod gtoc;
mod gtoc_math;
mod linear;
#[allow(clippy::module_inception)] // The package and its main class share the name.
mod plate;
mod shaped;
mod sol_em;
mod sol_em_terms;
mod solve;
mod solve_scalar;
mod system;

pub use free_gtoc::PlateFreeGtoCConstraint;
pub use gtoc::PlateGtoCConstraint;
pub use linear::{PlateLinearScalarConstraint, PlateLinearXYZConstraint, PlatePinpointConstraint};
pub use plate::PlatePlate;
pub use shaped::{
    PlateD1, PlateD2, PlateD3, PlateGlobalTranslationConstraint, PlateLineConstraint,
    PlatePlaneConstraint, PlateSampledCurveConstraint,
};
