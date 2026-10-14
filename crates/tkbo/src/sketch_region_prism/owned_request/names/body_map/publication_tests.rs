// SPDX-License-Identifier: GPL-2.0-only
//! Deliberately false table rows cannot translate a legacy accepted fixture into semantic names.
use super::fixture::{PublicationFixture, PublicationObserver};
use crate::sketch_region_prism::frozen::FrozenPrismKey;
use crate::sketch_region_prism::owned_request::names::{body_map::*, *};
#[test]
fn legacy_publication_is_not_implicitly_reinterpreted_as_dictionary_named() {
    let fixture = PublicationFixture::new();
    let result = fixture.publish(&mut PublicationObserver::default());
    let projection = result.join().projection();
    let members = projection
        .nodes()
        .iter()
        .map(|row| false_member(row.key))
        .collect();
    let uses = false_uses(&result);
    let error = fixture_bound_names(plan(), &result, members, uses).unwrap_err();
    assert!(
        matches!(error.cause,PrismSemanticMapCause::Operation {actual,expected:257} if actual==fixture.operation.authored.operation.0)
    );
    assert!(!fixture_unbound_symbolic_names().binds(&result));
}
fn plan() -> SingleBoundaryNamePlan {
    SingleBoundaryNamePlan::v1(
        SingleBoundaryIdentities {
            operation: CallerOperationId(401),
            boundary: CallerBoundaryId(403),
            closure: CallerVertexId(405),
            start: CallerDirectedUseId(407),
            end: CallerDirectedUseId(409),
        },
        SemanticSelectionIntent::BoundedRegionContaining { seed: [1., 1.] },
    )
    .unwrap()
}
fn false_member(frozen: FrozenPrismKey) -> SemanticMemberBinding {
    SemanticMemberBinding {
        frozen,
        semantic: SemanticMemberKey {
            operation: CallerOperationId(401),
            source: CallerBoundaryId(403),
            source_member: SemanticNameSlot {
                stage: SemanticNameStage::SweepBase,
                role: SemanticNameRole::Boundary,
            },
            sweep_role: frozen.role,
            kind: frozen.kind,
        },
        output_slot: None,
    }
}

fn false_uses(result: &super::PublishedNamedPrism) -> Vec<SemanticUseBinding> {
    result
        .join()
        .projection()
        .uses()
        .iter()
        .map(|row| SemanticUseBinding {
            frozen: row.key.clone(),
            semantic: SemanticUseKey(vec![]),
        })
        .collect()
}
