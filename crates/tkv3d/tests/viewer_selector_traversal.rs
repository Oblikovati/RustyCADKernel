//! Actual BVH traversal, callback failures and visibility filtering.
#[path = "support/traversal_scene.rs"]
mod traversal_scene;
use rustycad_tkernel::standard::Handle;
use rustycad_tkv3d::{select_mgr::*, select3d::Select3DSensitiveEntity};
use std::sync::atomic::Ordering;
use traversal_scene::{TraversalScene, volume::TraversalConfig};
#[test]
fn empty_traversal_clears_results_and_dispatches_the_derived_sorter() {
    let scene = TraversalScene::new(TraversalConfig {
        count: 0,
        ..TraversalConfig::default()
    });
    let base = scene.selector.viewer_selector_base();
    base.fields
        .write()
        .results
        .stored
        .add(Handle::default(), SelectMgrSortCriterion::default());
    scene.selector.traverse_sensitives(-1).unwrap();
    assert_eq!(scene.selector.nb_picked(), 0);
    assert!(base.fields.read().results.is_sorted);
    assert_eq!(scene.selector.sorts.load(Ordering::Acquire), 1);
}
#[test]
fn point_selection_uses_real_entity_matches_and_world_space_picked_points() {
    let scene = TraversalScene::new(TraversalConfig::default());
    scene.selector.traverse_sensitives(-1).unwrap();
    assert_eq!(scene.selector.nb_picked(), 1);
    assert_eq!(scene.selector.picked(1).unwrap(), scene.owners[1]);
    assert_eq!(
        *scene.selector.picked_entity(1).unwrap(),
        scene.entities[4].cast::<dyn Select3DSensitiveEntity>()
    );
    assert_eq!(scene.selector.sorts.load(Ordering::Acquire), 1);
    assert!(scene.events.read().contains(&4));
}
#[test]
fn matching_failure_preserves_unsorted_prefix_and_releases_worker_sentry() {
    let scene = TraversalScene::new(TraversalConfig {
        kind: 2,
        failing: 4,
        flags: 8,
        ..TraversalConfig::default()
    });
    assert!(scene.selector.traverse_sensitives(-1).is_err());
    assert!(
        !scene
            .selector
            .viewer_selector_base()
            .fields
            .read()
            .results
            .is_sorted
    );
    assert_eq!(scene.selector.sorts.load(Ordering::Acquire), 0);
    scene.entities[4].fail.store(0, Ordering::Release);
    scene.selector.traverse_sensitives(-1).unwrap();
    assert_eq!(scene.selector.sorts.load(Ordering::Acquire), 1);
}
#[test]
fn containment_discards_owners_with_unmatched_or_inactive_entities() {
    for flags in [0, 2] {
        let scene = TraversalScene::new(TraversalConfig {
            kind: 2,
            overlap: false,
            flags,
            ..TraversalConfig::default()
        });
        scene.selector.traverse_sensitives(-1).unwrap();
        assert_eq!(scene.selector.nb_picked(), 0);
    }
}
#[test]
fn view_affinity_is_ignored_only_for_the_all_views_sentinel() {
    let scene = TraversalScene::new(TraversalConfig {
        kind: 2,
        flags: 4,
        ..TraversalConfig::default()
    });
    scene.selector.traverse_sensitives(-1).unwrap();
    assert_eq!(scene.selector.nb_picked(), 2);
    scene.selector.traverse_sensitives(2).unwrap();
    assert_eq!(scene.selector.nb_picked(), 1);
    assert_eq!(scene.selector.picked(1).unwrap(), scene.owners[1]);
}
