//! Unit tests of the selections over the fake chain (d shares c, c shares a and b).
use super::*;
use crate::interface::test_norm::{FakeEntity, fake_chain};
use crate::interface::{InterfaceGraph, InterfaceInterfaceModel};
use rustycad_tkernel::{
    standard::{Handle, RuntimeTyped, StandardTransient},
    tcollection::HAsciiString,
};

fn chain() -> (
    Handle<dyn InterfaceInterfaceModel>,
    InterfaceGraph,
    Vec<Handle<StandardTransient>>,
) {
    let (model, ents) = fake_chain();
    let graph = InterfaceGraph::new(&model, true).unwrap();
    (model, graph, ents)
}

fn pointed(ents: &[&Handle<StandardTransient>]) -> Handle<dyn IFSelectSelection> {
    let sel = IFSelectSelectPointed::new();
    ents.iter().for_each(|e| {
        sel.add(e);
    });
    Handle::new(sel).cast()
}

fn count(sel: &dyn IFSelectSelection, g: &InterfaceGraph) -> i32 {
    sel.root_result(g).unwrap().nb_entities()
}

#[test]
fn model_selections() {
    let (_, g, ents) = chain();
    let roots = IFSelectSelectModelRoots::new();
    assert_eq!(
        roots.root_result(&g).unwrap().content(),
        vec![ents[3].clone()]
    );
    let all = IFSelectSelectModelEntities::new();
    assert_eq!(
        (
            count(&all, &g),
            all.complete_result(&g).unwrap().nb_entities()
        ),
        (4, 4)
    );
    let number = IFSelectSelectEntityNumber::new();
    assert_eq!(
        (count(&number, &g), number.label().as_bytes()),
        (0, &b"Entity Number 0"[..])
    );
    let param = Handle::new(IFSelectIntParam::new());
    param.set_value(2);
    number.set_number(&param);
    assert_eq!(
        number.root_result(&g).unwrap().content(),
        vec![ents[1].clone()]
    );
    param.set_value(9);
    assert_eq!(count(&number, &g), 0);
    assert!(number.number() == param && all.label().as_bytes() == b"All Entities from Model");
}

#[test]
fn pointed_lists() {
    let (_, g, ents) = chain();
    let sel = IFSelectSelectPointed::new();
    assert!(!sel.is_set() && !sel.add(&Handle::default()));
    sel.set_entity(&ents[2]);
    assert!(sel.is_set() && sel.nb_items() == 1 && sel.item(1) == ents[2] && sel.item(2).is_null());
    assert_eq!(sel.complete_result(&g).unwrap().nb_entities(), 3);
    assert!(sel.remove(&ents[0]), "native quirk: true when absent");
    assert!(sel.toggle(&ents[0]) && !sel.toggle(&ents[0]));
    assert!(
        !sel.add_list(Some(&[ents[0].clone(), ents[0].clone()])),
        "native quirk: always false"
    );
    assert_eq!(
        sel.nb_items(),
        3,
        "native quirk: the duplicates of the list are appended"
    );
    assert!(sel.remove_list(Some(&[ents[0].clone()])) && sel.toggle_list(None));
    sel.set_list(Some(&[ents[3].clone()]));
    assert_eq!(
        (sel.rank(&ents[3]), sel.label().as_bytes()),
        (1, &b"Pointed Entities"[..])
    );
    sel.clear();
    assert!(!sel.is_set() && sel.unique_result(&g).unwrap().nb_entities() == 0);
}

#[test]
fn deductions_and_extractions() {
    let (_, g, ents) = chain();
    let shared = IFSelectSelectShared::new();
    assert_eq!(count(&shared, &g), 0, "no input");
    shared.set_input(&pointed(&[&ents[2]]));
    assert!(shared.has_input() && !shared.has_alternate());
    assert_eq!(count(&shared, &g), 2);
    shared.alternate().set_entity(&ents[3]);
    assert!(shared.has_alternate());
    assert_eq!(
        shared.root_result(&g).unwrap().content(),
        vec![ents[2].clone()],
        "the alternate list, then cleared"
    );
    assert!(!shared.has_alternate());
    let sharing = IFSelectSelectSharing::new();
    sharing.set_input(&pointed(&[&ents[2]]));
    assert_eq!(
        sharing.root_result(&g).unwrap().content(),
        vec![ents[3].clone()]
    );
    let mut iter = IFSelectSelectionIterator::of_selection(&Handle::new(sharing).cast()).unwrap();
    assert!(iter.more());
    iter.next();
    assert!(!iter.more() && iter.value().is_err());
    let types = IFSelectSelectType::with_type(FakeEntity::static_type());
    types.set_input(&Handle::new(IFSelectSelectModelEntities::new()).cast());
    assert_eq!(count(&types, &g), 4);
    types.set_direct(false);
    assert_eq!(
        (count(&types, &g), types.label().as_bytes()),
        (0, &b"Removed: Fake_Entity"[..])
    );
    types.set_type(HAsciiString::static_type());
    assert_eq!(count(&types, &g), 4);
    let errors = IFSelectSelectErrorEntities::new();
    errors.set_input(&Handle::new(IFSelectSelectModelEntities::new()).cast());
    assert_eq!(
        (count(&errors, &g), errors.label().as_bytes()),
        (0, &b"Picked: Error Entities"[..])
    );
    let unknown = IFSelectSelectUnknownEntities::new();
    unknown.set_input(&Handle::new(IFSelectSelectModelEntities::new()).cast());
    assert_eq!(count(&unknown, &g), 0);
}

#[test]
fn combinations() {
    let (_, g, ents) = chain();
    let union = IFSelectSelectUnion::new();
    union.add(&pointed(&[&ents[0]]), 0);
    union.add(&pointed(&[&ents[1]]), 1);
    assert_eq!((union.nb_inputs(), count(&union, &g)), (2, 2));
    let first = union.input_at(1).unwrap();
    assert!(union.input_rank(&first) == 1 && union.input_at(3).is_err());
    assert!(union.remove(&first) && !union.remove_at(5) && union.nb_inputs() == 1);
    let inter = IFSelectSelectIntersection::new();
    inter.add(&pointed(&[&ents[0], &ents[1]]), 0);
    inter.add(&pointed(&[&ents[1], &ents[2]]), 0);
    assert_eq!(
        inter.root_result(&g).unwrap().content(),
        vec![ents[1].clone()]
    );
    inter.add(&pointed(&[&ents[1]]), 0);
    assert_eq!(
        count(&inter, &g),
        0,
        "native quirk: reset after the second of three inputs"
    );
    let diff = IFSelectSelectDiff::new();
    diff.set_main_input(&pointed(&[&ents[0], &ents[1]]));
    assert_eq!(count(&diff, &g), 2);
    diff.set_second_input(&pointed(&[&ents[1]]));
    assert!(diff.has_second_input());
    assert_eq!(
        diff.root_result(&g).unwrap().content(),
        vec![ents[0].clone()]
    );
    let mut iter = IFSelectSelectionIterator::new();
    diff.fill_iterator(&mut iter);
    union.fill_iterator(&mut iter);
    let mut all = IFSelectSelectionIterator::new();
    all.add_from_iter(&mut iter);
    let mut n = 0;
    while all.more() {
        n += 1;
        all.next();
    }
    assert_eq!(n, 3);
    assert!(
        diff.label().as_bytes() == b"Differences"
            && inter.label().as_bytes() == b"Intersection (AND)"
    );
}
