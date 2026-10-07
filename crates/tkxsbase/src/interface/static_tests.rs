//! Unit tests of the static functions of `Interface_Static` (the names are prefixed with
//! "test.static." because the map of static values is process-wide).
use super::{InterfaceParamType, InterfaceStatic};
use rustycad_tkernel::{ncollection::DataMap, tcollection::AsciiString};

#[test]
fn init_and_values() {
    assert!(
        InterfaceStatic::init(
            "TEST",
            "test.static.int",
            InterfaceParamType::ParamInteger,
            "4"
        )
        .unwrap()
    );
    assert!(
        !InterfaceStatic::init(
            "TEST",
            "test.static.int",
            InterfaceParamType::ParamInteger,
            "5"
        )
        .unwrap()
    );
    assert!(!InterfaceStatic::init("TEST", "", InterfaceParamType::ParamText, "").unwrap());
    assert!(InterfaceStatic::is_present("test.static.int"));
    assert_eq!(InterfaceStatic::i_val("test.static.int"), 4);
    assert!(InterfaceStatic::init_char("TEST", "test.static.int", b'&', "imax 10").unwrap());
    assert!(!InterfaceStatic::set_i_val("test.static.int", 11).unwrap());
    assert!(InterfaceStatic::set_i_val("test.static.int", 9).unwrap());
    assert_eq!(InterfaceStatic::c_def("test.static.int", "imax"), "10");
    assert_eq!(InterfaceStatic::i_def("test.static.int", "imax"), 10);
    assert_eq!(InterfaceStatic::c_def("test.static.int", "type"), "integer");
    assert_eq!(InterfaceStatic::c_def("test.static.int", "family"), "TEST");
    assert_eq!(InterfaceStatic::c_def("test.static.int", ""), "");
    assert!(
        InterfaceStatic::init(
            "TEST",
            "test.static.copy",
            InterfaceParamType::ParamMisc,
            "test.static.int"
        )
        .unwrap()
    );
    assert_eq!(InterfaceStatic::i_val("test.static.copy"), 9);
    assert!(
        !InterfaceStatic::init(
            "TEST",
            "test.static.none",
            InterfaceParamType::ParamMisc,
            "test.static.missing"
        )
        .unwrap()
    );
    assert!(InterfaceStatic::init_char("TEST", "test.static.real", b'r', "2.5").unwrap());
    assert!(InterfaceStatic::set_r_val("test.static.real", 1.5).unwrap());
    assert_eq!(InterfaceStatic::r_val("test.static.real"), 1.5);
    assert!(InterfaceStatic::init_char("TEST", "test.static.real", b'&', "rmin 0").unwrap());
    assert_eq!(
        InterfaceStatic::c_def("test.static.real", "rmin"),
        "0.000000"
    );
    assert!(!InterfaceStatic::init_char("TEST", "test.static.bad", b'x', "").unwrap());
    assert!(!InterfaceStatic::init_char("TEST", "test.static.real", b'&', "tmax 3").unwrap());
}

#[test]
fn text_values_flags_and_lists() {
    assert!(InterfaceStatic::init_char("$TEST", "test.static.text", b'p', "abc").unwrap());
    assert!(InterfaceStatic::set_c_val("test.static.text", "xyz").unwrap());
    assert_eq!(
        InterfaceStatic::c_val("test.static.text").as_bytes(),
        b"xyz"
    );
    assert!(InterfaceStatic::is_set("test.static.text", true));
    assert!(!InterfaceStatic::is_set("test.static.missing", true));
    assert!(InterfaceStatic::update("test.static.text"));
    assert!(InterfaceStatic::is_updated("test.static.text"));
    assert!(!InterfaceStatic::update("test.static.missing"));
    let dollars = InterfaceStatic::items(0, "$");
    assert!(
        dollars
            .iter()
            .any(|n| n.string().as_bytes() == b"test.static.text")
    );
    let normal = InterfaceStatic::items(0, "");
    assert!(
        !normal
            .iter()
            .any(|n| n.string().as_bytes() == b"test.static.text")
    );
    assert!(InterfaceStatic::items(100, "$TEST").is_empty());
    assert_eq!(InterfaceStatic::items(200, "$TEST").len(), 1);
    let mut map = DataMap::default();
    InterfaceStatic::fill_map(&mut map);
    assert_eq!(
        map.seek(&AsciiString::new("test.static.text"))
            .unwrap()
            .as_bytes(),
        b"xyz"
    );
    assert_eq!(
        InterfaceStatic::c_val("test.static.missing").as_bytes(),
        b""
    );
    assert_eq!(InterfaceStatic::i_val("test.static.missing"), 0);
    assert_eq!(InterfaceStatic::r_val("test.static.missing"), 0.0);
    assert!(!InterfaceStatic::set_c_val("test.static.missing", "x").unwrap());
}

#[test]
fn enumerations_by_letter() {
    assert!(InterfaceStatic::init_char("TEST", "test.static.enum", b'e', "").unwrap());
    for edit in ["enum 1", "eval Alpha", "eval Beta"] {
        assert!(InterfaceStatic::init_char("TEST", "test.static.enum", b'&', edit).unwrap());
    }
    assert_eq!(InterfaceStatic::i_def("test.static.enum", "ematch"), 0);
    assert_eq!(InterfaceStatic::i_def("test.static.enum", "evalue Beta"), 2);
    assert_eq!(
        InterfaceStatic::c_def("test.static.enum", "enum 1"),
        "Alpha"
    );
    assert_eq!(InterfaceStatic::c_def("test.static.enum", "type"), "enum");
    assert!(InterfaceStatic::set_c_val("test.static.enum", "Beta").unwrap());
    assert_eq!(InterfaceStatic::i_val("test.static.enum"), 2);
}
