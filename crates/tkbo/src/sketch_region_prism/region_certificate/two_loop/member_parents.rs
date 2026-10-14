// SPDX-License-Identifier: GPL-2.0-only
//! Parent conversion binds actual native identities, preserving every row and relation class.
use super::{failure::*, geometry, member_types::*, request::*, work::*};
use crate::sketch_region::{OriginalMemberIdentity, PartitionOccurrenceIdentity};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{edge, vertex},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(super) struct OriginalUseBinding {
    pub actual: OriginalMemberIdentity,
    pub key: TwoLoopOriginalUse,
}
pub(super) fn originals<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Vec<OriginalUseBinding>> {
    let mut result = Vec::new();
    for source in request.sources {
        result.push(binding(
            source.identity,
            source.shape,
            TwoLoopMemberRole::Boundary,
        ));
        let native = edge(source.shape).map_err(|e| work.failure(TwoLoopRegionCause::Native(e)))?;
        for endpoint in geometry::children(source.shape, ShapeEnum::Vertex, 2, work)? {
            let point =
                vertex(&endpoint).map_err(|e| work.failure(TwoLoopRegionCause::Native(e)))?;
            let parameter = BRepTool::parameter(&point, &native)
                .map_err(|e| work.failure(TwoLoopRegionCause::Native(e)))?;
            let role = endpoint_role(parameter, work)?;
            result.push(binding(source.identity, &endpoint, role));
        }
    }
    work.count("complete original directed use roster", result.len(), 6)?;
    Ok(result)
}
fn binding(
    source: crate::sketch_region::RegionSourceId,
    shape: &rustycad_tkbrep::topods::Shape,
    role: TwoLoopMemberRole,
) -> OriginalUseBinding {
    let actual = PartitionOccurrenceIdentity::capture(shape);
    OriginalUseBinding {
        actual: OriginalMemberIdentity {
            source,
            topology_address: actual.topology_address,
            location: actual.location,
            orientation: actual.orientation,
        },
        key: TwoLoopOriginalUse { source, role },
    }
}
pub(super) fn endpoint_role<'a>(
    parameter: f64,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopMemberRole> {
    match parameter {
        0. => Ok(TwoLoopMemberRole::NativeStart),
        1. => Ok(TwoLoopMemberRole::NativeEnd),
        actual => Err(work.failure(TwoLoopRegionCause::Scalar {
            operand: "bound native endpoint parameter",
            actual,
            expected: "exact endpoint 0 or 1",
        })),
    }
}
pub(super) fn parents<'a>(
    rows: &[OriginalMemberIdentity],
    bindings: &[OriginalUseBinding],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Vec<TwoLoopOriginalUse>> {
    work.units("kinded original parent", rows.len())?;
    let mut result = Vec::new();
    for row in rows {
        let matches: Vec<_> = bindings.iter().filter(|b| b.actual == *row).collect();
        work.count("unique actual original directed parent", matches.len(), 1)?;
        result.push(matches[0].key);
    }
    result.sort();
    Ok(result)
}
