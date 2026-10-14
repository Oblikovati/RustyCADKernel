// SPDX-License-Identifier: GPL-2.0-only
//! Literal symbolic domain only: this fixture is not a successfully published geometric body.
use super::{
    super::{body_keys::*, *},
    checks,
    types::*,
};
use crate::sketch_region_prism::frozen::{
    AuthoredPrismId, FrozenPrismKey, FrozenSweepRole, FrozenUseKey,
};
use FrozenSweepRole::{First, Generated, Last};
use SemanticNameRole::*;
use SemanticNameStage::{Output, Selected, SweepBase};

pub(super) fn plan() -> SingleBoundaryNamePlan {
    SingleBoundaryNamePlan::v1(
        SingleBoundaryIdentities {
            operation: CallerOperationId(257),
            boundary: CallerBoundaryId(0),
            closure: CallerVertexId(2052),
            start: CallerDirectedUseId(2309),
            end: CallerDirectedUseId(19),
        },
        SemanticSelectionIntent::BoundedRegionContaining { seed: [-0., 1.25] },
    )
    .unwrap()
}
pub(super) fn key(source: u64, role: FrozenSweepRole, kind: u8) -> FrozenPrismKey {
    FrozenPrismKey {
        operation: AuthoredPrismId(257),
        source: AuthoredPrismId(source),
        role,
        kind,
    }
}
fn row(
    source: u64,
    stage: SemanticNameStage,
    source_role: SemanticNameRole,
    role: FrozenSweepRole,
    kind: u8,
    output: Option<SemanticNameRole>,
) -> SemanticMemberBinding {
    SemanticMemberBinding {
        frozen: key(source, role, kind),
        semantic: literal_member(stage, source_role, role, kind),
        output_slot: output.map(|role| SemanticNameSlot {
            stage: Output,
            role,
        }),
    }
}
pub(super) fn members() -> Vec<SemanticMemberBinding> {
    let mut rows = vec![
        row(1544, Selected, BoundedRegion, Generated, 2, Some(Solid)),
        row(1544, Selected, BoundedRegion, Generated, 3, Some(Shell)),
    ];
    rows.extend(face_rows());
    rows.extend(wire_rows());
    rows.extend(edge_vertex_rows());
    rows
}
fn face_rows() -> Vec<SemanticMemberBinding> {
    vec![
        row(2056, SweepBase, BoundedRegion, First, 4, Some(FirstCap)),
        row(2056, SweepBase, BoundedRegion, Last, 4, Some(LastCap)),
        row(2051, SweepBase, Boundary, Generated, 4, Some(Side)),
    ]
}
fn wire_rows() -> Vec<SemanticMemberBinding> {
    vec![
        row(2058, SweepBase, BoundedLoop, First, 5, None),
        row(2058, SweepBase, BoundedLoop, Last, 5, None),
        row(2051, SweepBase, Boundary, Generated, 5, None),
    ]
}
fn edge_vertex_rows() -> Vec<SemanticMemberBinding> {
    vec![
        row(2051, SweepBase, Boundary, First, 6, Some(FirstRim)),
        row(2051, SweepBase, Boundary, Last, 6, Some(LastRim)),
        row(2052, SweepBase, ClosureVertex, Generated, 6, Some(Seam)),
        row(2052, SweepBase, ClosureVertex, First, 7, Some(FirstVertex)),
        row(2052, SweepBase, ClosureVertex, Last, 7, Some(LastVertex)),
    ]
}
pub(super) fn uses() -> Vec<SemanticUseBinding> {
    let mut rows = Vec::new();
    for role in [First, Last] {
        let rim = if role == First {
            FirstRimUse
        } else {
            LastRimUse
        };
        let path = face_path(key(2056, role, 4), key(2058, role, 5));
        append_edge(
            &mut rows,
            &path,
            key(2051, role, 6),
            rim,
            &[(role, StartUse), (role, EndUse)],
        );
    }
    side_uses(&mut rows);
    rows
}
fn side_uses(rows: &mut Vec<SemanticUseBinding>) {
    let path = face_path(key(2051, Generated, 4), key(2051, Generated, 5));
    side_rims(rows, &path);
    side_seams(rows, &path);
}
fn face_path(
    face: FrozenPrismKey,
    wire: FrozenPrismKey,
) -> Vec<(FrozenPrismKey, SemanticNameRole)> {
    vec![
        (key(1544, Generated, 2), SolidUse),
        (key(1544, Generated, 3), ShellUse),
        (face, FaceUse),
        (wire, LoopUse),
    ]
}
fn append_edge(
    rows: &mut Vec<SemanticUseBinding>,
    path: &[(FrozenPrismKey, SemanticNameRole)],
    edge: FrozenPrismKey,
    use_role: SemanticNameRole,
    endpoints: &[(FrozenSweepRole, SemanticNameRole)],
) {
    let mut branch = path.to_vec();
    branch.push((edge, use_role));
    for &(role, endpoint) in endpoints {
        let mut complete = branch.clone();
        complete.push((key(2052, role, 7), endpoint));
        append_prefixes(rows, &complete);
    }
}
fn append_prefixes(
    rows: &mut Vec<SemanticUseBinding>,
    path: &[(FrozenPrismKey, SemanticNameRole)],
) {
    for end in 1..=path.len() {
        let row = explicit_use(&path[..end]);
        if !rows.iter().any(|existing| existing.frozen == row.frozen) {
            rows.push(row);
        }
    }
}
pub(super) fn checked(
    members: Vec<SemanticMemberBinding>,
    uses: Vec<SemanticUseBinding>,
) -> MapCheck<ResolvedPrismNames> {
    let actual_members: Vec<_> = self::members().iter().map(|row| row.frozen).collect();
    let actual_uses: Vec<_> = self::uses().iter().map(|row| row.frozen.clone()).collect();
    checks::checked(
        plan(),
        NameMapDomain {
            members: &actual_members,
            uses: &actual_uses,
            publication: b"literal-symbolic-domain-only",
        },
        members,
        uses,
    )
}
pub(super) fn names() -> ResolvedPrismNames {
    checked(members(), uses()).unwrap()
}

fn literal_member(
    stage: SemanticNameStage,
    role: SemanticNameRole,
    sweep_role: FrozenSweepRole,
    kind: u8,
) -> SemanticMemberKey {
    SemanticMemberKey {
        operation: CallerOperationId(257),
        source: CallerBoundaryId(0),
        source_member: SemanticNameSlot { stage, role },
        sweep_role,
        kind,
    }
}
fn side_rims(rows: &mut Vec<SemanticUseBinding>, path: &[(FrozenPrismKey, SemanticNameRole)]) {
    append_edge(
        rows,
        path,
        key(2051, First, 6),
        FirstRimUse,
        &[(First, StartUse), (First, EndUse)],
    );
    append_edge(
        rows,
        path,
        key(2051, Last, 6),
        LastRimUse,
        &[(Last, StartUse), (Last, EndUse)],
    );
}
fn side_seams(rows: &mut Vec<SemanticUseBinding>, path: &[(FrozenPrismKey, SemanticNameRole)]) {
    append_edge(
        rows,
        path,
        key(2052, Generated, 6),
        SeamStartUse,
        &[(First, StartUse), (Last, StartUse)],
    );
    append_edge(
        rows,
        path,
        key(2052, Generated, 6),
        SeamEndUse,
        &[(First, EndUse), (Last, EndUse)],
    );
}
fn explicit_use(path: &[(FrozenPrismKey, SemanticNameRole)]) -> SemanticUseBinding {
    let frozen = FrozenUseKey(
        path.iter()
            .map(|(key, role)| (*key, AuthoredPrismId(2304 + *role as u64)))
            .collect(),
    );
    let semantic = SemanticUseKey(
        path.iter()
            .map(|(key, role)| explicit_step(*key, *role))
            .collect(),
    );
    SemanticUseBinding { frozen, semantic }
}
fn explicit_step(key: FrozenPrismKey, role: SemanticNameRole) -> SemanticUseStep {
    SemanticUseStep {
        member: members()
            .into_iter()
            .find(|row| row.frozen == key)
            .unwrap()
            .semantic,
        directed_use: SemanticNameSlot {
            stage: Output,
            role,
        },
    }
}
