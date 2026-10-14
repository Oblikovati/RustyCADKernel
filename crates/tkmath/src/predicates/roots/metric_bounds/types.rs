// SPDX-License-Identifier: GPL-2.0-only
use super::super::signs::RootSignError;
use super::super::{DyadicBound, RootBracket, RootIsolationError, RootIsolationOptions};

/// Stored Bézier coefficients; weights are positive or omitted for polynomial curves.
/// Example: `MetricBezier { poles: &[[0.,0.,0.],[1.,0.,0.]], weights: None }`.
#[derive(Clone, Copy)]
pub struct MetricBezier<'a, const DIM: usize> {
    /// Stored analytic control points.
    pub poles: &'a [[f64; DIM]],
    /// Matching positive rational weights, or unit weights when omitted.
    pub weights: Option<&'a [f64]>,
}
/// Same-parameter analytic representations; lines retain origin/direction without rounded endpoints.
#[derive(Clone, Copy)]
pub enum MetricCurve<'a, const DIM: usize> {
    /// Rational or polynomial Bézier on its natural [0,1] parameter domain.
    Bezier(MetricBezier<'a, DIM>),
    /// Affine curve origin + parameter * direction on the requested finite trim.
    Line {
        /// Stored affine origin.
        origin: [f64; DIM],
        /// Stored affine parameter direction.
        direction: [f64; DIM],
    },
}
/// Affine placement evaluated exactly from its stored binary64 coefficients.
/// Example: a planar lift uses two axis columns and a three-coordinate origin.
#[derive(Clone, Copy)]
pub struct MetricEmbedding<const DIM: usize> {
    /// Unscaled linear coefficients; no precomputed rounded products.
    pub linear: [[f64; DIM]; 3],
    /// Stored affine origin.
    pub origin: [f64; 3],
    /// Independent unscaled-matrix multiplier; products are formed exactly.
    pub scale: f64,
}
/// Both representations use the same actual parameter, including the finite trim.
/// Example: `interval: [0.25,0.75]` checks the complete middle half, not samples.
pub struct PlanarResidualQuery<'a> {
    /// Actual three-dimensional analytic representation.
    pub curve: MetricCurve<'a, 3>,
    /// Actual placement applied to the space curve.
    pub curve_embedding: MetricEmbedding<3>,
    /// Actual analytic curve in plane parameters.
    pub pcurve: MetricCurve<'a, 2>,
    /// Actual plane frame before location.
    pub plane_embedding: MetricEmbedding<2>,
    /// Actual surface location applied after its stored planar frame.
    pub plane_placement: MetricEmbedding<3>,
    /// Complete common parameter trim.
    pub interval: [f64; 2],
    /// Requested nonnegative spatial distance limit.
    pub spatial_limit: f64,
    /// Root approximation/work policy, independent of spatial tolerance.
    pub options: RootIsolationOptions,
}
/// Actual exact inequality evidence, not a numerically estimated supremum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResidualDecision {
    /// The full interval is bounded; this bound need not be the tight maximum.
    Within {
        /// Certified squared error upper bound, not a tight measured supremum.
        proven_squared_upper: DyadicBound,
    },
    /// An endpoint or stationary algebraic parameter proves limit exceedance.
    Exceeds {
        /// Exact parameter evidence of negative threshold polynomial.
        witness: ResidualViolation,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
/// Parameter witness proving a spatial limit is violated.
pub enum ResidualViolation {
    /// Actual included trim endpoint.
    Endpoint(DyadicBound),
    /// Certified stationary root of the exact threshold polynomial.
    Stationary(RootBracket),
}
/// No complete decision is returned after unsupported input or exhausted work.
#[derive(Debug)]
pub enum ResidualError {
    /// Missing or mismatched control arrays.
    Shape {
        /// Name of the offending representation.
        representation: &'static str,
        /// Stored analytic control points.
        /// Actual number of supplied poles.
        poles: usize,
        /// Matching positive rational weights, or unit weights when omitted.
        /// Actual number of supplied weights when present.
        weights: Option<usize>,
    },
    /// Unsupported nonfinite value or nonpositive weight.
    Value {
        /// Name of the rejected coefficient role.
        role: &'static str,
        /// Zero-based offending coefficient index.
        index: usize,
        /// Actual offending supplied value.
        value: f64,
    },
    /// Unsupported trim outside the actual representation domain.
    Domain([f64; 2]),
    /// Actual exact subinterval does not belong to the original declared trim.
    ExactDomain {
        /// Actual ordered or offending exact parameter bounds.
        supplied: [DyadicBound; 2],
        /// Original declared source trim, never an approximate candidate cut.
        declared: [f64; 2],
    },
    /// Supplied parameter evidence must be an ordered exact bracket.
    ParameterBracket {
        /// Offending exact supplied bounds.
        supplied: [DyadicBound; 2],
    },
    /// Exact transported Bézier domain lies outside its native [0,1] basis.
    ParameterDomain {
        /// Exact transported bounds in the stored curve basis.
        supplied: [DyadicBound; 2],
    },
    /// Original root-completion or policy cause.
    Roots(RootIsolationError),
    /// Original exact algebraic sign-completion cause.
    Signs(RootSignError),
}
impl From<RootIsolationError> for ResidualError {
    fn from(cause: RootIsolationError) -> Self {
        Self::Roots(cause)
    }
}
impl From<RootSignError> for ResidualError {
    fn from(cause: RootSignError) -> Self {
        Self::Signs(cause)
    }
}
impl std::fmt::Display for ResidualError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shape {
                representation,
                poles,
                weights,
            } => f.write_str(&shape_message(representation, *poles, *weights)),
            Self::Value { role, index, value } => f.write_str(&value_message(role, *index, *value)),
            Self::Domain(interval) => f.write_str(&domain_message(interval)),
            Self::ExactDomain { supplied, declared } => {
                f.write_str(&exact_domain_message(supplied, declared))
            }
            Self::ParameterBracket { supplied } => {
                f.write_str(&parameter_bracket_message(supplied))
            }
            Self::ParameterDomain { supplied } => f.write_str(&parameter_domain_message(supplied)),
            Self::Roots(cause) => cause.fmt(f),
            Self::Signs(cause) => cause.fmt(f),
        }
    }
}
impl std::error::Error for ResidualError {}

fn shape_message(representation: &str, poles: usize, weights: Option<usize>) -> String {
    format!(
        "{representation}: poles={poles}, weights={weights:?}; expected nonempty poles and matching weights"
    )
}

fn exact_domain_message(supplied: &[DyadicBound; 2], declared: &[f64; 2]) -> String {
    format!(
        "exact metric bounds {supplied:?}; expected ordered bounds inside declared source trim {declared:?}"
    )
}

fn value_message(role: &str, index: usize, value: f64) -> String {
    format!(
        "{role}[{index}]={value}; expected finite coefficients, positive weights and nonnegative spatial limit"
    )
}
fn domain_message(interval: &[f64; 2]) -> String {
    format!(
        "residual interval {interval:?}; expected finite first <= last, within [0,1] when a Bézier is present"
    )
}

fn parameter_bracket_message(supplied: &[DyadicBound; 2]) -> String {
    format!("parameter bracket {supplied:?}; expected ordered exact bounds")
}
fn parameter_domain_message(supplied: &[DyadicBound; 2]) -> String {
    format!("transported domain {supplied:?}; expected native Bézier bounds inside [0,1]")
}
