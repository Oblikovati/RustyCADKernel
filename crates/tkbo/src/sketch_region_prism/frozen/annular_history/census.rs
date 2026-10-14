// SPDX-License-Identifier: GPL-2.0-only
//! Version two admits exactly the closed one-edge loops already sealed by the two-loop proof.
use super::*;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Admit one scope census: bounded rows, exact closed-loop kind counts and F/R orientations.
pub(super) fn admit_scope(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    operands: &AnnularScopeOperands,
) -> HistoryCheck<()> {
    let (limits, rows) = (request.requested.limits, &operands.rows);
    let maximum = limits.maximum_uses;
    let counted = limits.count("actual history scope census", rows.len(), maximum);
    counted.map_err(PrismHistoryCause::Bounds)?;
    let shape = expected_shape(request, operands.scope)?;
    admit_kinds(&shape, rows)?;
    let vertices = physical_vertices(rows);
    let required = "one physical closure vertex per loop";
    history_count(required, vertices, shape.loops)?;
    for row in rows {
        forward_reversed_history_use(row.occurrence.orientation)?;
    }
    Ok(())
}
fn admit_kinds(shape: &ScopeShape, rows: &[RegionObservedContext]) -> HistoryCheck<()> {
    for (kind, expected) in shape.kinds() {
        let actual = rows.iter().filter(|r| r.kind == kind).count();
        let required = "actual uses of required closed-loop scope kind";
        history_count(required, actual, expected)?;
    }
    let required = "complete closed-loop scope census";
    history_count(required, rows.len(), shape.total())
}
/// Expected container/face/loop structure of one scope.
pub(super) struct ScopeShape {
    pub container: bool,
    pub face: bool,
    pub loops: usize,
}
impl ScopeShape {
    fn kinds(&self) -> [(ShapeEnum, usize); 5] {
        [
            (ShapeEnum::Compound, usize::from(self.container)),
            (ShapeEnum::Face, usize::from(self.face)),
            (ShapeEnum::Wire, if self.face { self.loops } else { 0 }),
            (ShapeEnum::Edge, self.loops),
            (ShapeEnum::Vertex, 2 * self.loops),
        ]
    }
    fn total(&self) -> usize {
        self.kinds().iter().map(|(_, n)| n).sum()
    }
}
fn expected_shape(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    scope: AnnularHistoryScope,
) -> HistoryCheck<ScopeShape> {
    let loops = request.region.sources.len();
    let shape = |container, face, loops| ScopeShape {
        container,
        face,
        loops,
    };
    Ok(match scope {
        AnnularHistoryScope::Original(_) | AnnularHistoryScope::InputCopy(_) => {
            shape(false, false, 1)
        }
        AnnularHistoryScope::Partition | AnnularHistoryScope::CandidateCopy => {
            shape(true, false, loops)
        }
        AnnularHistoryScope::SweepBase => shape(false, true, loops),
        AnnularHistoryScope::Selected(name) | AnnularHistoryScope::Rejected(name) => {
            shape(false, true, authored_candidate(request, name)?.wires.len())
        }
    })
}
/// Borrow the authored candidate binding by its unique name.
pub(super) fn authored_candidate<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    name: u64,
) -> HistoryCheck<&'a TwoLoopCandidateBinding> {
    let candidates = &request.region.identities.candidates;
    let rows: Vec<_> = candidates.iter().filter(|c| c.name == name).collect();
    history_count("one authored candidate binding per name", rows.len(), 1)?;
    Ok(rows[0])
}
fn physical_vertices(rows: &[RegionObservedContext]) -> usize {
    let mut vertices: Vec<&crate::sketch_region::PartitionOccurrenceIdentity> = Vec::new();
    for row in rows.iter().filter(|row| row.kind == ShapeEnum::Vertex) {
        let known = vertices
            .iter()
            .any(|other| same_definition(other, &row.occurrence));
        if !known {
            vertices.push(&row.occurrence);
        }
    }
    vertices.len()
}
