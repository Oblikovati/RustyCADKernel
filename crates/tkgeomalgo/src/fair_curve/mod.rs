// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package FairCurve: fair 2D curves of minimal energy between two points.
//!
//! `FairCurveBatten` bends an elastic batten (minimal flexion energy) and
//! `FairCurveMinimalVariation` minimizes the variation of curvature as well. Both
//! reach the requested constraints through a homotopy of small steps; each step
//! deforms the current B-spline by Hermite polynomials and minimizes a
//! `FairCurveEnergy` with `FairCurveNewton`. The energies integrate the
//! distributions (`FairCurveDistributionOfTension`, `...Sagging`, `...Jerk`) with a
//! 24-point Gauss rule.
//!
//! Native quirks kept (see docs/tkgeomalgo-progress.md): the distributions hold
//! copies of the batten law, so only the tension sees the sliding updates; the
//! integration loop of `FairCurveEnergyOfBatten` can succeed after a failed sagging
//! integral of an earlier interval; the evaluation sources define
//! `No_Standard_RangeError`, so an out-of-range index (only reachable through
//! inconsistent inputs) is undefined behaviour natively and panics here.
mod batten;
mod batten_law;
mod distribution;
mod dump;
mod energy;
mod energy_batten;
mod energy_mvc;
mod energy_terms;
mod jerk;
mod minimal_variation;
mod newton;
mod sagging;
mod tension;

pub use batten::FairCurveBatten;
pub use batten_law::FairCurveBattenLaw;
pub use distribution::FairCurveDistributionOfEnergy;
pub use energy::{FairCurveEnergy, FairCurveEnergyModel};
pub use energy_batten::FairCurveEnergyOfBatten;
pub use energy_mvc::FairCurveEnergyOfMvc;
pub use jerk::FairCurveDistributionOfJerk;
pub use minimal_variation::FairCurveMinimalVariation;
pub use newton::FairCurveNewton;
pub use sagging::FairCurveDistributionOfSagging;
pub use tension::FairCurveDistributionOfTension;

/// `FairCurve_AnalysisCode`: the outcome of a batten computation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FairCurveAnalysisCode {
    /// The computation converged.
    #[default]
    Ok,
    /// The minimization did not converge; the curve may be of poor quality.
    NotConverged,
    /// The free sliding grew without bound; impose a sliding factor instead.
    InfiniteSliding,
    /// The height of the batten vanishes at one end; change the slope.
    NullHeight,
}
