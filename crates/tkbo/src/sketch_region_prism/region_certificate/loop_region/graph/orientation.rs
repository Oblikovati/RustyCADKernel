// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0084 step 3: the certified sign of every orbit's Green integral (ADR 0066 part A).
//! An orbit that is the exact reverse of an earlier one takes the negated sign.
use super::super::binding::SegmentBinding;
use super::super::measure::curves;
use super::super::types::*;
use super::GraphRegionRequest;
use super::components::Graph;
use super::darts::Dart;
use rustycad_tkmath::predicates::roots::metric_bounds::certify_rational_boundary_integral;
use std::collections::BTreeMap;

/// `+1` or `-1` per orbit; an enclosure containing zero is refused.
pub(super) fn signs(
    request: &GraphRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
    graph: &Graph,
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<Vec<i8>> {
    let mut known: BTreeMap<Vec<Dart>, i8> = BTreeMap::new();
    let mut signs = Vec::with_capacity(graph.orbits.len());
    for (index, orbit) in graph.orbits.iter().enumerate() {
        let reverse = canonical(orbit.iter().map(|dart| dart.reversed()));
        let sign = match known.get(&reverse) {
            Some(sign) => -sign,
            None => certified_sign(request, binding, index, orbit, cancelled)?,
        };
        known.insert(canonical(orbit.iter().copied()), sign);
        signs.push(sign);
    }
    Ok(signs)
}

fn canonical(darts: impl Iterator<Item = Dart>) -> Vec<Dart> {
    let mut darts: Vec<Dart> = darts.collect();
    darts.sort_unstable();
    darts
}

fn certified_sign(
    request: &GraphRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
    index: usize,
    orbit: &[Dart],
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<i8> {
    let uses: Vec<AuthoredLoopUse> = orbit.iter().map(|d| d.authored(binding)).collect();
    let occurrences = curves(binding, &uses);
    let mut stopped = false;
    let integral = certify_rational_boundary_integral(&occurrences, request.options.area, || {
        stopped = stopped || cancelled();
        stopped
    });
    let stage = LoopRegionStage::Faces;
    let measure = match integral {
        Ok(measure) => measure,
        Err(_) if stopped => return refuse(stage, LoopRegionCause::Cancelled),
        Err(error) => {
            let cause = LoopRegionCause::Area {
                region: index as u64 + 1,
                curve_index: error.curve_index,
                cause: error.to_string(),
            };
            return refuse(stage, cause);
        }
    };
    match measure.enclosure() {
        [lower, _] if lower > 0. => Ok(1),
        [_, upper] if upper < 0. => Ok(-1),
        enclosure => refuse(
            stage,
            LoopRegionCause::OrbitOrientation {
                orbit: index,
                enclosure,
            },
        ),
    }
}
