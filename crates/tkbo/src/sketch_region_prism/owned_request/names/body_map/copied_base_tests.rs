// SPDX-License-Identifier: GPL-2.0-only
//! Literal dictionary applicability checks; actual production issuance is tested at the resolver entry.
use super::{super::*, expansion, test_fixture::*};
use crate::sketch_region_prism::frozen::FrozenSweepRole;
#[test]
fn copied_base_root_definitions_require_v2_and_keep_exact_stage_meaning() {
    let old = plan();
    let new = SingleBoundaryNamePlan::v2(old.identities(), old.selection()).unwrap();
    for (kind, expected) in [(2, SemanticNameRole::Solid), (3, SemanticNameRole::Shell)] {
        let actual = key(2056, FrozenSweepRole::Generated, kind);
        assert!(expansion::member(&old, actual).is_err());
        let member = expansion::member(&new, actual).unwrap();
        assert_eq!(member.source_member.stage, SemanticNameStage::SweepBase);
        let output = expansion::output(&new, actual, member.source_member)
            .unwrap()
            .unwrap();
        assert_eq!(
            output,
            SemanticNameSlot {
                stage: SemanticNameStage::Output,
                role: expected
            }
        );
    }
}
#[test]
fn copied_base_extension_does_not_admit_other_stage_roots() {
    let old = plan();
    let new = SingleBoundaryNamePlan::v2(old.identities(), old.selection()).unwrap();
    for source in [257, 515, 771, 1032, 1288, 1800, 2320] {
        for kind in [2, 3] {
            assert!(
                expansion::member(&new, key(source, FrozenSweepRole::Generated, kind)).is_err()
            );
        }
    }
    assert!(expansion::member(&new, key(2058, FrozenSweepRole::Generated, 2)).is_err());
    assert!(expansion::member(&new, key(2056, FrozenSweepRole::First, 2)).is_err());
}
