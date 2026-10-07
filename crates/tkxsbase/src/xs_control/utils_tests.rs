//! Unit tests of `XSControl_Utils`: texts, lists, arrays and shapes.
use super::*;
use crate::transfer_brep::TransferBRepShapeMapper;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::{
    ncollection::{HArray1, HSequence},
    standard::{Handle, RuntimeTyped, StandardTransient},
    tcollection::{AsciiString, HAsciiString, HExtendedString},
};
use rustycad_tkg3d::top_abs::ShapeEnum;

fn edge() -> Shape {
    let mut e = Edge::default();
    BRepBuilder::default().make_edge(&mut e).unwrap();
    e.into_shape()
}

fn face() -> Shape {
    let mut f = Face::default();
    BRepBuilder::default().make_face(&mut f);
    f.into_shape()
}

#[test]
fn utils_texts() {
    let xu = XSControlUtils::new();
    let h: Handle<StandardTransient> = Handle::new(HAsciiString::new("abc")).cast();
    assert_eq!(xu.type_name(&h, false), "TCollection_HAsciiString");
    assert_eq!(xu.type_name(&h, true), "HAsciiString");
    assert_eq!(
        xu.type_name(&HAsciiString::static_type_handle().cast(), true),
        "HAsciiString",
        "a type gives its name"
    );
    assert!(xu.type_name(&Handle::default(), false).is_empty());
    assert!(xu.is_kind(&h, Some(StandardTransient::static_type())) && !xu.is_kind(&h, None));
    assert_eq!(xu.to_c_string(&Handle::default()).len(), 0);
    assert_eq!(xu.to_c_string(&xu.to_h_string("x")).as_bytes(), b"x");
    assert_eq!(xu.to_a_string("y").as_bytes(), b"y");
    let units = xu.ascii_to_extended("Ab");
    assert_eq!(units, vec![0x41, 0x62]);
    assert!(xu.is_ascii(&units) && !xu.is_ascii(&[0x3A9]));
    assert_eq!(
        xu.extended_to_ascii(&[0x3A9]).as_bytes(),
        &[0x29],
        "the 7 low bits"
    );
    assert_eq!(xu.to_e_string(&xu.to_h_string_ext(&units)), units);
    assert_eq!(xu.to_e_string_ext(&xu.to_x_string(&units)), units);
    assert!(xu.to_e_string(&Handle::default()).is_empty());
    assert_eq!(
        xu.to_c_string_ascii(&AsciiString::new("z")).as_bytes(),
        b"z"
    );
    let date = xu.date_string(1999, 1, 22, 12, 40, 45).unwrap();
    assert_eq!(date.as_bytes(), b"1999-01-22:12-40-45");
    assert_eq!(
        xu.date_values("1999-01-22:12-40-45"),
        [1999, 1, 22, 12, 40, 45]
    );
    xu.trace_line("trace");
    xu.trace_lines(&h);
}

#[test]
fn utils_lists() {
    let xu = XSControlUtils::new();
    let texts = xu.new_seq_c_str();
    xu.append_c_str(&texts, "one").unwrap();
    xu.append_c_str(&texts, "two").unwrap();
    let list: Handle<StandardTransient> = texts.clone().cast();
    assert_eq!(xu.seq_length(&list), 2);
    assert_eq!(xu.c_str_value(&list, 2).unwrap().as_bytes(), b"two");
    assert!(xu.c_str_value(&list, 3).unwrap().is_empty() && xu.c_str_value(&list, 0).is_err());
    assert_eq!(
        xu.e_str_value(&list, 1).unwrap(),
        xu.ascii_to_extended("one")
    );
    assert_eq!(
        xu.tra_value(&list, 1)
            .cast::<HAsciiString>()
            .string()
            .as_bytes(),
        b"one"
    );
    assert!(xu.tra_value(&list, 0).is_null());
    xu.trace_lines(&list);
    let ascii = HSequence::<AsciiString>::default();
    ascii.append(AsciiString::new("a"));
    let ascii: Handle<StandardTransient> = Handle::new(ascii).cast();
    assert_eq!(xu.c_str_value(&ascii, 1).unwrap().as_bytes(), b"a");
    assert!(
        xu.e_str_value(&ascii, 1).unwrap().is_empty(),
        "native defect: no units for an AsciiString list"
    );
    let ext = xu.new_seq_e_str();
    xu.append_e_str(&ext, &[0x41]).unwrap();
    assert_eq!(xu.c_str_value(&ext.cast(), 1).unwrap().as_bytes(), b"A");
    assert_eq!(
        xu.e_str_value(&Handle::new(HExtendedString::from_utf16(&[0x42])).cast(), 1)
            .unwrap(),
        vec![0x42]
    );
    let tra = xu.new_seq_tra();
    xu.append_tra(&tra, &list).unwrap();
    assert!(
        xu.tra_value(&tra.clone().cast(), 1) == list && xu.seq_length(&tra.clone().cast()) == 1
    );
    assert!(xu.append_tra(&Handle::default(), &list).is_err());
    let ints = HSequence::<i32>::default();
    ints.append(7);
    let ints = Handle::new(ints);
    assert_eq!(
        (
            xu.seq_int_value(&ints, 1).unwrap(),
            xu.seq_int_value(&Handle::default(), 1).unwrap()
        ),
        (7, 0)
    );
    assert!(xu.seq_int_value(&ints, 2).is_err());
    assert_eq!(xu.seq_length(&ints.cast()), 1);
}

#[test]
fn utils_arrays() {
    let xu = XSControlUtils::new();
    let texts = xu.new_seq_c_str();
    xu.append_c_str(&texts, "a").unwrap();
    xu.append_c_str(&texts, "b").unwrap();
    let arr = xu.seq_to_arr(&texts.clone().cast(), 1).unwrap();
    let array = arr.cast::<HArray1<Handle<HAsciiString>>>();
    assert_eq!((array.array1().lower(), array.array1().upper()), (1, 2));
    let back = xu.arr_to_seq(&arr).unwrap();
    assert_eq!(xu.seq_length(&back), 2);
    assert!(
        xu.seq_to_arr(&texts.clone().cast(), 2).is_err(),
        "native quirk: items below the bounds"
    );
    assert!(
        xu.seq_to_arr(&Handle::new(HAsciiString::new("x")).cast(), 1)
            .is_err()
    );
    assert!(
        xu.arr_to_seq(&Handle::new(HAsciiString::new("x")).cast())
            .is_err()
    );
    assert!(xu.seq_to_arr(&Handle::default(), 1).unwrap().is_null());
}

#[test]
fn utils_shapes() {
    let xu = XSControlUtils::new();
    let seq = xu.new_seq_shape();
    xu.append_shape(&seq, &edge()).unwrap();
    xu.append_shape(&seq, &edge()).unwrap();
    let compound = xu.compound_from_seq(&seq).unwrap();
    assert_eq!(xu.shape_type(&compound, true).unwrap(), ShapeEnum::Edge);
    assert_eq!(
        xu.shape_type(&compound, false).unwrap(),
        ShapeEnum::Compound
    );
    assert!(
        xu.shape_value(&seq, 2).is_same(&xu.shape_value(&seq, 2))
            && xu.shape_value(&seq, 3).is_null()
    );
    let wires = xu
        .sorted_compound(&compound, ShapeEnum::Wire, false, false)
        .unwrap();
    assert_eq!(
        xu.shape_type(&wires, true).unwrap(),
        ShapeEnum::Wire,
        "the edges become wires"
    );
    let shell = xu
        .sorted_compound(&face(), ShapeEnum::Shell, false, false)
        .unwrap();
    assert_eq!(shell.shape_type().unwrap(), ShapeEnum::Shell);
    assert!(
        xu.sorted_compound(&face(), ShapeEnum::Edge, false, false)
            .unwrap()
            .is_null(),
        "no exploration"
    );
    assert!(
        xu.sorted_compound(&face(), ShapeEnum::Edge, true, false)
            .unwrap()
            .is_null(),
        "a bare face has no edge"
    );
    let one = xu.new_seq_shape();
    xu.append_shape(&one, &edge()).unwrap();
    let single = xu
        .sorted_compound(
            &xu.compound_from_seq(&one).unwrap(),
            ShapeEnum::Edge,
            false,
            false,
        )
        .unwrap();
    assert_eq!(
        single.shape_type().unwrap(),
        ShapeEnum::Edge,
        "one item alone"
    );
    let e = edge();
    for hs in [true, false] {
        assert!(xu.binder_shape(&xu.shape_binder(&e, hs)).is_same(&e));
    }
    let mapper: Handle<StandardTransient> = Handle::new(TransferBRepShapeMapper::new(&e)).cast();
    assert!(xu.binder_shape(&mapper).is_same(&e));
    assert!(xu.binder_shape(&Handle::default()).is_null());
}
