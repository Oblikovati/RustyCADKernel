//! Unit tests of `MoniTool_TypedValue`: specifications, setters, printing and library.
use super::{MoniToolTypedValue, MoniToolValueType};
use rustycad_tkernel::{
    standard::{Handle, RuntimeTyped, StandardTransient},
    tcollection::HAsciiString,
};

fn text(value: &str) -> Handle<HAsciiString> {
    Handle::new(HAsciiString::new(value))
}

fn printed(value: &MoniToolTypedValue) -> String {
    let mut out = Vec::new();
    value.print(&mut out).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn unsupported_type_is_a_construction_error() {
    let error = MoniToolTypedValue::new("x", MoniToolValueType::ValueHexa, "")
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let init = MoniToolTypedValue::new("x", MoniToolValueType::ValueInteger, "12").unwrap();
    assert_eq!(init.integer_value(), 12);
    let refused = MoniToolTypedValue::new("x", MoniToolValueType::ValueInteger, "abc").unwrap();
    assert!(!refused.is_set_value());
}

#[test]
fn integer_and_real_limits() {
    let value = MoniToolTypedValue::new("i", MoniToolValueType::ValueInteger, "").unwrap();
    assert_eq!(value.integer_limit(true), (false, i32::MAX));
    assert!(value.add_def("imax 10").unwrap());
    assert!(value.add_def("imin -2").unwrap());
    assert!(value.satisfies(&text("10")));
    assert!(!value.satisfies(&text("11")));
    assert!(!value.satisfies(&text("1.5")));
    assert!(value.satisfies(&text("")));
    assert_eq!(value.definition().as_bytes(), b"Integer  >= -2  <= 10");
    assert!(value.set_real_limit(true, 1.0).is_err());
    let real = MoniToolTypedValue::new("r", MoniToolValueType::ValueReal, "").unwrap();
    assert!(real.add_def("rmin 0.5").unwrap());
    assert!(real.add_def("unit mm").unwrap());
    assert_eq!(real.definition().as_bytes(), b"Real  >= 0.500000  Unit:mm");
    assert!(real.set_real_value(2.0).unwrap());
    assert_eq!(real.real_value(), 2.0);
    assert!(!real.set_real_value(0.25).unwrap());
    assert!(!real.add_def("nothing").unwrap());
}

#[test]
fn enumerations() {
    let value = MoniToolTypedValue::new("e", MoniToolValueType::ValueEnum, "").unwrap();
    value.start_enum(1, true).unwrap();
    value.add_enum(&["One", "Two", "", "Three"]).unwrap();
    value.add_enum_value("Deux", 2).unwrap();
    assert_eq!(value.enum_def(), Some((1, 3, true)));
    assert_eq!(value.enum_case("Two"), 2);
    assert_eq!(value.enum_case("Deux"), 2);
    assert_eq!(value.enum_case("7"), 7);
    assert_eq!(value.enum_case("x"), 0);
    assert_eq!(value.enum_val(3).as_bytes(), b"Three");
    assert_eq!(value.enum_val(9).as_bytes(), b"");
    assert!(value.set_c_string_value("Deux").unwrap());
    assert_eq!(value.c_string_value().as_bytes(), b"Two");
    assert_eq!(value.integer_value(), 2);
    assert!(value.set_integer_value(3).unwrap());
    assert_eq!(value.c_string_value().as_bytes(), b"Three");
    assert!(!value.set_c_string_value("Four").unwrap());
    let coded = value.interpret(&text("One"), false);
    assert_eq!(coded.string().as_bytes(), b"1");
    let error = value.add_enum_value("Zero", 0).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

#[test]
fn text_objects_and_clear() {
    let value = MoniToolTypedValue::new("t", MoniToolValueType::ValueText, "").unwrap();
    value.set_max_length(3);
    assert!(!value.set_c_string_value("long").unwrap());
    assert!(value.set_c_string_value("abc").unwrap());
    let shared = value.h_string_value();
    assert!(value.set_c_string_value("xy").unwrap());
    assert_eq!(shared.string().as_bytes(), b"xy");
    value.clear_value();
    assert!(!value.is_set_value());
    let error = value.set_c_string_value("a").err().unwrap();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    assert!(value.set_h_string_value(&text("b")));
    assert_eq!(value.c_string_value().as_bytes(), b"b");
    let object = MoniToolTypedValue::new("o", MoniToolValueType::ValueIdent, "").unwrap();
    assert!(!value.set_object_value(&Handle::default()));
    object
        .set_object_type(&HAsciiString::static_type_handle())
        .unwrap();
    assert!(!object.set_object_value(&Handle::new(StandardTransient::default())));
    assert!(object.set_object_value(&text("s").cast()));
    assert_eq!(object.object_type_name(), "TCollection_HAsciiString");
    assert_eq!(
        object.definition().as_bytes(),
        b"Object(Entity):TCollection_HAsciiString"
    );
}

#[test]
fn print_shows_value_and_interpretation() {
    let value = MoniToolTypedValue::new("flag", MoniToolValueType::ValueEnum, "").unwrap();
    value.add_def("ematch 0").unwrap();
    value.add_enum(&["Off", "On"]).unwrap();
    value.set_label("A flag");
    assert!(value.set_c_string_value("On").unwrap());
    let expected =
        "--- Typed Value : flag  Label : A flag\n--- Type : Enum [in 0-1] 0:Off 1:On , alpha: ";
    let text = printed(&value);
    assert!(text.starts_with(expected), "{text}");
    assert!(
        text.ends_with("--- Value : On  (Native:On  Coded:1)\n"),
        "{text}"
    );
    let unset = MoniToolTypedValue::new("n", MoniToolValueType::ValueInteger, "").unwrap();
    assert!(printed(&unset).contains("--- Value : (not set)\n"));
}

#[test]
fn copies_and_library() {
    let value = MoniToolTypedValue::new("c", MoniToolValueType::ValueInteger, "4").unwrap();
    value.set_integer_limit(true, 9).unwrap();
    let copy = MoniToolTypedValue::copied(&value);
    assert_eq!(copy.integer_limit(true), (true, 9));
    assert_eq!(copy.integer_limit(false), (false, i32::MIN));
    assert!(copy.h_string_value() != value.h_string_value());
    assert_eq!(copy.definition().as_bytes(), b"Integer  <= 9");
    let boolean = MoniToolTypedValue::from_lib("Boolean");
    assert_eq!(boolean.enum_case("True"), 1);
    assert!(MoniToolTypedValue::lib("Boolean") != boolean);
    assert!(MoniToolTypedValue::lib("missing").is_null());
    assert!(MoniToolTypedValue::add_lib(&Handle::new(value), "my def"));
    assert!(
        MoniToolTypedValue::lib_list()
            .iter()
            .any(|n| n.as_bytes() == b"c")
    );
    assert_eq!(
        MoniToolTypedValue::lib("c").definition().as_bytes(),
        b"my def"
    );
    assert!(MoniToolTypedValue::static_value("c").is_null());
    assert!(MoniToolTypedValue::stats().is_empty());
}
