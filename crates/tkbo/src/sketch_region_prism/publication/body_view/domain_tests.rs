// SPDX-License-Identifier: GPL-2.0-only
use super::{coverage, error::*, lookup, test_fixture::*};
#[test]
fn symbolic_complete_domains_match_but_grant_no_publication_binding() {
    let names = names();
    let (members, uses) = all_domain(&names);
    coverage::domains(&names, &members, &uses).unwrap();
    assert_eq!(members.len(), 13);
    assert_eq!(uses.len(), 26);
}
#[test]
fn duplicate_member_cannot_hide_omitted_member_behind_equal_count() {
    let names = names();
    let (mut members, uses) = all_domain(&names);
    members[0].key = members[1].key;
    let failed = coverage::domains(&names, &members, &uses).unwrap_err();
    assert!(matches!(
        failed.cause,
        PrismBodyProjectionCause::Coverage {
            actual: 12,
            expected: 13,
            ..
        }
    ));
    assert_eq!(failed.class, PrismBodyProjectionClass::Incomplete);
}
#[test]
fn duplicate_use_cannot_hide_a_distinct_ancestry_or_endpoint() {
    let names = names();
    let (members, mut uses) = all_domain(&names);
    uses[0].key = uses[1].key.clone();
    assert!(coverage::domains(&names, &members, &uses).is_err());
    let (_, fresh) = all_domain(&names);
    assert!(coverage::domains(&names, &members, &fresh).is_ok());
}
#[test]
fn missing_and_extra_domain_rows_refuse_without_partial_body() {
    let names = names();
    let (mut members, uses) = all_domain(&names);
    members.pop();
    assert!(coverage::domains(&names, &members, &uses).is_err());
    members.push(node(key(
        999,
        crate::sketch_region_prism::frozen::FrozenSweepRole::First,
        6,
    )));
    assert!(coverage::domains(&names, &members, &uses).is_err());
}
#[test]
fn lookup_retains_full_wire_identity_and_typed_unknown_token() {
    let names = names();
    for row in names
        .member_bindings()
        .iter()
        .filter(|row| row.frozen.kind == 5)
    {
        assert_eq!(lookup::output_slot(&names, &row.frozen).unwrap(), None);
        assert_eq!(lookup::member(&names, &row.frozen).unwrap(), row.semantic);
    }
    let unknown = key(
        999,
        crate::sketch_region_prism::frozen::FrozenSweepRole::First,
        6,
    );
    assert!(matches!(
        lookup::member(&names, &unknown).unwrap_err().cause,
        PrismBodyProjectionCause::SemanticMap(_)
    ));
}

#[test]
fn root_is_unique_semantic_solid_occurrence_not_first_returned_row() {
    let names = names();
    let (_, mut uses) = all_domain(&names);
    let root = lookup::root_occurrence(&names, &uses).unwrap();
    uses.reverse();
    assert_eq!(lookup::root_occurrence(&names, &uses).unwrap(), root);
    assert_eq!(root.0.len(), 1);
    assert_eq!(root.0[0].member.kind, 2);
    uses.retain(|row| row.key.0.len() != 1);
    assert!(lookup::root_occurrence(&names, &uses).is_err());
}
