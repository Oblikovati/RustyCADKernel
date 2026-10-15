// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0084 steps 1-3: expected vertex incidences, the exact rotation system and its orbits.
//! A dart is one source traversed forward (native 0 to 1) or backward.
use super::super::binding::SegmentBinding;
use super::super::joins::{Expected, key};
use super::super::types::*;
use super::GraphRegionRequest;
use rustycad_tkmath::predicates::{OrientationSign, orient2d};
use std::cmp::Ordering;
use std::collections::BTreeMap;

const STAGE: LoopRegionStage = LoopRegionStage::Faces;

/// One directed source; `index` is the source's position in request order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct Dart {
    pub source: usize,
    pub forward: bool,
}

impl Dart {
    pub(super) fn reversed(self) -> Self {
        Self {
            forward: !self.forward,
            ..self
        }
    }
    /// Caller vertex at the start of the dart.
    pub(super) fn tail(self, ends: &[[u64; 2]]) -> u64 {
        ends[self.source][usize::from(!self.forward)]
    }
    /// Caller vertex at the end of the dart.
    pub(super) fn head(self, ends: &[[u64; 2]]) -> u64 {
        ends[self.source][usize::from(self.forward)]
    }
    pub(super) fn authored(self, binding: &SegmentBinding<'_>) -> AuthoredLoopUse {
        AuthoredLoopUse {
            source: binding.sources[self.source],
            reversed: !self.forward,
        }
    }
}

/// Outgoing darts of every vertex, counterclockwise from the positive chart x direction.
pub(super) type Rotation = BTreeMap<u64, Vec<Dart>>;

/// One vertex pair per source.
pub(super) fn require_ends(
    request: &GraphRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
) -> LoopRegionCheck<()> {
    if request.ends.len() == binding.sources.len() {
        return Ok(());
    }
    let (actual, expected) = (request.ends.len(), binding.sources.len());
    let quantity = "vertex pair per source";
    refuse(
        LoopRegionStage::Admission,
        LoopRegionCause::Count {
            quantity,
            actual,
            expected,
        },
    )
}

/// The contacts an embedded plane graph has: one per pair of source ends at one vertex id,
/// and the closure of a closed single source. A closed source must be alone at its vertex.
pub(super) fn incidence_contacts(
    request: &GraphRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
) -> LoopRegionCheck<Expected> {
    let ends = request.ends;
    let mut expected = Expected::new();
    for a in 0..ends.len() {
        let pa = binding.positions[a];
        if ends[a][0] == ends[a][1] {
            require_alone(ends, a, binding)?;
            expected.entry(key(pa, pa)).or_default().push([0, 1]);
        }
        for b in a + 1..ends.len() {
            let pb = binding.positions[b];
            for (ea, eb) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                if ends[a][ea] != ends[b][eb] {
                    continue;
                }
                let entry = if pa < pb { [ea, eb] } else { [eb, ea] };
                expected
                    .entry(key(pa, pb))
                    .or_default()
                    .push(entry.map(|e| e as u8));
            }
        }
    }
    Ok(expected)
}

fn require_alone(
    ends: &[[u64; 2]],
    closed: usize,
    binding: &SegmentBinding<'_>,
) -> LoopRegionCheck<()> {
    let vertex = ends[closed][0];
    let shared = (0..ends.len()).any(|other| other != closed && ends[other].contains(&vertex));
    if !shared {
        return Ok(());
    }
    let quantity = "closed source alone at its vertex";
    let source = binding.sources[closed];
    refuse(STAGE, LoopRegionCause::Binding { quantity, source })
}

/// Exact counterclockwise order of the darts leaving every vertex.
pub(super) fn rotation(
    request: &GraphRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
) -> LoopRegionCheck<Rotation> {
    let mut rotation = Rotation::new();
    for source in 0..request.ends.len() {
        for forward in [true, false] {
            let dart = Dart { source, forward };
            rotation
                .entry(dart.tail(request.ends))
                .or_default()
                .push(dart);
        }
    }
    for (vertex, darts) in rotation.iter_mut() {
        *darts = sorted_around(binding, *vertex, darts)?;
    }
    Ok(rotation)
}

/// The darts of one vertex in exact counterclockwise order; ties refuse at degree three
/// or more, and darts starting at different chart points refuse.
fn sorted_around(
    binding: &SegmentBinding<'_>,
    vertex: u64,
    darts: &[Dart],
) -> LoopRegionCheck<Vec<Dart>> {
    let legs = darts
        .iter()
        .map(|dart| leg(binding, *dart, vertex))
        .collect::<LoopRegionCheck<Vec<_>>>()?;
    let start = legs[0][0].map(f64::to_bits);
    if legs.iter().any(|l| l[0].map(f64::to_bits) != start) {
        refuse(STAGE, LoopRegionCause::VertexChart { vertex })?;
    }
    let mut order: Vec<usize> = (0..darts.len()).collect();
    order.sort_by(|a, b| compare_legs(legs[*a], legs[*b]));
    let tie = |pair: &[usize]| compare_legs(legs[pair[0]], legs[pair[1]]) == Ordering::Equal;
    if darts.len() > 2 && order.windows(2).any(tie) {
        refuse(STAGE, LoopRegionCause::TangentBranches { vertex })?;
    }
    Ok(order.into_iter().map(|index| darts[index]).collect())
}

/// The dart's start point and its first control distinct from it, which gives the exact
/// tangent direction of a positive-weight Bézier at its end.
fn leg(binding: &SegmentBinding<'_>, dart: Dart, vertex: u64) -> LoopRegionCheck<[[f64; 2]; 2]> {
    let controls = &binding.segments[dart.source].controls;
    let ordered: Vec<[f64; 2]> = match dart.forward {
        true => controls.clone(),
        false => controls.iter().rev().copied().collect(),
    };
    let start = ordered[0];
    let same = |p: &[f64; 2]| p.map(f64::to_bits) == start.map(f64::to_bits);
    match ordered.iter().find(|p| !same(p)) {
        Some(next) => Ok([start, *next]),
        None => refuse(STAGE, LoopRegionCause::DegenerateSource { vertex }),
    }
}

/// Angular order of two legs from one start point: upper half-plane `[0, pi)` first, then
/// counterclockwise within a half-plane by the exact orientation sign.
fn compare_legs(a: [[f64; 2]; 2], b: [[f64; 2]; 2]) -> Ordering {
    let half = |[start, next]: [[f64; 2]; 2]| {
        let upper = next[1] > start[1] || (next[1] == start[1] && next[0] > start[0]);
        u8::from(!upper)
    };
    half(a).cmp(&half(b)).then_with(|| {
        match orient2d(a[0], a[1], b[1]).expect("finite census controls") {
            OrientationSign::Positive => Ordering::Less,
            OrientationSign::Zero => Ordering::Equal,
            OrientationSign::Negative => Ordering::Greater,
        }
    })
}

/// Face orbits: after arriving along a dart, leave by the clockwise neighbour of its
/// reverse, which keeps the face on the left. Orbits start at their first unvisited dart in
/// source order, forward before backward.
pub(super) fn orbits(request: &GraphRegionRequest<'_, '_>, rotation: &Rotation) -> Vec<Vec<Dart>> {
    let next = |dart: Dart| {
        let around = &rotation[&dart.head(request.ends)];
        let back = around
            .iter()
            .position(|d| *d == dart.reversed())
            .expect("own reverse");
        around[(back + around.len() - 1) % around.len()]
    };
    let mut visited = std::collections::BTreeSet::new();
    let mut orbits = Vec::new();
    for source in 0..request.ends.len() {
        for forward in [true, false] {
            let first = Dart { source, forward };
            if visited.contains(&first) {
                continue;
            }
            let mut orbit = vec![first];
            visited.insert(first);
            let mut dart = next(first);
            while dart != first {
                visited.insert(dart);
                orbit.push(dart);
                dart = next(dart);
            }
            orbits.push(orbit);
        }
    }
    orbits
}
