//! TObj references, back references and deletion modes, against the semantics of
//! TObj_Object.cxx and TObj_TReference.cxx (OCCT 3d097a0): every expected value below is
//! derived from the native bodies, not from a run of the port.
#![allow(non_snake_case)]
use crate::tobj_model::{ObjectFixture, TObjTestObject};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, RuntimeTyped, StandardTransient},
};
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tktobj::tobj::{
    TObjCheckModel, TObjDeletingMode, TObjModel, TObjModelState, TObjObject, TObjObjectBase,
    TObjObjectData, TObjObjectIterator, TObjObjectState, TObjPartition,
};

/// A container object: it can drop its references (`CanRemoveReference` is true).
pub struct TObjTestContainer {
    transient: StandardTransient,
    base: TObjObjectBase,
}
impl_transient!(TObjTestContainer, "TObj_TestContainer", dyn TObjObject, transient,
    interfaces: [dyn TObjObject]);
impl TObjObject for TObjTestContainer {
    fn object_base(&self) -> &TObjObjectBase {
        &self.base
    }
    fn can_remove_reference(&self, _object: &Handle<dyn TObjObject>) -> Result<bool, Failure> {
        Ok(true)
    }
}
impl TObjTestContainer {
    /// A new container on the label.
    pub fn new(label: &TdfLabel) -> Handle<TObjTestContainer> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: TObjObjectBase::new(label),
        });
        TObjObjectBase::attach(&object.cast(), true).unwrap();
        object
    }
}

/// Every object an iterator visits.
fn visited(iterator: &Handle<dyn TObjObjectIterator>) -> Vec<Handle<dyn TObjObject>> {
    let mut objects = Vec::new();
    while iterator.more() {
        objects.push(iterator.value());
        iterator.next().unwrap();
    }
    objects
}

fn dyn_object<T: TObjObject + RuntimeTyped>(object: &Handle<T>) -> Handle<dyn TObjObject> {
    object.cast()
}

#[test]
fn reference_registers_a_back_reference_and_is_found() {
    let fixture = ObjectFixture::set_up();
    let (first, second) = fixture.command(|| (fixture.create_object(), fixture.create_object()));
    fixture.command(|| first.set_ref(&dyn_object(&second)));
    assert!(first.get_ref() == second);
    assert!(first.has_reference(&dyn_object(&second)).unwrap());
    assert!(!second.has_reference(&dyn_object(&first)).unwrap());
    let references = visited(&first.get_references(None).unwrap());
    assert_eq!(references.len(), 1);
    assert!(references[0] == second);
    let back = visited(&second.get_back_references(None));
    assert_eq!(back.len(), 1);
    assert!(back[0] == first);
    assert!(second.has_back_references());
}

#[test]
fn a_referenced_object_detaches_only_as_the_mode_allows() {
    let fixture = ObjectFixture::set_up();
    let (first, second) = fixture.command(|| (fixture.create_object(), fixture.create_object()));
    fixture.command(|| first.set_ref(&dyn_object(&second)));
    assert!(!second.can_detach(TObjDeletingMode::FreeOnly).unwrap());
    // A plain object cannot drop its reference, so KeepDepending refuses too.
    assert!(!second.can_detach(TObjDeletingMode::KeepDepending).unwrap());
    assert!(second.can_detach(TObjDeletingMode::Forced).unwrap());
    fixture.command(|| assert!(!second.detach(TObjDeletingMode::FreeOnly).unwrap()));
    assert!(second.is_alive().unwrap());
    // Forced: the strong referrer is detached with it.
    fixture.command(|| assert!(second.detach(TObjDeletingMode::Forced).unwrap()));
    assert!(!second.is_alive().unwrap());
    assert!(!first.is_alive().unwrap());
}

#[test]
fn keep_depending_unlinks_a_container_and_keeps_it() {
    let fixture = ObjectFixture::set_up();
    let (container, target) = fixture.command(|| {
        let container = TObjTestContainer::new(&fixture.my_partition.new_label().unwrap());
        (container, fixture.create_object())
    });
    fixture.command(|| container.set_reference(&dyn_object(&target), 1, 0).unwrap());
    assert!(target.can_detach(TObjDeletingMode::KeepDepending).unwrap());
    fixture.command(|| assert!(target.detach(TObjDeletingMode::KeepDepending).unwrap()));
    assert!(!target.is_alive().unwrap());
    assert!(container.is_alive().unwrap());
    assert!(visited(&container.get_references(None).unwrap()).is_empty());
}

#[test]
fn replacing_a_reference_moves_the_back_reference() {
    let fixture = ObjectFixture::set_up();
    let (first, second, third) = fixture.command(|| {
        (
            fixture.create_object(),
            fixture.create_object(),
            fixture.create_object(),
        )
    });
    fixture.command(|| first.set_ref(&dyn_object(&second)));
    fixture.command(|| {
        first
            .replace_reference(&dyn_object(&second), &dyn_object(&third))
            .unwrap()
    });
    assert!(first.get_ref() == third);
    assert!(!second.has_back_references());
    assert!(visited(&third.get_back_references(None))[0] == first);
    // A null new object removes the reference.
    fixture.command(|| first.remove_reference(&dyn_object(&third)).unwrap());
    assert!(first.get_ref().is_null());
    assert!(!third.has_back_references());
}

#[test]
fn removing_all_references_clears_the_back_references() {
    let fixture = ObjectFixture::set_up();
    let (first, second) = fixture.command(|| (fixture.create_object(), fixture.create_object()));
    let added = fixture.command(|| first.add_reference(2, &dyn_object(&second)).unwrap());
    assert_eq!(added.father().tag(), 2);
    assert_eq!(visited(&first.get_references(None).unwrap()).len(), 1);
    assert!(second.has_back_references());
    fixture.command(|| first.remove_all_references().unwrap());
    assert!(visited(&first.get_references(None).unwrap()).is_empty());
    assert!(!second.has_back_references());
}

#[test]
fn undoing_a_reference_removes_its_back_reference() {
    let fixture = ObjectFixture::set_up();
    let document = fixture.my_model().get_document().unwrap();
    document.set_undo_limit(10).unwrap();
    let (first, second) = fixture.command(|| (fixture.create_object(), fixture.create_object()));
    fixture.command(|| first.set_ref(&dyn_object(&second)));
    assert!(second.has_back_references());
    assert!(document.undo().unwrap());
    assert!(first.get_ref().is_null());
    assert!(!second.has_back_references());
}

#[test]
fn bad_references_point_outside_the_root() {
    let fixture = ObjectFixture::set_up();
    let (first, second) = fixture.command(|| (fixture.create_object(), fixture.create_object()));
    fixture.command(|| first.set_ref(&dyn_object(&second)));
    let mut bad = TdfLabel::default();
    assert!(
        first
            .get_bad_reference(&first.get_label(), &mut bad)
            .unwrap()
    );
    assert!(bad == second.get_label());
    let mut none = TdfLabel::default();
    let model_label = fixture.my_model().get_label();
    assert!(!first.get_bad_reference(&model_label, &mut none).unwrap());
    assert!(none.is_null());
}

#[test]
fn relocating_references_follows_the_equivalent_label() {
    let fixture = ObjectFixture::set_up();
    let (first, second, third) = fixture.command(|| {
        (
            fixture.create_object(),
            fixture.create_object(),
            fixture.create_object(),
        )
    });
    fixture.command(|| first.set_ref(&dyn_object(&second)));
    let relocated = fixture.command(|| {
        first
            .relocate_references(&second.get_label(), &third.get_label(), true)
            .unwrap()
    });
    assert!(relocated);
    assert!(first.get_ref() == third);
    assert!(!second.has_back_references());
    assert!(visited(&third.get_back_references(None))[0] == first);
}

#[test]
fn the_father_object_of_a_type_is_found_above() {
    let fixture = ObjectFixture::set_up();
    let (parent, child) = fixture.command(|| {
        let parent = fixture.create_object();
        let child = parent.add_child();
        (parent, child)
    });
    assert!(child.get_father_object(None).unwrap() == parent);
    let partition_type = <dyn TObjPartition as RuntimeTyped>::static_type();
    let father = child.get_father_object(Some(partition_type)).unwrap();
    assert!(father == fixture.my_partition);
    let child_type = TObjTestObject::static_type();
    assert!(child.get_father_object(Some(child_type)).unwrap() == parent);
}

#[test]
fn the_checker_finds_and_fixes_a_stale_back_reference() {
    let fixture = ObjectFixture::set_up();
    let (first, second) = fixture.command(|| (fixture.create_object(), fixture.create_object()));
    fixture.command(|| first.set_ref(&dyn_object(&second)));
    let checker: Handle<TObjCheckModel> = fixture.my_model().get_checker().unwrap();
    assert!(checker.perform().unwrap());
    // A back reference without the matching reference is an alarm.
    first.add_back_reference(&dyn_object(&second));
    assert!(!checker.perform().unwrap());
    checker.set_to_fix(true);
    assert!(checker.is_to_fix());
    assert!(checker.perform().unwrap());
    assert!(!first.has_back_references());
    assert!(checker.perform().unwrap());
}
