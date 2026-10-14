// SPDX-License-Identifier: GPL-2.0-only
use super::*;

pub(super) fn chart(identity: &AnnularChartIdentity, bytes: &mut Vec<u8>) -> ReplayCheck {
    context(&identity.source, bytes)?;
    context(&identity.output, bytes)?;
    bytes.push(identity.role as u8);
    Ok(())
}
pub(super) fn domain(
    value: &AnnularFaceDomain,
    binding: &AnnularPrismBinding,
    bytes: &mut Vec<u8>,
) -> ReplayCheck {
    chart(&value.identity, bytes)?;
    kind(&value.kind, bytes)?;
    count(value.trims.len(), bytes);
    for value in &value.trims {
        trim(value, binding, bytes)?;
    }
    Ok(())
}
fn kind(value: &AnnularDomainKind, bytes: &mut Vec<u8>) -> ReplayCheck {
    match value {
        AnnularDomainKind::Cap { .. } => cap_kind(value, bytes),
        AnnularDomainKind::Side { .. } => side_kind(value, bytes)?,
    }
    Ok(())
}
fn cap_kind(value: &AnnularDomainKind, bytes: &mut Vec<u8>) {
    let AnnularDomainKind::Cap {
        region,
        outer,
        inner,
        normalized_sweep,
    } = value
    else {
        unreachable!("cap variant dispatched")
    };
    cap(*region, outer.0, inner.0, *normalized_sweep, bytes);
}
fn side_kind(value: &AnnularDomainKind, bytes: &mut Vec<u8>) -> ReplayCheck {
    let AnnularDomainKind::Side {
        source,
        boundary,
        native,
        stored_sweep,
        normalized_sweep,
        height,
    } = value
    else {
        unreachable!("side variant dispatched")
    };
    side(
        source.0,
        boundary,
        [*native, *stored_sweep, *normalized_sweep],
        *height,
        bytes,
    )
}
fn cap(region: u64, outer: u64, inner: u64, sweep: f64, bytes: &mut Vec<u8>) {
    bytes.push(0);
    bytes.extend(region.to_le_bytes());
    bytes.extend(outer.to_le_bytes());
    bytes.extend(inner.to_le_bytes());
    numbers([sweep], bytes);
}
fn side(
    source: u64,
    boundary: &RegionObservedContext,
    ranges: [[f64; 2]; 3],
    height: f64,
    bytes: &mut Vec<u8>,
) -> ReplayCheck {
    bytes.push(1);
    bytes.extend(source.to_le_bytes());
    context(boundary, bytes)?;
    numbers(ranges.into_iter().flatten(), bytes);
    numbers([height], bytes);
    Ok(())
}
pub(super) fn trim(
    value: &AnnularAuthoredTrim,
    binding: &AnnularPrismBinding,
    bytes: &mut Vec<u8>,
) -> ReplayCheck {
    let owner = bound_trim_owner(value, binding)?;
    chart(&owner.identity, bytes)?;
    context(&value.identity.source, bytes)?;
    context(&value.identity.output, bytes)?;
    bytes.push(value.identity.role as u8);
    numbers(value.operand.interval, bytes);
    // Parent classes/multiplicities are in the binding block; this full-context edge selects them.
    count(value.operand.parents.len(), bytes);
    Ok(())
}
fn bound_trim_owner<'a>(
    value: &AnnularAuthoredTrim,
    binding: &'a AnnularPrismBinding,
) -> ReplayCheck<&'a AnnularBoundChart> {
    let matches = matching_trim_owners(value, binding);
    let [owner] = matches.as_slice() else {
        return Err(Box::new(AnnularBindingCause::Count {
            required: "one full-context source/face/trim/parent operand in binding replay",
            actual: matches.len(),
            expected: 1,
        }));
    };
    Ok(*owner)
}
fn matching_trim_owners<'a>(
    value: &AnnularAuthoredTrim,
    binding: &'a AnnularPrismBinding,
) -> Vec<&'a AnnularBoundChart> {
    binding
        .charts()
        .iter()
        .flat_map(|chart| {
            chart
                .chart
                .captured
                .trims
                .iter()
                .filter(move |trim| same_bound_trim(value, chart, trim))
                .map(move |_| chart)
        })
        .collect()
}
fn same_bound_trim(
    value: &AnnularAuthoredTrim,
    chart: &AnnularBoundChart,
    trim: &crate::sketch_region_prism::surface_charts::CapturedPrismTrim,
) -> bool {
    // Kernel #10: both cap rims share a role; identity and operands must select the same row.
    chart.trims.contains(&value.identity)
        && trim.source == value.identity.source.occurrence
        && trim.output == value.identity.output.occurrence
        && trim.role == value.identity.role
        && super::super::super::trim_incidence::retain_trim(&chart.chart, trim) == value.operand
}
