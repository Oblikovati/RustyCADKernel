// SPDX-License-Identifier: GPL-2.0-only
//! Owned operands of the existing original-source singleton query, without a new mapper.
use crate::sketch_region::OriginalMemberIdentity;
use crate::sketch_region_source_conformance::{CapturedPlaneSupport, CapturedSpaceRepresentation};
use rustycad_tkmath::predicates::roots::{
    DyadicBound, RootIsolationOptions, metric_bounds::MetricParameterMap,
};

#[derive(Clone)]
pub(crate) struct ContactPointQuerySnapshot {
    pub original: OriginalMemberIdentity,
    pub representation: CapturedSpaceRepresentation,
    pub copied: CapturedSpaceRepresentation,
    pub pcurve: crate::sketch_region_geometry_capture::CapturedMetricCurve<2>,
    pub captured: crate::sketch_region_geometry_capture::CapturedMetricCurve<2>,
    pub support: CapturedPlaneSupport,
    pub copied_occurrence: crate::sketch_region::PartitionOccurrenceIdentity,
    pub parents: Vec<OriginalMemberIdentity>,
    pub native_interval: [f64; 2],
    pub basis_interval: [f64; 2],
    pub maps: [MetricParameterMap; 2],
    pub interval: [f64; 2],
    pub bounds: [DyadicBound; 2],
    pub roots: RootIsolationOptions,
    pub requested_world_limit: f64,
}
