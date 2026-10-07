//! 1:1 translation of src/ApplicationFramework/TKLCAF/GTests/TDataStd_List_Test.cxx.
#![allow(non_snake_case)]
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{
    TDataStdBooleanList, TDataStdExtStringArray, TDataStdExtStringList, TDataStdIntegerList,
    TDataStdRealList, TDataStdReferenceList,
};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;

fn text(value: &str) -> ExtendedString {
    ExtendedString::new(value)
}

#[test]
fn TDataStd_List_Test__CafBug_23465_ListInsertRemove() {
    let document = TDocStdDocument::new(&text("XmlOcaf")).unwrap();
    let label = |tag| document.main().unwrap().find_child(tag, true).unwrap();
    let (label1, label2, label3, label4, label5) =
        (label(1), label(2), label(3), label(4), label(5));

    let boolean_list = TDataStdBooleanList::set(&label1).unwrap();
    for value in [false, false, true, false] {
        boolean_list.append(value).unwrap();
    }
    let integer_list = TDataStdIntegerList::set(&label2).unwrap();
    for value in [0, 0, 1, 0] {
        integer_list.append(value).unwrap();
    }
    let real_list = TDataStdRealList::set(&label3).unwrap();
    for value in [0.0, 0.0, 1.0, 0.0] {
        real_list.append(value).unwrap();
    }
    let ext_string_list = TDataStdExtStringList::set(&label4).unwrap();
    for value in ["0", "0", "1", "0"] {
        ext_string_list.append(text(value)).unwrap();
    }
    let reference_list = TDataStdReferenceList::set(&label5).unwrap();
    reference_list.append(label1.clone()).unwrap();
    reference_list.append(label2.clone()).unwrap();
    reference_list.append(label4.clone()).unwrap();

    assert!(boolean_list.insert_before_by_index(3, true).unwrap());
    assert!(integer_list.insert_before_by_index(3, 1).unwrap());
    assert!(real_list.insert_before_by_index(3, 1.0).unwrap());
    assert!(
        ext_string_list
            .insert_before_by_index(3, text("1"))
            .unwrap()
    );
    assert!(
        reference_list
            .insert_before_by_index(3, label3.clone())
            .unwrap()
    );

    assert!(boolean_list.insert_after_by_index(4, true).unwrap());
    assert!(integer_list.insert_after_by_index(4, 1).unwrap());
    assert!(real_list.insert_after_by_index(4, 1.0).unwrap());
    assert!(ext_string_list.insert_after_by_index(4, text("1")).unwrap());
    assert!(
        reference_list
            .insert_after_by_index(4, label5.clone())
            .unwrap()
    );

    assert!(boolean_list.remove_by_index(6).unwrap());
    assert!(integer_list.remove_by_index(6).unwrap());
    assert!(real_list.remove_by_index(6).unwrap());
    assert!(ext_string_list.remove_by_index(6).unwrap());
    assert!(reference_list.remove_by_index(1).unwrap());

    assert_eq!(boolean_list.extent(), 5);
    assert_eq!(boolean_list.list(), vec![0u8, 0, 1, 1, 1]);
    assert_eq!(integer_list.extent(), 5);
    assert_eq!(integer_list.list(), vec![0, 0, 1, 1, 1]);
    assert_eq!(real_list.extent(), 5);
    assert_eq!(real_list.list(), vec![0.0, 0.0, 1.0, 1.0, 1.0]);
    assert_eq!(ext_string_list.extent(), 5);
    let expected_strings = ["0", "0", "1", "1", "1"];
    for (value, expected) in ext_string_list.list().iter().zip(expected_strings) {
        assert!(value.is_equal(text(expected)));
    }
    assert_eq!(reference_list.extent(), 4);
    assert_eq!(reference_list.list(), vec![label2, label3, label4, label5]);
}

// caf/bugs/bug23912: an extended-string array keeps each assigned value at its
// own index.
#[test]
fn TDataStd_List_Test__CafBug_23912_ExtStringArrayValues() {
    let document = TDocStdDocument::new(&text("BinOcaf")).unwrap();
    let array = TDataStdExtStringArray::set(&document.main().unwrap(), 1, 5, false).unwrap();
    assert!(!array.is_null());
    let expected = ["A", "B", "C", "D", "E"];
    for (index, value) in (1..=5).zip(expected) {
        array.set_value(index, text(value)).unwrap();
    }
    for (index, value) in (1..=5).zip(expected) {
        assert!(array.value(index).unwrap().is_equal(text(value)));
    }
}
