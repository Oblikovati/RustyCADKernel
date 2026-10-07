//! Unit tests of the dispatches over the fake chain (d shares c, c shares a and b).
use super::*;
use crate::if_graph::{IFGraphSubPartsEvaluate, IFGraphSubPartsIterator};
use crate::interface::InterfaceGraph;
use crate::interface::test_norm::fake_chain;
use rustycad_tkernel::{standard::Handle, tcollection::HAsciiString};

fn chain() -> InterfaceGraph {
    let (model, _) = fake_chain();
    InterfaceGraph::new(&model, true).unwrap()
}

fn all() -> Handle<dyn IFSelectSelection> {
    Handle::new(IFSelectSelectModelEntities::new()).cast()
}

/// The sizes of the parts of `packs`.
fn part_sizes(g: &InterfaceGraph, disp: &dyn IFSelectDispatch) -> Vec<i32> {
    let mut packs = IFGraphSubPartsIterator::new(g, false).unwrap();
    disp.packets(g, &mut packs).unwrap();
    let mut sizes = Vec::new();
    packs.start().unwrap();
    while packs.more().unwrap() {
        sizes.push(packs.entities().unwrap().nb_entities());
        packs.next();
    }
    sizes
}

fn param(value: i32) -> Handle<IFSelectIntParam> {
    let p = Handle::new(IFSelectIntParam::new());
    p.set_value(value);
    p
}

#[test]
fn dispatch_base_members() {
    let g = chain();
    let disp = IFSelectDispGlobal::new();
    assert!(!disp.has_root_name() && disp.get_entities(&g).is_err());
    disp.set_root_name(&Handle::new(HAsciiString::new("root")));
    assert!(disp.has_root_name() && disp.root_name().is_some());
    let shared = IFSelectSelectShared::new();
    shared.set_input(&all());
    disp.set_final_selection(&Handle::new(shared).cast());
    let iter = disp.selections().unwrap();
    assert!(
        !iter.more(),
        "native quirk: the iterator is returned at its end"
    );
    assert!(!disp.can_have_remainder() && disp.remainder(&g).unwrap().nb_entities() == 0);
    assert_eq!(
        disp.packeted(&g).unwrap().nb_entities(),
        disp.get_entities(&g).unwrap().nb_entities()
    );
}

#[test]
fn global_and_per_one() {
    let g = chain();
    let global = IFSelectDispGlobal::new();
    global.set_final_selection(&all());
    assert_eq!(part_sizes(&g, &global), vec![4]);
    let per_one = IFSelectDispPerOne::new();
    per_one.set_final_selection(&all());
    assert_eq!(
        (per_one.label().as_bytes(), per_one.limited_max(7)),
        (&b"One File per Input Entity"[..], Some(7))
    );
    assert_eq!(part_sizes(&g, &per_one).len(), 4, "one part per root");
}

#[test]
fn per_count_and_per_files() {
    let g = chain();
    let per_count = IFSelectDispPerCount::new();
    per_count.set_final_selection(&all());
    assert_eq!(
        (per_count.count_value(), per_count.label().as_bytes()),
        (1, &b"One File per 1 Entities"[..])
    );
    per_count.set_count(&param(3));
    assert!(per_count.count().is_some() && per_count.limited_max(7) == Some(3));
    assert_eq!(part_sizes(&g, &per_count), vec![3, 1]);
    let per_files = IFSelectDispPerFiles::new();
    per_files.set_final_selection(&all());
    per_files.set_count(&param(3));
    assert_eq!(
        (per_files.label().as_bytes(), per_files.limited_max(7)),
        (&b"Maximum 3 Files"[..], Some(3))
    );
    assert!(per_files.count().is_some());
    assert_eq!(part_sizes(&g, &per_files), vec![2, 2]);
}

#[test]
fn per_signature() {
    let g = chain();
    let disp = IFSelectDispPerSignature::new();
    disp.set_final_selection(&all());
    assert_eq!(disp.label().as_bytes(), b"One File per Signature ???");
    assert_eq!(part_sizes(&g, &disp), vec![4]);
    let counter = IFSelectSignCounter::with_signature(
        &Handle::new(IFSelectSignType::new(true)).cast(),
        true,
        false,
    )
    .unwrap();
    let counter: Handle<dyn IFSelectSignCounterOps> = Handle::new(counter).cast();
    disp.set_sign_counter(&counter).unwrap();
    assert!(
        disp.sign_counter() == counter
            && disp.sign_name() == "Class Type"
            && disp.limited_max(3) == Some(3)
    );
    assert_eq!(part_sizes(&g, &disp), vec![4]);
    assert!(disp.set_sign_counter(&Handle::default()).is_err());
}
