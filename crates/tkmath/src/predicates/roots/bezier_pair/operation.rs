// SPDX-License-Identifier: GPL-2.0-only
//! The regular graph and certified exclusions cover the domain; unresolved fibers stay incomplete.
use super::super::{
    RootBracket, RootIsolationOptions, certificate_arithmetic::world::WorldEnvelope,
    signs::RootSignEvaluation,
};
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
}
pub(super) fn compute(
    sources: [OwnedBezierSource; 2],
    self_domain: bool,
    options: RootIsolationOptions,
    requested_error: f64,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<PairOperationProof, PairOperationFailure> {
    validate_world_request(requested_error)?;
    let (projection, exceptional_fibers) =
        checked_projection(&sources, self_domain, options, cancelled)?;
    let settings = (self_domain, options, requested_error);
    let task = GraphTask::new(&sources, &projection, settings);
    let contacts = task.contacts(cancelled)?;
    Ok(owned_proof(
        sources,
        projection,
        contacts,
        exceptional_fibers,
    ))
}
fn validate_world_request(value: f64) -> Result<(), PairOperationFailure> {
    if !value.is_finite() || value <= 0. {
        return Err(PairOperationFailure::WorldRequest(value));
    }
    Ok(())
}
fn owned_proof(
    sources: [OwnedBezierSource; 2],
    projection: LinearFiberProjection,
    contacts: Vec<PairContact>,
    exceptional_fibers: RootSignEvaluation,
) -> PairOperationProof {
    PairOperationProof {
        sources,
        projection,
        contacts,
        exceptional_fibers,
    }
}
fn checked_projection(
    sources: &[OwnedBezierSource; 2],
    self_domain: bool,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(LinearFiberProjection, RootSignEvaluation), PairOperationFailure> {
    let equations = checked_equations(sources, self_domain, options, cancelled)?;
    let projection = LinearFiberProjection::derive(&equations, cancelled)
        .map_err(PairOperationFailure::Root)?
        .ok_or(PairOperationFailure::NonLinearFiber)?;
    let exceptional =
        super::exceptional::certify(&projection, sources[0].interval, options, cancelled)?;
    Ok((projection, exceptional))
}
fn checked_equations(
    sources: &[OwnedBezierSource; 2],
    self_domain: bool,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<PairedEquations, PairOperationFailure> {
    let left = checked_source(&sources[0], options, cancelled)?;
    let right = checked_source(&sources[1], options, cancelled)?;
    let equations =
        PairedEquations::derive(&left, &right, cancelled).map_err(PairOperationFailure::Root)?;
    if self_domain {
        return equations
            .exclude_diagonal(cancelled)
            .map_err(PairOperationFailure::Diagonal);
    }
    Ok(equations)
}
fn checked_source(
    source: &OwnedBezierSource,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<super::source::HomogeneousPolynomials, PairOperationFailure> {
    let polynomials = source
        .homogeneous(cancelled)
        .map_err(PairOperationFailure::Source)?;
    let roots = algebraic::solve(
        &polynomials.components[2],
        source.interval,
        options,
        cancelled,
    )
    .map_err(PairOperationFailure::Root)?;
    if !roots.roots().is_empty() {
        return Err(PairOperationFailure::DenominatorRoots);
    }
    Ok(polynomials)
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
    ) -> Result<Vec<PairContact>, PairOperationFailure> {
        let axis = self.projection_source_axis()?;
        let polynomial = &self.projection.cleared_originals[axis];
        let roots = algebraic::solve(
            polynomial,
            self.sources[0].interval,
            self.options,
            cancelled,
        )
        .map_err(PairOperationFailure::Root)?;
        let flags = self.membership(&roots, axis, cancelled)?;
        self.construct(&roots, flags, cancelled)
    }
}

impl GraphTask<'_> {
    fn projection_source_axis(&self) -> Result<usize, PairOperationFailure> {
        self.projection
            .cleared_originals
            .iter()
            .position(|p| p.degree().is_some())
            .ok_or(PairOperationFailure::PositiveDimensional)
    }
}
