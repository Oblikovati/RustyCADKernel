// SPDX-License-Identifier: GPL-2.0-only
//! The regular graph and certified exclusions cover the domain; unresolved fibers stay incomplete.
use super::super::{
    RootBracket, RootIsolationOptions, certificate_arithmetic::world::WorldEnvelope,
    signs::RootSignEvaluation,
};
use super::candidate_certificate::CandidateCertificate;
pub(super) use super::operation_failure::PairOperationFailure;
use super::{
    algebraic, equations::PairedEquations, projection::LinearFiberProjection,
    source::OwnedBezierSource,
};

pub(super) struct PairContact {
    pub root: RootBracket,
    pub u_bounds: [f64; 2],
    pub world: WorldEnvelope,
}
pub(super) struct PairOperationProof {
    pub sources: [OwnedBezierSource; 2],
    pub projection: LinearFiberProjection,
    pub contacts: Vec<PairContact>,
    pub exceptional_fibers: RootSignEvaluation,
    pub source_certificates: [super::source_certificate::SourceCertificate; 2],
    pub candidate_certificate: CandidateCertificate,
}
pub(super) fn compute(
    sources: [OwnedBezierSource; 2],
    self_domain: bool,
    options: RootIsolationOptions,
    requested_error: f64,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<PairOperationProof, PairOperationFailure> {
    validate_world_request(requested_error)?;
    let prepared = checked_projection(&sources, self_domain, options, cancelled)?;
    let settings = (self_domain, options, requested_error);
    let task = GraphTask::new(&sources, &prepared.projection, settings);
    let outcome = task.contacts(cancelled)?;
    Ok(owned_proof(sources, prepared, outcome))
}
fn validate_world_request(value: f64) -> Result<(), PairOperationFailure> {
    if !value.is_finite() || value <= 0. {
        return Err(PairOperationFailure::WorldRequest(value));
    }
    Ok(())
}
struct PreparedProjection {
    projection: LinearFiberProjection,
    exceptional_fibers: RootSignEvaluation,
    source_certificates: [super::source_certificate::SourceCertificate; 2],
}
fn owned_proof(
    sources: [OwnedBezierSource; 2],
    prepared: PreparedProjection,
    outcome: GraphOutcome,
) -> PairOperationProof {
    PairOperationProof {
        sources,
        projection: prepared.projection,
        contacts: outcome.contacts,
        candidate_certificate: outcome.certificate,
        exceptional_fibers: prepared.exceptional_fibers,
        source_certificates: prepared.source_certificates,
    }
}
fn checked_projection(
    sources: &[OwnedBezierSource; 2],
    self_domain: bool,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<PreparedProjection, PairOperationFailure> {
    let (equations, source_certificates) =
        checked_equations(sources, self_domain, options, cancelled)?;
    let projection = LinearFiberProjection::derive(&equations, cancelled)
        .map_err(PairOperationFailure::Root)?
        .ok_or(PairOperationFailure::NonLinearFiber)?;
    let exceptional =
        super::exceptional::certify(&projection, sources[0].interval, options, cancelled)?;
    Ok(PreparedProjection {
        projection,
        exceptional_fibers: exceptional,
        source_certificates,
    })
}
fn checked_equations(
    sources: &[OwnedBezierSource; 2],
    self_domain: bool,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<
    (
        PairedEquations,
        [super::source_certificate::SourceCertificate; 2],
    ),
    PairOperationFailure,
> {
    let left = super::source_certificate::certify(&sources[0], options, cancelled)?;
    let right = super::source_certificate::certify(&sources[1], options, cancelled)?;
    let equations = PairedEquations::derive(&left.homogeneous, &right.homogeneous, cancelled)
        .map_err(PairOperationFailure::Root)?;
    let equations = divide_if_self(equations, self_domain, cancelled)?;
    Ok((equations, [left, right]))
}
fn divide_if_self(
    equations: PairedEquations,
    self_domain: bool,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<PairedEquations, PairOperationFailure> {
    if self_domain {
        return equations
            .exclude_diagonal(cancelled)
            .map_err(PairOperationFailure::Diagonal);
    }
    Ok(equations)
}
struct GraphOutcome {
    contacts: Vec<PairContact>,
    certificate: CandidateCertificate,
}
pub(super) struct GraphTask<'a> {
    pub sources: &'a [OwnedBezierSource; 2],
    pub projection: &'a LinearFiberProjection,
    pub self_domain: bool,
    pub options: RootIsolationOptions,
    pub requested_error: f64,
}
impl<'a> GraphTask<'a> {
    fn new(
        sources: &'a [OwnedBezierSource; 2],
        projection: &'a LinearFiberProjection,
        settings: (bool, RootIsolationOptions, f64),
    ) -> Self {
        Self {
            sources,
            projection,
            self_domain: settings.0,
            options: settings.1,
            requested_error: settings.2,
        }
    }
}
impl GraphTask<'_> {
    fn contacts(
        &self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<GraphOutcome, PairOperationFailure> {
        let axis = self.projection_source_axis()?;
        let polynomial = &self.projection.cleared_originals[axis];
        let roots = self.isolated_candidates(polynomial, cancelled)?;
        let membership = self.membership(&roots, axis, cancelled)?;
        let contacts = self.construct(&roots, membership.accepted.clone(), cancelled)?;
        let certificate = CandidateCertificate::new(polynomial.clone(), roots, membership);
        Ok(GraphOutcome {
            contacts,
            certificate,
        })
    }
}

impl GraphTask<'_> {
    fn isolated_candidates(
        &self,
        polynomial: &super::polynomial::ExactCoefficientPolynomial,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<super::super::RootIsolation, PairOperationFailure> {
        algebraic::solve(
            polynomial,
            self.sources[0].interval,
            self.options,
            cancelled,
        )
        .map_err(PairOperationFailure::Root)
    }
    fn projection_source_axis(&self) -> Result<usize, PairOperationFailure> {
        self.projection
            .cleared_originals
            .iter()
            .position(|p| p.degree().is_some())
            .ok_or(PairOperationFailure::PositiveDimensional)
    }
}
