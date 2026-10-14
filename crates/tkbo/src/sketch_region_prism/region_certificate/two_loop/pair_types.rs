// SPDX-License-Identifier: GPL-2.0-only
//! Typed projections of genuine complete provider evidence, including excluded roots/fibres.
use crate::sketch_region::RegionSourceId;
use rustycad_tkmath::predicates::roots::{
    RootBracket, RootIsolation, bezier_pair::*, signs::RootSignEvaluation,
};
#[derive(Debug)]
pub(crate) struct TwoLoopExactPolynomial(pub(crate) Vec<String>);
#[derive(Debug)]
pub(crate) struct TwoLoopPairProof {
    pub(crate) sources: [RegionSourceId; 2],
    pub(crate) policy: BezierPairPolicy,
    pub(crate) work: BezierPairWork,
    pub(crate) denominators: [RootIsolation; 2],
    pub(crate) homogeneous: [[TwoLoopExactPolynomial; 3]; 2],
    pub(crate) evidence: TwoLoopPairExclusion,
    pub(crate) closure: Option<TwoLoopClosureProof>,
}
#[derive(Debug)]
pub(crate) enum TwoLoopPairExclusion {
    Graph(Box<TwoLoopGraphProof>),
    Constant {
        axis: usize,
        divided_self_diagonal: bool,
        equations: [Vec<TwoLoopExactPolynomial>; 2],
    },
}
#[derive(Debug)]
pub(crate) struct TwoLoopGraphProof {
    pub(crate) candidate_polynomial: TwoLoopExactPolynomial,
    pub(crate) candidates: RootIsolation,
    pub(crate) alpha: TwoLoopExactPolynomial,
    pub(crate) beta: TwoLoopExactPolynomial,
    pub(crate) cleared_originals: [TwoLoopExactPolynomial; 2],
    pub(crate) leading_factors: Vec<TwoLoopExactPolynomial>,
    pub(crate) membership: Vec<TwoLoopMembershipProof>,
    pub(crate) accepted: Vec<bool>,
    pub(crate) exceptional: RootSignEvaluation,
    pub(crate) effective_signs: BezierPairEffectiveSigns,
}
#[derive(Debug)]
pub(crate) struct TwoLoopMembershipProof {
    pub(crate) coefficients: TwoLoopExactPolynomial,
    pub(crate) rule: BezierPairMembershipRule,
    pub(crate) evaluation: RootSignEvaluation,
}
#[derive(Debug)]
pub(crate) struct TwoLoopClosureProof {
    pub(crate) parameter: RootBracket,
    /// The actual exact paired enclosure has been compared equal to [1,1].
    pub(crate) exact_paired_parameter: [u8; 2],
    pub(crate) paired_parameter_bounds: [f64; 2],
    pub(crate) world_bounds: [[f64; 2]; 2],
    pub(crate) point: [f64; 2],
    pub(crate) achieved_world_error: f64,
}
