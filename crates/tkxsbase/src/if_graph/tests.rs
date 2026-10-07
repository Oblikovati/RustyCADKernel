//! Unit tests of the IFGraph tools over the fake chain (d shares c, c shares a and b).
use super::*;
use crate::interface::test_norm::fake_chain;
use crate::interface::{InterfaceEntityIterator, InterfaceGraph, InterfaceGraphContentEvaluate};
use rustycad_tkernel::standard::{Handle, StandardTransient};

fn chain_graph() -> (InterfaceGraph, Vec<Handle<StandardTransient>>) {
    let (model, ents) = fake_chain();
    (InterfaceGraph::new(&model, true).unwrap(), ents)
}

fn list(ents: &[&Handle<StandardTransient>]) -> InterfaceEntityIterator {
    let mut iter = InterfaceEntityIterator::new();
    ents.iter().for_each(|e| iter.add_item(e));
    iter
}

#[test]
fn all_shared_takes_the_shared_entities() {
    let (graph, ents) = chain_graph();
    let mut all = IFGraphAllShared::with_entity(&graph, &ents[2]).unwrap();
    assert_eq!(all.result().unwrap().nb_entities(), 3);
    all.get_from_iter(&list(&[&ents[3]])).unwrap();
    assert_eq!(all.result().unwrap().nb_entities(), 4);
    all.reset_data().unwrap();
    assert_eq!(all.result().unwrap().nb_entities(), 0);
    let foreign = crate::interface::test_norm::fake_entity("x", &[]);
    let mut none = IFGraphAllShared::with_entity(&graph, &foreign).unwrap();
    assert_eq!(none.result().unwrap().nb_entities(), 0);
}

#[test]
fn compare_two_lists() {
    let (graph, ents) = chain_graph();
    let mut cmp = IFGraphCompare::new(&graph).unwrap();
    cmp.get_from_iter(&list(&[&ents[0], &ents[1]]), true)
        .unwrap();
    cmp.get_from_iter(&list(&[&ents[1], &ents[2]]), false)
        .unwrap();
    assert_eq!(cmp.common().unwrap().nb_entities(), 1);
    assert_eq!(cmp.first_only().unwrap().nb_entities(), 1);
    assert_eq!(cmp.second_only().unwrap().nb_entities(), 1);
    assert_eq!(cmp.result().unwrap().nb_entities(), 3);
    cmp.get_from_entity(&ents[3], true).unwrap();
    assert_eq!(
        cmp.result().unwrap().nb_entities(),
        3,
        "native defect: GetFromEntity adds nothing"
    );
    cmp.keep_common().unwrap();
    assert_eq!(cmp.first_only().unwrap().nb_entities(), 1);
    cmp.reset_data().unwrap();
    cmp.get_from_iter(&list(&[&ents[0]]), true).unwrap();
    assert_eq!(
        cmp.result().unwrap().nb_entities(),
        0,
        "native quirk: Reset drops the statuses of the graph for good"
    );
    let mut fresh = IFGraphCompare::new(&graph).unwrap();
    fresh.get_from_iter(&list(&[&ents[0]]), true).unwrap();
    fresh.get_from_iter(&list(&[&ents[1]]), false).unwrap();
    fresh.remove_second().unwrap();
    assert_eq!(fresh.result().unwrap().nb_entities(), 1);
    fresh.get_from_iter(&list(&[&ents[2]]), false).unwrap();
    fresh.merge().unwrap();
    assert_eq!(fresh.first_only().unwrap().nb_entities(), 2);
}

#[test]
fn cumulate_counts_selections() {
    let (graph, ents) = chain_graph();
    let mut cum = IFGraphCumulate::new(&graph).unwrap();
    cum.get_from_iter(&list(&[&ents[0], &ents[1]])).unwrap();
    cum.get_from_iter(&list(&[&ents[1]])).unwrap();
    assert_eq!(
        (
            cum.nb_times(&ents[0]).unwrap(),
            cum.nb_times(&ents[1]).unwrap()
        ),
        (1, 2)
    );
    assert_eq!(cum.highest_nb_times().unwrap(), 2);
    assert_eq!(cum.overlapped().unwrap().nb_entities(), 1);
    assert_eq!(cum.forgotten().unwrap().nb_entities(), 2);
    assert_eq!(cum.per_count(1).unwrap().nb_entities(), 1);
    assert_eq!(cum.result().unwrap().nb_entities(), 2);
    cum.get_from_entity(&ents[3]).unwrap();
    assert_eq!(
        cum.nb_times(&ents[3]).unwrap(),
        -1,
        "native defect: GetFromEntity adds nothing"
    );
    assert_eq!(
        cum.nb_times(&crate::interface::test_norm::fake_entity("x", &[]))
            .unwrap(),
        0
    );
    cum.reset_data().unwrap();
}

#[test]
fn external_sources_of_a_list() {
    let (graph, ents) = chain_graph();
    let mut ext = IFGraphExternalSources::new(&graph).unwrap();
    ext.get_from_iter(&list(&[&ents[0]])).unwrap();
    assert_eq!(ext.result().unwrap().content(), vec![ents[2].clone()]);
    assert!(!ext.is_empty().unwrap());
    ext.reset_data().unwrap();
    assert!(ext.is_empty().unwrap());
    let mut fresh = IFGraphExternalSources::new(&graph).unwrap();
    fresh.get_from_entity(&ents[2]).unwrap();
    assert_eq!(fresh.result().unwrap().content(), vec![ents[3].clone()]);
}

#[test]
fn sub_parts_and_components() {
    let (graph, ents) = chain_graph();
    let mut parts = IFGraphSubPartsIterator::new(&graph, false).unwrap();
    parts.add_part();
    parts.get_from_entity(&ents[2], true).unwrap();
    parts.add_part();
    parts.get_from_iter(&list(&[&ents[3]])).unwrap();
    parts.add_part();
    assert_eq!((parts.nb_parts(), parts.part_num()), (3, 3));
    assert!(parts.set_part_num(4).is_err());
    parts.set_part_num(1).unwrap();
    parts.set_load();
    assert_eq!(parts.entity_part_num(&ents[0]).unwrap(), 1);
    assert!(parts.is_in_part(&ents[3]).unwrap() && parts.is_loaded(&ents[1]).unwrap());
    parts.start().unwrap();
    let mut sizes = Vec::new();
    while parts.more().unwrap() {
        sizes.push((
            parts.entities().unwrap().nb_entities(),
            parts.is_single().unwrap(),
        ));
        parts.next();
    }
    assert_eq!(
        sizes,
        vec![(3, false), (1, true)],
        "the empty part 3 is skipped"
    );
    assert!(parts.first_entity().is_err() && parts.entities().is_err());
    assert_eq!(parts.loaded().unwrap().nb_entities(), 0);
    assert_eq!(parts.loaded_graph().unwrap().size(), 4);
    let mut copy = IFGraphSubPartsIterator::from_other(&mut parts).unwrap();
    assert_eq!(copy.nb_parts(), 2);
    assert!(
        copy.more().unwrap() && copy.first_entity().is_err(),
        "native defect: no first entities"
    );
    copy.reset().unwrap();
    copy.start().unwrap();
    copy.add_part();
    assert!(
        copy.start().is_err(),
        "native defect: Start clears the null first entities"
    );
    let mut other = IFGraphSubPartsIterator::new(&graph, true).unwrap();
    other.get_parts(&mut parts).unwrap();
    assert_eq!(other.nb_parts(), 2);
    let (model2, _) = fake_chain();
    let mut foreign =
        IFGraphSubPartsIterator::new(&InterfaceGraph::new(&model2, true).unwrap(), true).unwrap();
    assert!(foreign.get_parts(&mut parts).is_err());
}

#[test]
fn strong_components_and_roots() {
    let (graph, _) = chain_graph();
    let mut comps = IFGraphStrongComponants::new(&graph, true).unwrap();
    comps.start().unwrap();
    assert_eq!(comps.nb_parts(), 4, "one component per entity");
    let mut roots = IFGraphSCRoots::new(&graph, true).unwrap();
    roots.start().unwrap();
    assert_eq!(
        roots.nb_parts(),
        4,
        "the components come in model order (a, b, c, d): none is reached from a previous one"
    );
    // native quirk: Evaluate takes every present entity again, so each Start adds the
    // components again (the former parts become empty)
    comps.start().unwrap();
    assert_eq!(comps.nb_parts(), 8);
    // the copy restarts comps (12 parts) and iterates from the first part, empty but not
    // skipped by Start, then the four non-empty ones
    let mut copied = IFGraphSCRoots::from_components(&mut comps).unwrap();
    assert_eq!((comps.nb_parts(), copied.nb_parts()), (12, 5));
    assert!(copied.more().unwrap());
}
