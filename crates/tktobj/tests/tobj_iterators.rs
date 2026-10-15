//! The TObj iterators and their protected state, against the semantics of
//! TObj_SequenceIterator.cxx, TObj_OcafObjectIterator.cxx, TObj_ReferenceIterator.cxx,
//! TObj_LabelIterator.cxx and TObj_ModelIterator.cxx (OCCT 3d097a0).
#![allow(non_snake_case)]
use crate::tobj_model::{ObjectFixture, TObjTestObject};
use crate::tobj_references::TObjTestContainer;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tktobj::tobj::{
    TObjBackReferences, TObjLabelIterator, TObjLabelIteratorBase, TObjModel, TObjModelIterator,
    TObjObjectData, TObjObjectIterator, TObjObjectState, TObjOcafObjectIterator, TObjPersistence,
    TObjReferenceIterator, TObjSequenceIterator, label_iterator_more,
};

#[test]
fn a_sequence_iterator_skips_other_types_and_stops_at_a_null_entry() {
    let fixture = ObjectFixture::set_up();
    let (plain, container) = fixture.command(|| {
        let plain = fixture.create_object();
        let container = TObjTestContainer::new(&fixture.my_partition.new_label().unwrap());
        (plain, container)
    });
    let sequence = Handle::new(TObjBackReferences::default());
    for object in [
        container.cast(),
        plain.cast(),
        Handle::default(),
        plain.cast(),
    ] {
        sequence.append(object);
    }
    let kind = TObjTestObject::static_type();
    let iterator = TObjSequenceIterator::new(&sequence, Some(kind));
    assert!(std::ptr::eq(iterator.kind().unwrap(), kind));
    assert!(iterator.objects() == sequence);
    // More moves past the container, as the native const member does.
    assert!(iterator.more());
    assert_eq!(iterator.index(), 2);
    assert!(iterator.value() == plain);
    iterator.next().unwrap();
    // A null entry ends the iteration, although an object follows it.
    assert!(!iterator.more());
    assert!(!TObjSequenceIterator::empty().more());
}

#[test]
fn an_ocaf_iterator_stops_at_the_first_object_level_unless_asked() {
    let fixture = ObjectFixture::set_up();
    let parent = fixture.command(|| {
        let parent = fixture.create_object();
        parent.add_child().add_child();
        parent
    });
    let label = parent.get_child_label().unwrap();
    let first_level = TObjOcafObjectIterator::new(&label, None, true, false).unwrap();
    assert!(first_level.kind().is_none() && !first_level.all_sub_children());
    let mut count = 0;
    while first_level.more() {
        count += 1;
        assert!(!first_level.label_value().is_null());
        first_level.next().unwrap();
    }
    assert_eq!(count, 1);
    let all = TObjOcafObjectIterator::new(&label, None, true, true).unwrap();
    assert!(all.all_sub_children());
    let mut count = 0;
    while all.more() {
        count += 1;
        all.next().unwrap();
    }
    assert_eq!(count, 2);
}

#[test]
fn a_reference_iterator_filters_the_referred_type() {
    let fixture = ObjectFixture::set_up();
    let (holder, container) = fixture.command(|| {
        let holder = fixture.create_object();
        let container = TObjTestContainer::new(&fixture.my_partition.new_label().unwrap());
        holder.set_reference(&container.cast(), 1, 0).unwrap();
        holder
            .set_reference(&fixture.create_object().cast(), 2, 0)
            .unwrap();
        (holder, container)
    });
    let label = holder.get_reference_label().unwrap();
    let kind = TObjTestObject::static_type();
    let iterator = TObjReferenceIterator::new(&label, Some(kind), true).unwrap();
    assert!(std::ptr::eq(iterator.kind().unwrap(), kind));
    assert!(iterator.more());
    assert!(iterator.value() != container);
    iterator.next().unwrap();
    assert!(!iterator.more());
    let any = TObjReferenceIterator::new(&label, None, true).unwrap();
    assert!(any.value() == container);
}

#[test]
fn a_label_iterator_base_starts_empty_or_on_a_label() {
    let fixture = ObjectFixture::set_up();
    let base = TObjLabelIteratorBase::new();
    assert!(base.node().is_null() && base.object().is_null());
    assert!(!base.child_iterator().more());
    let object = fixture.command(|| fixture.create_object());
    let started =
        TObjLabelIteratorBase::with_label(&fixture.my_partition.get_child_label().unwrap(), false)
            .unwrap();
    assert!(started.child_iterator().more());
    started.set_current(&object.cast(), &object.get_label());
    assert!(started.object() == object && started.node() == object.get_label());
    let failure = TObjLabelIteratorBase::with_label(&Default::default(), false)
        .err()
        .unwrap();
    assert!(failure.is_kind("Standard_NullObject"));
    let iterator = TObjOcafObjectIterator::new(
        &fixture.my_partition.get_child_label().unwrap(),
        None,
        false,
        false,
    )
    .unwrap();
    assert!(label_iterator_more(&*iterator));
}

#[test]
fn a_model_iterator_keeps_a_stack_of_child_iterators() {
    let fixture = ObjectFixture::set_up();
    fixture.command(|| {
        fixture.create_object().add_child();
    });
    let model: Handle<dyn TObjModel> = fixture.my_model().clone().cast();
    let iterator = TObjModelIterator::new(&model).unwrap();
    assert!(iterator.object() == fixture.my_partition);
    assert_eq!(iterator.iterators().len(), 1);
    iterator.next().unwrap();
    // The first object's children iterator is pushed on the stack.
    assert_eq!(iterator.iterators().len(), 2);
    iterator.next().unwrap();
    iterator.next().unwrap();
    assert!(!iterator.more() && iterator.iterators().is_empty());
}

#[test]
fn the_persistence_map_holds_the_registered_tools() {
    let fixture = ObjectFixture::set_up();
    let label = fixture.command(|| fixture.my_partition.new_label().unwrap());
    let has_partition = TObjPersistence::map_of_types()
        .items()
        .any(|(name, _)| name.as_bytes() == b"TObj_Partition");
    assert!(has_partition);
    let created = TObjPersistence::create_new_object("TObj_TestObject", &label).unwrap();
    assert!(created.is_kind(TObjTestObject::static_type()));
    assert!(!created.is_alive().unwrap());
}
