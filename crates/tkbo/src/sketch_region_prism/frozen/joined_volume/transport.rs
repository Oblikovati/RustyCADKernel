// SPDX-License-Identifier: GPL-2.0-only
//! A sign proof, never a rounded integral or mesh, chooses the exact isometry x -> +/-x.
use super::*;
use rustycad_tkmath::predicates::roots::metric_bounds::CertifiedPolynomialMeasure;

pub(super) fn interpret(
    join: &JoinedPrismProof,
    directions: &JoinedPrismOrientationCertificate,
    measures: &JoinedPrismMeasures,
) -> Result<JoinedPrismVolume, JoinedPrismVolumeCause> {
    let factors = factors(directions);
    let sign = signed_integral_sign(factors)?;
    let source = measures.signed_volume().clone();
    let (enclosure, representative) = transport_scalar(&source, sign)?;
    Ok(JoinedPrismVolume {
        factors,
        representative,
        source,
        enclosure,
        binding: join.replay_bytes().into(),
        directions: directions.replay_bytes(),
        measures: measures.replay_bytes(),
    })
}
pub(in crate::sketch_region_prism) fn transport_scalar(
    source: &CertifiedPolynomialMeasure,
    sign: i8,
) -> Result<([f64; 2], f64), JoinedPrismVolumeCause> {
    let enclosure = transport_enclosure(source.enclosure(), sign)?;
    // Negation is exact; retaining the transported representative preserves its certified error.
    Ok((enclosure, f64::from(sign) * source.representative()))
}
fn factors(directions: &JoinedPrismOrientationCertificate) -> VolumeSignFactors {
    let (candidate_facing, boundary_traversal, selected_winding, increasing_parameter) =
        directions.winding().factors();
    VolumeSignFactors {
        candidate_facing,
        boundary_traversal,
        selected_winding,
        increasing_parameter,
        jacobian: directions.jacobian_sign(),
    }
}
pub(super) fn signed_integral_sign(
    factors: VolumeSignFactors,
) -> Result<i8, JoinedPrismVolumeCause> {
    let signs = [
        factors.candidate_facing,
        factors.boundary_traversal,
        factors.increasing_parameter,
        factors.jacobian,
    ];
    if signs.iter().any(|sign| !matches!(sign, -1 | 1)) || factors.selected_winding != 1 {
        return Err(JoinedPrismVolumeCause::SignFactors(factors));
    }
    if factors.increasing_parameter != factors.candidate_facing * factors.boundary_traversal {
        return Err(JoinedPrismVolumeCause::SignFactors(factors));
    }
    Ok(factors.increasing_parameter * factors.boundary_traversal * factors.jacobian)
}
pub(super) fn transport_enclosure(
    signed: [f64; 2],
    sign: i8,
) -> Result<[f64; 2], JoinedPrismVolumeCause> {
    let positive = if sign == 1 {
        signed
    } else {
        [-signed[1], -signed[0]]
    };
    if !matches!(sign, -1 | 1)
        || !positive.iter().all(|x| x.is_finite())
        || positive[0] < 0.
        || positive[0] > positive[1]
    {
        return Err(JoinedPrismVolumeCause::ScalarContradiction {
            signed_enclosure: signed,
            exact_sign: sign,
        });
    }
    Ok(positive)
}
