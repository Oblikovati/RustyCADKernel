// SPDX-License-Identifier: GPL-2.0-only
//! Bind every authored source to exactly one unsplit full-range census segment.
use super::types::*;
use crate::sketch_region::{ArrangementSource, RegionSourceId};
use crate::sketch_region_candidates::CandidateSegment;
use crate::sketch_region_contact_transport::PairedContactTransaction;
use std::collections::BTreeSet;

/// Census segment index of every request source, in request source order.
pub(super) struct SegmentBinding<'a> {
    pub sources: Vec<RegionSourceId>,
    pub segments: Vec<&'a CandidateSegment>,
    /// Position of each bound segment in the census roster (the pair-domain index).
    pub positions: Vec<usize>,
}
impl<'a> SegmentBinding<'a> {
    /// Position of `source` in the request roster.
    pub(super) fn index(&self, source: RegionSourceId) -> Option<usize> {
        self.sources.iter().position(|row| *row == source)
    }
    /// Census roster position of `source`, the index used by census pair domains.
    pub(super) fn position(&self, source: RegionSourceId) -> usize {
        self.positions[self.index(source).expect("bound source")]
    }
    /// Request source at census roster position `position`.
    pub(super) fn source_at(&self, position: usize) -> Option<RegionSourceId> {
        let index = self.positions.iter().position(|p| *p == position)?;
        Some(self.sources[index])
    }
    pub(super) fn segment(&self, source: RegionSourceId) -> &'a CandidateSegment {
        self.segments[self.index(source).expect("bound source")]
    }
}
const STAGE: LoopRegionStage = LoopRegionStage::Admission;

pub(super) fn bind<'a>(request: &LoopRegionRequest<'a, '_>) -> LoopRegionCheck<SegmentBinding<'a>> {
    let binding = bind_segments(request.bridge, request.sources, request.options.version)?;
    authored_uses(request, &binding)?;
    Ok(binding)
}
/// Version check and the one unsplit full-range census segment of every source; the
/// shared first stage of the loop proof and of the plane graph proof (ADR 0084).
pub(super) fn bind_segments<'a>(
    bridge: &'a PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    version: u32,
) -> LoopRegionCheck<SegmentBinding<'a>> {
    if version != 1 {
        let cause = LoopRegionCause::Version {
            actual: version,
            expected: 1,
        };
        return refuse(STAGE, cause);
    }
    let census = &bridge.census;
    count(
        "census segments",
        census.inputs.segments.len(),
        sources.len(),
    )?;
    count(
        "census originals",
        census.inputs.originals.len(),
        sources.len(),
    )?;
    let mut binding = SegmentBinding {
        sources: Vec::new(),
        segments: Vec::new(),
        positions: Vec::new(),
    };
    for source in sources {
        let (position, row) = segment(bridge, source.identity)?;
        binding.segments.push(row);
        binding.positions.push(position);
        binding.sources.push(source.identity);
    }
    Ok(binding)
}
fn count(quantity: &'static str, actual: usize, expected: usize) -> LoopRegionCheck<()> {
    if actual == expected {
        return Ok(());
    }
    refuse(
        STAGE,
        LoopRegionCause::Count {
            quantity,
            actual,
            expected,
        },
    )
}
fn segment(
    bridge: &PairedContactTransaction,
    source: RegionSourceId,
) -> LoopRegionCheck<(usize, &CandidateSegment)> {
    let inputs = &bridge.census.inputs;
    let originals: Vec<_> = inputs
        .originals
        .iter()
        .filter(|o| o.original.source == source)
        .collect();
    if originals.len() != 1 {
        return refuse(
            STAGE,
            LoopRegionCause::Binding {
                quantity: "one original capture",
                source,
            },
        );
    }
    let parent = std::slice::from_ref(&originals[0].original);
    let matches: Vec<_> = inputs
        .segments
        .iter()
        .enumerate()
        .filter(|(_, s)| s.parents.as_slice() == parent)
        .collect();
    if matches.len() != 1 {
        return refuse(
            STAGE,
            LoopRegionCause::Binding {
                quantity: "one unsplit segment",
                source,
            },
        );
    }
    let (position, row) = matches[0];
    let full = row.interval == [0., 1.] && row.original_interval == [0., 1.] && !row.reversed;
    if !full {
        let quantity = "forward full native [0,1] segment";
        return refuse(STAGE, LoopRegionCause::Binding { quantity, source });
    }
    Ok((position, row))
}
/// Every source is used exactly once over all loops, and every loop is nonempty.
fn authored_uses(
    request: &LoopRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
) -> LoopRegionCheck<()> {
    let mut used = BTreeSet::new();
    for authored in request.loops {
        count("uses in authored loop", authored.uses.len().min(1), 1)?;
        for directed in &authored.uses {
            let source = directed.source;
            if binding.index(source).is_none() || !used.insert(source.0) {
                let quantity = "one use of a request source";
                return refuse(STAGE, LoopRegionCause::Binding { quantity, source });
            }
        }
    }
    count("used request sources", used.len(), binding.sources.len())
}
