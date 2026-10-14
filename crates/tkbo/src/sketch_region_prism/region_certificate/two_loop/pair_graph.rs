// SPDX-License-Identifier: GPL-2.0-only
//! Preserve exact graph/membership/exceptional evidence instead of a contact-count assertion.
use super::{pair_capture::*, pair_types::*, work::*};
use rustycad_tkmath::predicates::roots::bezier_pair::*;

pub(super) fn evidence<'a>(
    proof: &BezierPairCertificate<'_>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopPairExclusion> {
    if let Some(graph) = proof.graph() {
        return Ok(TwoLoopPairExclusion::Graph(Box::new(graph_proof(
            graph, work,
        )?)));
    }
    let constant = proof
        .constant_exclusion()
        .expect("complete graph or constant exclusion");
    let first = constant.equation(0).expect("first original equation");
    let second = constant.equation(1).expect("second original equation");
    work.units("constant exclusion", 1)?;
    Ok(TwoLoopPairExclusion::Constant {
        axis: constant.coordinate_axis(),
        divided_self_diagonal: constant.divided_self_diagonal(),
        equations: [polynomials(first, work)?, polynomials(second, work)?],
    })
}
fn graph_proof<'a>(
    graph: BezierPairGraphCertificate<'_>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopGraphProof> {
    let (coefficients, candidates) = graph.candidates();
    let projection = graph.projection();
    work.units("graph proof", 1)?;
    work.units("candidate acceptance", graph.accepted_candidates().len())?;
    Ok(TwoLoopGraphProof {
        candidate_polynomial: polynomial(coefficients, work)?,
        candidates: isolation(candidates, work)?,
        alpha: polynomial(projection.alpha, work)?,
        beta: polynomial(projection.beta, work)?,
        cleared_originals: [
            polynomial(projection.cleared_originals[0], work)?,
            polynomial(projection.cleared_originals[1], work)?,
        ],
        leading_factors: polynomials(graph.leading_factors(), work)?,
        membership: memberships(&graph, work)?,
        accepted: graph.accepted_candidates().to_vec(),
        exceptional: signs(graph.exceptional_fibers(), work)?,
        effective_signs: graph.effective_signs(),
    })
}
fn polynomials<'a, 'p>(
    rows: impl Iterator<Item = ExactPairPolynomial<'p>>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Vec<TwoLoopExactPolynomial>> {
    let mut captured = Vec::new();
    for row in rows {
        work.units("polynomial record", 1)?;
        captured.push(polynomial(row, work)?);
    }
    Ok(captured)
}
fn memberships<'a>(
    graph: &BezierPairGraphCertificate<'_>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Vec<TwoLoopMembershipProof>> {
    let mut rows = Vec::new();
    for row in graph.membership() {
        work.units("membership record", 1)?;
        rows.push(TwoLoopMembershipProof {
            coefficients: polynomial(row.coefficients, work)?,
            rule: row.rule,
            evaluation: signs(row.evaluation, work)?,
        });
    }
    Ok(rows)
}
