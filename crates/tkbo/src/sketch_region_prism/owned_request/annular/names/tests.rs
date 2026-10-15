// SPDX-License-Identifier: GPL-2.0-only
//! Production names publish the owned annulus end to end; their equality pattern is exactly the
//! independently authored fixture's, and the dictionary resolves every body key to caller IDs.
use super::super::super::names::CallerVertexId;
use super::super::tests::{owned_request, prepared};
use super::*;
use crate::sketch_region_prism::frozen::annular_fixture::authored;
use crate::sketch_region_prism::frozen::annular_history::fixture::{
    authored_names, fresh_embedding,
};
use crate::sketch_region_prism::frozen::annular_orientation::fixture::current_binding;
use crate::sketch_region_prism::frozen::{FrozenPrismKey, FrozenSweepRole};
use crate::sketch_region_prism::publication::annular::{
    AnnularPublicationRequest, PublishedAnnularPrism, publish_annular_prism, tests::options,
};
use crate::sketch_region_prism::publication::{
    PrismPublicationCheckpoint, PrismPublicationProgress,
};

struct Continue;
impl PrismPublicationProgress for Continue {
    fn checkpoint(&mut self, _: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        Ok(false)
    }
}
fn issued(prepared: &PreparedAnnularPrism) -> IssuedAnnularNames {
    prepared.with_region(|region, receipt| {
        let binding = current_binding(region, receipt);
        let names = issue_annular_names(prepared.request(), region, receipt, &binding);
        names.unwrap_or_else(|_| panic!("production annular names"))
    })
}
fn with_published<T>(
    prepared: &PreparedAnnularPrism,
    names: &IssuedAnnularNames,
    run: impl FnOnce(&PublishedAnnularPrism) -> T,
) -> T {
    prepared.with_region(|region, receipt| {
        let request = AnnularPublicationRequest {
            region,
            receipt,
            bindings: &names.publication,
            copied: &names.copied,
            historical: &names.historical,
            requested: options(),
        };
        let embedding = fresh_embedding(region, receipt);
        let result = publish_annular_prism(&request, embedding, &mut Continue);
        run(&result.unwrap_or_else(|f| panic!("refused at {:?} class {:?}", f.phase, f.class)))
    })
}
#[test]
fn production_names_publish_the_owned_annulus_with_complete_history() {
    let prepared = prepared(false);
    let names = issued(&prepared);
    let replay = with_published(&prepared, &names, |published| {
        assert_eq!(published.history().continued().len(), 6);
        assert_eq!(published.history().retired_wires().len(), 2);
        published.replay_bytes().to_vec()
    });
    let again = prepared_names_replay(false);
    assert_eq!(replay, again, "independent preparations replay identically");
}
fn prepared_names_replay(reversed: bool) -> Vec<u8> {
    let prepared = prepared(reversed);
    let names = issued(&prepared);
    with_published(&prepared, &names, |p| p.replay_bytes().to_vec())
}
/// Exact relabeling: equal production names iff equal independently authored fixture names.
fn same_pattern(actual: &[u64], expected: &[u64]) {
    refines(actual, expected, &[]);
}
/// Production equality implies fixture equality; fixture-only equalities must be listed.
fn refines(actual: &[u64], expected: &[u64], fixture_only: &[[u64; 2]]) {
    assert_eq!(actual.len(), expected.len());
    for i in 0..actual.len() {
        for j in 0..actual.len() {
            if actual[i] == actual[j] {
                assert_eq!(expected[i], expected[j], "rows {i},{j}");
            } else if expected[i] == expected[j] {
                let pair = [expected[i], expected[j]];
                assert!(fixture_only.contains(&pair), "rows {i},{j}: {pair:?}");
            }
        }
    }
}
fn publication_names(bindings: &PrismPublicationBindings) -> Vec<u64> {
    let mut names = vec![bindings.operation.0, bindings.selected_region.0];
    names.extend(
        bindings
            .originals
            .iter()
            .flat_map(|o| [o.member.0, o.directed_use.0]),
    );
    names.extend(bindings.sources.iter().map(|s| s.member.0));
    names.extend(bindings.directed.iter().map(|d| d.authored_use.0));
    names
}
fn history_names(copied: &PrismCopyBindings, historical: &AnnularHistoryBindings) -> Vec<u64> {
    let mut names: Vec<u64> = copied.selected_members.iter().map(|m| m.member.0).collect();
    names.extend(copied.selected_uses.iter().map(|u| u.authored_use.0));
    names.extend(copied.copied_uses.iter().map(|u| u.authored_use.0));
    for scope in &historical.scopes {
        names.extend(scope.names.members.iter().map(|m| m.member.0));
        names.extend(scope.names.uses.iter().map(|u| u.authored_use.0));
    }
    names
}
/// The fixture's InputCopy bases 11000 + 100 * {10, 20} coincide with its Partition (12000) and
/// CandidateCopy (13000) bases; production names keep those scopes distinct, the only difference.
#[test]
fn issued_names_have_the_independent_fixture_equality_pattern() {
    let prepared = prepared(false);
    let names = issued(&prepared);
    let (publication, (copied, historical)) = prepared.with_region(|region, receipt| {
        let binding = current_binding(region, receipt);
        let publication = authored(region, receipt, &binding);
        (publication, authored_names(region, receipt))
    });
    same_pattern(
        &publication_names(&names.publication),
        &publication_names(&publication),
    );
    let fixture_only = [
        [12014, 12014],
        [12015, 12015],
        [13024, 13024],
        [13025, 13025],
    ];
    refines(
        &history_names(&names.copied, &names.historical),
        &history_names(&copied, &historical),
        &fixture_only,
    );
}
fn meaning_of(names: &IssuedAnnularNames, key: &FrozenPrismKey) -> AnnularNameMeaning {
    let meaning = names.dictionary.meaning(key.source);
    meaning.unwrap_or_else(|| panic!("unissued body token {:?}", key.source))
}
#[test]
fn every_face_key_resolves_to_caller_annulus_or_boundary_identity() {
    let prepared = prepared(false);
    let names = issued(&prepared);
    let faces = with_published(&prepared, &names, |published| {
        let faces = published.join().faces().iter();
        faces
            .map(|f| f.chart.occurrence.0.last().expect("face key").0)
            .collect::<Vec<_>>()
    });
    let mut caps = Vec::new();
    let mut sides = Vec::new();
    for key in &faces {
        assert_eq!(
            names.dictionary.meaning(key.operation),
            Some(AnnularNameMeaning::Operation(CallerOperationId(77)))
        );
        match (key.role, meaning_of(&names, key)) {
            (FrozenSweepRole::Generated, meaning) => {
                sides.push(meaning.boundary().expect("side boundary").0)
            }
            (role, AnnularNameMeaning::Annulus { region: 301 }) => caps.push(role),
            other => panic!("unexpected face key meaning {other:?}"),
        }
    }
    sides.sort();
    assert_eq!(sides, [101, 202]);
    assert_eq!(caps.len(), 2);
    assert_ne!(caps[0], caps[1]);
}
/// Operation and selected-region tokens are the caller values the two-loop proof already binds;
/// every role token is caller-independent and its caller meaning lives only in the dictionary.
#[test]
fn caller_identities_live_in_the_dictionary_not_the_role_tokens() {
    let first = issued(&prepared(false));
    let mut request = owned_request(false);
    request.operation = CallerOperationId(9001);
    request.regions.annulus = 9301;
    request.outer.closure = CallerVertexId(4242);
    let outcome = prepare_owned_annular_prism(request, &mut super::super::tests::Continue);
    let other = issued(&outcome.unwrap_or_else(|f| panic!("refused at {:?}", f.phase)));
    let (a, b) = (
        publication_names(&first.publication),
        publication_names(&other.publication),
    );
    assert_eq!((a[0], a[1]), (77, 301));
    assert_eq!((b[0], b[1]), (9001, 9301));
    let relabel = |name: u64| match name {
        77 => 9001,
        301 => 9301,
        role => role,
    };
    assert_eq!(a.into_iter().map(relabel).collect::<Vec<_>>(), b);
    let vertex = first.publication.originals.iter().map(|o| o.member);
    let closures: Vec<_> = vertex
        .filter_map(|token| match other.dictionary.meaning(token) {
            Some(AnnularNameMeaning::Boundary {
                caller,
                kind: 5,
                role: AnnularBoundaryRole::Outer,
                ..
            }) => Some(caller),
            _ => None,
        })
        .collect();
    assert!(!closures.is_empty() && closures.iter().all(|c| *c == 4242));
    assert_ne!(first.dictionary, other.dictionary);
}
#[test]
fn caller_value_colliding_with_a_role_token_is_refused() {
    let first = issued(&prepared(false));
    let role_token = first.publication.originals[0].member;
    let mut request = owned_request(false);
    request.regions.annulus = role_token.0;
    let outcome = prepare_owned_annular_prism(request, &mut super::super::tests::Continue);
    let prepared = outcome.unwrap_or_else(|f| panic!("refused at {:?}", f.phase));
    let collided = prepared.with_region(|region, receipt| {
        let binding = current_binding(region, receipt);
        let names = issue_annular_names(prepared.request(), region, receipt, &binding);
        matches!(names, Err(AnnularNameCause::Collision { .. }))
    });
    assert!(
        collided,
        "a caller value equal to an issued role token must not alias it"
    );
}
