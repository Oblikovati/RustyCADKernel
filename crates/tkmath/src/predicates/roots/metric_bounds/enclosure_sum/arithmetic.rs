// SPDX-License-Identifier: GPL-2.0-only
use super::*;
type SumResult<T> = Result<T, EnclosureSumCause>;

pub(super) fn sum(
    components: &[[f64; 2]],
    options: EnclosureSumOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> SumResult<CertifiedEnclosureSum> {
    checkpoint(cancelled)?;
    admit(components.len(), options)?;
    let bounds = accumulate(components, cancelled)?;
    let report = publish(components, bounds, options)?;
    checkpoint(cancelled)?;
    Ok(report)
}
fn publish(
    components: &[[f64; 2]],
    bounds: [ExactRatio; 2],
    options: EnclosureSumOptions,
) -> SumResult<CertifiedEnclosureSum> {
    let enclosure = [
        rounded(&bounds[0], "sum lower")?[0],
        rounded(&bounds[1], "sum upper")?[1],
    ];
    let width =
        ExactRatio::from_finite(enclosure[1]).subtract(&ExactRatio::from_finite(enclosure[0]));
    let error = rounded(&width, "sum width")?[1];
    require_error(error, options.absolute_error_limit)?;
    Ok(CertifiedEnclosureSum {
        components: components.to_vec(),
        enclosure,
        error,
        options,
    })
}

fn admit(count: usize, options: EnclosureSumOptions) -> SumResult<()> {
    if options.behavior_version != 1
        || options.max_components > 64
        || !options.absolute_error_limit.is_finite()
        || options.absolute_error_limit < 0.
    {
        return Err(EnclosureSumCause::Policy(options));
    }
    if count > usize::from(options.max_components) {
        return Err(EnclosureSumCause::ComponentCount {
            actual: count,
            maximum: options.max_components,
        });
    }
    Ok(())
}
fn accumulate(
    components: &[[f64; 2]],
    cancelled: &mut impl FnMut() -> bool,
) -> SumResult<[ExactRatio; 2]> {
    let mut sum = [ExactRatio::from_finite(0.), ExactRatio::from_finite(0.)];
    for (index, supplied) in components.iter().copied().enumerate() {
        checkpoint(cancelled)?;
        require_component(index, supplied)?;
        sum[0] = sum[0].add(&ExactRatio::from_finite(supplied[0]));
        sum[1] = sum[1].add(&ExactRatio::from_finite(supplied[1]));
    }
    Ok(sum)
}
fn require_component(index: usize, supplied: [f64; 2]) -> SumResult<()> {
    if supplied.iter().any(|value| !value.is_finite())
        || supplied[0] < 0.
        || supplied[0] > supplied[1]
    {
        return Err(EnclosureSumCause::Component { index, supplied });
    }
    Ok(())
}
fn rounded(exact: &ExactRatio, quantity: &'static str) -> SumResult<[f64; 2]> {
    outward_binary64(exact).ok_or_else(|| EnclosureSumCause::Unrepresentable {
        quantity,
        exact_value: format!("{exact:?}"),
    })
}
fn require_error(achieved: f64, requested: f64) -> SumResult<()> {
    if achieved > requested {
        return Err(EnclosureSumCause::ErrorLimit {
            requested,
            achieved,
        });
    }
    Ok(())
}
