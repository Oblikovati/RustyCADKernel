//! Unit tests of the flag, root, range, suite, exploration and list selections over the
//! fake chain (d shares c, c shares a and b).
use super::*;
use crate::interface::test_norm::{FakeEntity, fake_chain};
use crate::interface::{InterfaceEntityIterator, InterfaceGraph};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, StandardTransient},
    tcollection::AsciiString,
};

fn chain() -> (InterfaceGraph, Vec<Handle<StandardTransient>>) {
    let (model, ents) = fake_chain();
    (InterfaceGraph::new(&model, true).unwrap(), ents)
}

fn all() -> Handle<dyn IFSelectSelection> {
    Handle::new(IFSelectSelectModelEntities::new()).cast()
}

fn param(value: i32) -> Handle<IFSelectIntParam> {
    let p = Handle::new(IFSelectIntParam::new());
    p.set_value(value);
    p
}

#[test]
fn flags_and_roots() {
    let (g, ents) = chain();
    let flag = IFSelectSelectFlag::new("ShareError");
    flag.set_input(&all());
    assert_eq!(
        flag.root_result(&g).unwrap().nb_entities(),
        0,
        "no share error"
    );
    flag.set_direct(false);
    assert_eq!(flag.root_result(&g).unwrap().nb_entities(), 4);
    assert_eq!(
        IFSelectSelectFlag::new("NoSuchFlag")
            .root_result(&g)
            .unwrap()
            .nb_entities(),
        0
    );
    let incorrect = IFSelectSelectIncorrectEntities::new();
    incorrect.set_input(&all());
    assert_eq!(
        (
            incorrect.flag_name().as_bytes(),
            incorrect.root_result(&g).unwrap().nb_entities()
        ),
        (&b"Incorrect"[..], 0)
    );
    assert_eq!(
        incorrect.label().as_bytes(),
        b"Picked: Entities Flagged by Incorrect"
    );
    let roots = IFSelectSelectRoots::new();
    roots.set_input(&all());
    assert_eq!(
        roots.root_result(&g).unwrap().content(),
        ents,
        "native defect: every input entity is a root"
    );
    assert!(roots.has_unique_result() && roots.label().as_bytes() == b"Picked: Local Roots");
    let comps = IFSelectSelectRootComps::new();
    comps.set_input(&all());
    assert_eq!(comps.root_result(&g).unwrap().nb_entities(), 4);
    comps.set_direct(false);
    assert_eq!(comps.root_result(&g).unwrap().nb_entities(), 0);
}

#[test]
fn ranges() {
    let (g, ents) = chain();
    let range = IFSelectSelectRange::new();
    range.set_input(&all());
    assert_eq!(
        (
            range.root_result(&g).unwrap().nb_entities(),
            range.label().as_bytes()
        ),
        (4, &b"Picked: Rank no 0"[..])
    );
    range.set_range(&param(2), &param(3));
    assert_eq!(
        range.root_result(&g).unwrap().content(),
        vec![ents[1].clone(), ents[2].clone()]
    );
    assert_eq!(range.label().as_bytes(), b"Picked: From 2 Until 3");
    range.set_from(&param(3));
    assert_eq!(
        (
            range.root_result(&g).unwrap().nb_entities(),
            range.label().as_bytes()
        ),
        (2, &b"Picked: From no 0"[..])
    );
    range.set_until(&param(1));
    assert_eq!(
        (
            range.root_result(&g).unwrap().nb_entities(),
            range.label().as_bytes()
        ),
        (1, &b"Picked: Until no 1"[..])
    );
    range.set_one(&param(4));
    assert!(range.has_lower() && range.has_upper() && range.lower() == range.upper());
    assert_eq!(
        range.root_result(&g).unwrap().content(),
        vec![ents[3].clone()]
    );
}

#[test]
fn suites() {
    let (g, ents) = chain();
    let suite = IFSelectSelectSuite::new();
    let start = IFSelectSelectPointed::new();
    start.set_entity(&ents[3]);
    assert!(suite.add_input(&Handle::new(start).cast()) && !suite.add_input(&all()));
    let shared: Handle<dyn IFSelectSelectDeduct> = Handle::new(IFSelectSelectShared::new()).cast();
    suite.add_next(&shared);
    suite.add_next(&Handle::new(IFSelectSelectShared::new()).cast());
    assert_eq!(suite.nb_items(), 2);
    assert!(suite.item(1).unwrap() == shared && suite.item(3).is_err());
    assert_eq!(
        suite.root_result(&g).unwrap().nb_entities(),
        2,
        "d, then c, then a and b"
    );
    suite.set_label("chain");
    assert_eq!(suite.label().as_bytes(), b"chain");
    let other = IFSelectSelectSuite::new();
    assert!(other.add_input(&shared.cast()) && other.nb_items() == 1 && !other.has_input());
    other.add_previous(&Handle::new(IFSelectSelectSharing::new()).cast());
    assert_eq!(other.nb_items(), 2);
}

/// An exploration of the shared entities.
struct FakeExplore {
    transient: StandardTransient,
    base: IFSelectSelectExploreBase,
}
impl_transient!(FakeExplore, "Fake_Explore", dyn IFSelectSelectExplore, transient, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectDeduct, dyn IFSelectSelectExplore]);

impl IFSelectSelectDeduct for FakeExplore {
    fn deduct_base(&self) -> &IFSelectSelectDeductBase {
        &self.base.deduct
    }
}

impl IFSelectSelectExplore for FakeExplore {
    fn explore_base(&self) -> &IFSelectSelectExploreBase {
        &self.base
    }
    fn explore(
        &self,
        _level: i32,
        ent: &Handle<StandardTransient>,
        g: &InterfaceGraph,
        explored: &mut InterfaceEntityIterator,
    ) -> Result<bool, Failure> {
        *explored = g.shareds(ent)?;
        Ok(true)
    }
    fn explore_label(&self) -> AsciiString {
        AsciiString::new(" Shareds")
    }
}

impl IFSelectSelection for FakeExplore {
    fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        explore_root_result(self, g)
    }
    fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator) {
        deduct_fill_iterator(self, iter)
    }
    fn label(&self) -> AsciiString {
        explore_label(self)
    }
}

#[test]
fn explorations() {
    let (g, ents) = chain();
    let recursive = FakeExplore {
        transient: StandardTransient::default(),
        base: IFSelectSelectExploreBase::new(0),
    };
    let start = IFSelectSelectPointed::new();
    start.set_entity(&ents[3]);
    recursive.set_input(&Handle::new(start).cast());
    assert_eq!(
        recursive.root_result(&g).unwrap().content(),
        vec![ents[0].clone(), ents[1].clone()],
        "the leaves"
    );
    assert_eq!(
        (recursive.level(), recursive.label().as_bytes()),
        (0, &b"(Recursive) Shareds"[..])
    );
    let one = FakeExplore {
        transient: StandardTransient::default(),
        base: IFSelectSelectExploreBase::new(1),
    };
    one.set_input(&all());
    assert_eq!(
        one.root_result(&g).unwrap().nb_entities(),
        3,
        "a and b kept, then c, a, b explored at level 1"
    );
    assert_eq!(one.label().as_bytes(), b"(Level 1) Shareds");
}

/// A list selection over the entities shared by a fake entity.
#[derive(Default)]
struct FakeList {
    transient: StandardTransient,
    base: IFSelectSelectAnyListBase,
}
impl_transient!(FakeList, "Fake_List", dyn IFSelectSelectAnyList, transient, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectDeduct, dyn IFSelectSelectAnyList]);

impl IFSelectSelectDeduct for FakeList {
    fn deduct_base(&self) -> &IFSelectSelectDeductBase {
        &self.base.deduct
    }
}

impl IFSelectSelectAnyList for FakeList {
    fn any_list_base(&self) -> &IFSelectSelectAnyListBase {
        &self.base
    }
    fn keep_input_entity(&self, _iter: &mut InterfaceEntityIterator) {}
    fn nb_items(&self, ent: &Handle<StandardTransient>) -> i32 {
        ent.cast::<FakeEntity>()
            .get()
            .map_or(0, |e| e.shared.read().len() as i32)
    }
    fn fill_result(
        &self,
        n1: i32,
        n2: i32,
        ent: &Handle<StandardTransient>,
        res: &mut InterfaceEntityIterator,
    ) -> Result<(), Failure> {
        let shared = ent
            .cast::<FakeEntity>()
            .get()
            .map(|e| e.shared.read().clone())
            .unwrap_or_default();
        (n1..=n2).for_each(|i| res.add_item(&shared[i as usize - 1]));
        Ok(())
    }
    fn list_label(&self) -> AsciiString {
        AsciiString::new("Shared")
    }
}

impl IFSelectSelection for FakeList {
    fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        any_list_root_result(self, g)
    }
    fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator) {
        deduct_fill_iterator(self, iter)
    }
    fn label(&self) -> AsciiString {
        any_list_label(self)
    }
}

#[test]
fn any_lists() {
    let (g, ents) = chain();
    let list = FakeList::default();
    assert_eq!(list.root_result(&g).unwrap().nb_entities(), 0);
    list.set_input(&all());
    assert_eq!(
        list.root_result(&g)
            .err()
            .map(|e| e.message().to_owned())
            .unwrap_or_default(),
        "SelectAnyList : more than ONE Entity in input"
    );
    let one = IFSelectSelectPointed::new();
    one.set_entity(&ents[2]);
    list.set_input(&Handle::new(one).cast());
    assert_eq!(list.root_result(&g).unwrap().nb_entities(), 2);
    assert_eq!(list.label().as_bytes(), b"In List Shared (no 0)");
    list.any_list_base().range.set_one(&param(2));
    assert_eq!(
        list.root_result(&g).unwrap().content(),
        vec![ents[1].clone()]
    );
    list.any_list_base().range.set_from(&param(5));
    assert_eq!(
        (
            list.root_result(&g).unwrap().nb_entities(),
            list.label().as_bytes()
        ),
        (0, &b"In List Shared (0 ->)"[..])
    );
    list.any_list_base().range.set_until(&param(1));
    assert_eq!(list.label().as_bytes(), b"In List Shared (-> 0)");
    list.any_list_base().range.set_range(&param(1), &param(2));
    assert_eq!(list.label().as_bytes(), b"In List Shared (1 -> 2)");
}
