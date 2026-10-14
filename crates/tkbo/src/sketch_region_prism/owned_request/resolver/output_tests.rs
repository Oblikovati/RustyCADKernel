// SPDX-License-Identifier: GPL-2.0-only
use super::{source_tests::*, *};
use crate::sketch_region_prism::frozen::PrismDirectedBinding;
fn named_outputs(poles: bool, traversal: bool) -> Vec<PrismDirectedBinding> {
    let mut request = super::super::test_fixture::request();
    request.sources[0].reversed_traversal = traversal;
    if poles {
        request.sources[0].world_poles.reverse();
    }
    let prepared = super::super::test_fixture::prepared(request);
    let plan = plan_for(&prepared);
    let roles = checked(roster::resolve_stages(&prepared, &plan, &mut || false));
    checked(output::directed_outputs(&prepared, &plan, &roles.binding))
}
#[test]
fn output_seam_endpoints_keep_source_anchor_at_both_sweep_ends() {
    for (poles, traversal) in [(false, false), (false, true), (true, false), (true, true)] {
        let rows = named_outputs(poles, traversal);
        assert_eq!(rows.len(), 26);
        assert_seam_anchors(&rows, 2336, 2309);
        assert_seam_anchors(&rows, 2337, 2310);
    }
}
fn assert_seam_anchors(rows: &[PrismDirectedBinding], seam: u64, endpoint: u64) {
    let edges: Vec<_> = rows.iter().filter(|r| r.authored_use.0 == seam).collect();
    assert_eq!(edges.len(), 1);
    let path = &edges[0].actual.child_slots;
    let vertices: Vec<_> = rows
        .iter()
        .filter(|r| {
            r.actual.child_slots.len() == path.len() + 1 && r.actual.child_slots.starts_with(path)
        })
        .collect();
    assert_eq!(vertices.len(), 2);
    assert!(vertices.iter().all(|row| row.authored_use.0 == endpoint));
    assert!(
        !vertices[0]
            .actual
            .occurrence
            .is_same(&vertices[1].actual.occurrence)
    );
}
#[test]
fn materialized_history_uses_all_actual_stages_without_caller_intermediate_ids() {
    let prepared = operation(false);
    let plan = plan_for(&prepared);
    let roles = checked(roster::resolve_stages(&prepared, &plan, &mut || false));
    let history = checked(materialize::intermediates(&plan, &roles));
    assert_eq!(history.complementary_region.0, 1801);
    let counts: Vec<_> = history
        .intermediate
        .iter()
        .map(|r| (r.members.len(), r.uses.len()))
        .collect();
    assert_eq!(counts, [(2, 3), (3, 4), (3, 4), (4, 5)]);
    assert_intermediate_tokens(&history);
}

fn assert_intermediate_tokens(history: &PrismHistoryBindings) {
    let mut tokens: Vec<_> = history
        .intermediate
        .iter()
        .flat_map(|r| r.uses.iter().map(|u| u.authored_use.0))
        .collect();
    tokens.sort();
    assert_eq!(
        tokens,
        [
            773, 774, 775, 1029, 1030, 1031, 1059, 1285, 1286, 1287, 1315, 1797, 1798, 1799, 1805,
            1807
        ]
    );
}
