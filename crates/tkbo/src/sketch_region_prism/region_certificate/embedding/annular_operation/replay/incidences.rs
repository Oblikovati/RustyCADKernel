// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::PrismIncidenceConclusion;
use super::*;

pub(super) fn encode(
    value: &AnnularFaceIncidence,
    binding: &AnnularPrismBinding,
    bytes: &mut Vec<u8>,
) -> ReplayCheck {
    domains::chart(&value.first, bytes)?;
    domains::chart(&value.second, bytes)?;
    match &value.conclusion {
        AnnularIncidenceConclusion::InjectiveAnnularDomain => bytes.push(0),
        AnnularIncidenceConclusion::DisjointCaps => bytes.push(1),
        AnnularIncidenceConclusion::DisjointCertifiedBoundaries => bytes.push(2),
        AnnularIncidenceConclusion::TrimIncidence {
            first,
            second,
            conclusion,
        } => {
            bytes.push(3);
            domains::trim(first, binding, bytes)?;
            domains::trim(second, binding, bytes)?;
            scalar_conclusion(first, second, conclusion, bytes)?;
        }
    }
    Ok(())
}
fn scalar_conclusion(
    first: &AnnularAuthoredTrim,
    second: &AnnularAuthoredTrim,
    value: &PrismIncidenceConclusion,
    bytes: &mut Vec<u8>,
) -> ReplayCheck {
    let (first_operand, second_operand) = conclusion_operands(value)?;
    if first_operand != &first.operand || second_operand != &second.operand {
        return Err(unbound_conclusion());
    }
    conclusion_scalars(value, bytes)
}
fn conclusion_scalars(value: &PrismIncidenceConclusion, bytes: &mut Vec<u8>) -> ReplayCheck {
    match value {
        PrismIncidenceConclusion::InjectiveModuloSeam {
            native_endpoints,
            normalized_sweep,
            ..
        } => seam_scalars(*native_endpoints, *normalized_sweep, bytes),
        PrismIncidenceConclusion::ExactlyRim {
            native,
            normalized_sweep,
            ..
        } => rim_scalars(*native, *normalized_sweep, bytes),
        _ => return Err(unbound_conclusion()),
    }
    Ok(())
}
fn seam_scalars(native: [f64; 2], sweep: [f64; 2], bytes: &mut Vec<u8>) {
    bytes.push(0);
    numbers(native.into_iter().chain(sweep), bytes);
}
fn rim_scalars(native: [f64; 2], sweep: f64, bytes: &mut Vec<u8>) {
    bytes.push(1);
    numbers(native, bytes);
    numbers([sweep], bytes);
}
fn conclusion_operands(
    value: &PrismIncidenceConclusion,
) -> ReplayCheck<(
    &super::super::super::PrismAuthoredTrimUse,
    &super::super::super::PrismAuthoredTrimUse,
)> {
    match value {
        PrismIncidenceConclusion::InjectiveModuloSeam { start, end, .. } => Ok((start, end)),
        PrismIncidenceConclusion::ExactlyRim { cap, side, .. } => Ok((cap, side)),
        _ => Err(unbound_conclusion()),
    }
}
fn unbound_conclusion() -> Box<AnnularBindingCause<'static>> {
    Box::new(AnnularBindingCause::Count {
        required: "trim incidence conclusion bound to both complete directed operands",
        actual: 0,
        expected: 1,
    })
}
