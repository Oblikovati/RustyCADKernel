//! TObj partitions, naming, ordered children, persistence and cloning, against the
//! semantics of TObj_Partition.cxx, TObj_Object.cxx and TObj_Persistence.cxx (OCCT
//! 3d097a0); the expected values are derived from the native bodies.
#![allow(non_snake_case)]
use crate::tobj_model::{ObjectFixture, TObjTestObject, h_name};
use rustycad_tkernel::{
    ncollection::HArray1,
    standard::{Handle, RuntimeTyped},
    tcollection::{AsciiString, ExtendedString},
};
use rustycad_tklcaf::tdf::TdfRelocationTable;
use rustycad_tktobj::tobj::{
    TObjHiddenPartition, TObjModel, TObjModelState, TObjObject, TObjObjectIterator,
    TObjObjectState, TObjObjectStateFlags, TObjPartition, TObjPersistence, TObjTypeFlags,
};

/// The ASCII names of the objects an iterator visits, in order.
fn names_of(iterator: &Handle<dyn TObjObjectIterator>) -> Vec<String> {
    let mut names = Vec::new();
    while iterator.more() {
        let mut name = AsciiString::default();
        iterator.value().get_name_ascii(&mut name).unwrap();
        names.push(String::from_utf8_lossy(name.as_bytes()).into_owned());
        iterator.next().unwrap();
    }
    names
}

#[test]
fn a_partition_prefix_names_new_objects() {
    let fixture = ObjectFixture::set_up();
    fixture.my_partition.set_name_prefix(&h_name("Obj_"));
    let objects = fixture.command(|| {
        [
            fixture.create_object(),
            fixture.create_object(),
            fixture.create_object(),
        ]
    });
    let names = names_of(&fixture.my_model().get_children().unwrap());
    assert_eq!(names, ["Obj_1", "Obj_2", "Obj_3"]);
    // The last index moves only past a name that was already taken (the second one).
    assert_eq!(fixture.my_partition.get_last_index().unwrap(), 2);
    assert!(
        objects[0]
            .get_name()
            .unwrap()
            .string()
            .is_equal(ExtendedString::new("Obj_1"))
    );
}

#[test]
fn a_new_name_without_count_keeps_the_last_index() {
    let fixture = ObjectFixture::set_up();
    fixture.my_partition.set_name_prefix(&h_name("P"));
    let name = fixture.command(|| fixture.my_partition.get_new_name(false).unwrap());
    assert!(name.string().is_equal(ExtendedString::new("P1")));
    assert_eq!(fixture.my_partition.get_last_index().unwrap(), 0);
    assert!(
        fixture
            .my_partition
            .get_name_prefix()
            .string()
            .is_equal(ExtendedString::new("P"))
    );
}

#[test]
fn a_partition_name_is_not_registered() {
    let fixture = ObjectFixture::set_up();
    let name = h_name("Main");
    fixture.command(|| assert!(fixture.my_partition.set_name(&name).unwrap()));
    let dictionary = fixture.my_model().get_dictionary().unwrap();
    assert!(
        !fixture
            .my_model()
            .is_registered_name(&name, &dictionary)
            .unwrap()
    );
    assert!(
        fixture
            .my_partition
            .get_name()
            .unwrap()
            .string()
            .is_equal(ExtendedString::new("Main"))
    );
}

#[test]
fn a_hidden_sub_partition_is_named_and_not_visible() {
    let fixture = ObjectFixture::set_up();
    let name = ExtendedString::new("Hidden");
    let hidden = fixture.command(|| {
        fixture
            .my_model()
            .get_partition_of_main(3, &name, true)
            .unwrap()
    });
    assert!(!hidden.cast::<TObjHiddenPartition>().is_null());
    assert_eq!(hidden.get_type_flags() & TObjTypeFlags::VISIBLE, 0);
    assert!(hidden.get_name().unwrap().string().is_equal(name.clone()));
    // The same index gives the same partition.
    let again = fixture.command(|| {
        fixture
            .my_model()
            .get_partition_of_main(3, &name, true)
            .unwrap()
    });
    assert!(again == hidden);
    assert!(<dyn TObjPartition>::get_partition(&hidden.cast()).unwrap() == fixture.my_partition);
}

#[test]
fn ordered_children_come_by_their_order() {
    let fixture = ObjectFixture::set_up();
    let parent = fixture.command(|| {
        let parent = fixture.create_object();
        for (index, order) in [30, 10, 20, 40, 5, 20].into_iter().enumerate() {
            let child = parent.add_child();
            assert!(child.set_name_c_str(&format!("c{index}")).unwrap());
            child.set_order(order).unwrap();
        }
        parent.set_flags(TObjObjectStateFlags::ORDERED).unwrap();
        parent
    });
    let iterator = parent.get_children(None).unwrap();
    let mut orders = Vec::new();
    while iterator.more() {
        orders.push(iterator.value().get_order().unwrap());
        iterator.next().unwrap();
    }
    assert_eq!(orders, [5, 10, 20, 20, 30, 40]);
}

#[test]
fn the_model_iterator_visits_depth_first_from_the_root() {
    let fixture = ObjectFixture::set_up();
    fixture.command(|| {
        let first = fixture.create_object();
        first.set_name_c_str("a").unwrap();
        first.add_child().set_name_c_str("a1").unwrap();
        fixture.create_object().set_name_c_str("b").unwrap();
    });
    let names = names_of(&fixture.my_model().get_objects().unwrap());
    // The root is the unnamed main partition.
    assert_eq!(names, ["", "a", "a1", "b"]);
}

#[test]
fn the_persistence_creates_registered_types_only() {
    let fixture = ObjectFixture::set_up();
    let label = fixture.command(|| fixture.my_partition.new_label().unwrap());
    let created = TObjPersistence::create_new_object("TObj_Partition", &label).unwrap();
    assert!(created.is_kind(<dyn TObjPartition as RuntimeTyped>::static_type()));
    // The persistence constructor does not attach the object.
    assert!(!created.is_alive().unwrap());
    assert!(
        TObjPersistence::create_new_object("No_Such_Type", &label)
            .unwrap()
            .is_null()
    );
    let mut names = Vec::new();
    TObjPersistence::dump_types(&mut names).unwrap();
    let names = String::from_utf8(names).unwrap();
    assert!(names.contains("TObj_Partition\n") && names.contains("TObj_HiddenPartition\n"));
}

#[test]
fn a_clone_copies_data_children_and_references() {
    let fixture = ObjectFixture::set_up();
    let (source, referred) = fixture.command(|| {
        let source = fixture.create_object();
        source.set_name_c_str("src").unwrap();
        source.set_int(7);
        let values = Handle::new(HArray1::with_value(1, 2, 1.5).unwrap());
        source.set_real_arr(&values);
        let child = source.add_child();
        child.set_name_c_str("kid").unwrap();
        child.set_int(9);
        let referred = fixture.create_object();
        source.set_ref(&referred.cast());
        (source, referred)
    });
    let clone = fixture.command(|| {
        let target = fixture.my_partition.new_label().unwrap();
        source.clone_object(&target, &Handle::default()).unwrap()
    });
    let clone: Handle<TObjTestObject> = clone.cast();
    assert!(!clone.is_null() && clone.is_alive().unwrap());
    assert_eq!(clone.get_int(), 7);
    assert_eq!(clone.get_real_arr().array1().as_slice(), &[1.5, 1.5]);
    // The name is taken already, so the clone stays unnamed.
    assert!(clone.get_name().unwrap().is_empty());
    let children = clone.get_children(None).unwrap();
    assert!(children.more());
    let child: Handle<TObjTestObject> = children.value().cast();
    assert_eq!(child.get_int(), 9);
    assert!(clone.get_ref() == referred);
    let back = referred.get_back_references(None);
    let mut referring = 0;
    while back.more() {
        referring += 1;
        back.next().unwrap();
    }
    assert_eq!(referring, 2);
}

#[test]
fn a_model_pastes_its_objects_into_another() {
    let fixture = ObjectFixture::set_up();
    fixture.command(|| {
        let object = fixture.create_object();
        object.set_name_c_str("one").unwrap();
        object.set_int(11);
    });
    let other = crate::tobj_model::TObjTestModel::new();
    assert!(other.load(&ExtendedString::default()).unwrap());
    other.open_command().unwrap();
    fixture.my_model().open_command().unwrap();
    let table = Handle::new(TdfRelocationTable::new(false));
    assert!(
        fixture
            .my_model()
            .paste(&other.clone().cast(), &table)
            .unwrap()
    );
    fixture.my_model().commit_command().unwrap();
    other.commit_command().unwrap();
    let children = other.get_children().unwrap();
    assert!(children.more());
    let pasted: Handle<TObjTestObject> = children.value().cast();
    assert_eq!(pasted.get_int(), 11);
    assert!(other.close().unwrap());
}
