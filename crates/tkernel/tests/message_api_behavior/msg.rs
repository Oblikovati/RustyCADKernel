//! Message_Msg formatting; expected texts come from the OCCT probe (`msg_run`).
use super::fakes::FakeMsgFileSource;
use rustycad_tkernel::message::{Msg, MsgFile};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString, HAsciiString, HExtendedString};
use std::sync::Arc;

fn text(message: &Msg) -> String {
    message.value().to_utf8()
}

fn with(text: &str) -> Msg {
    let mut message = Msg::new();
    message.set(&ExtendedString::from_utf8(text));
    message
}

#[test]
fn arguments_fill_matching_specifications_in_order() {
    let mut message = with("%d items, %s, %.2f");
    assert!(!message.is_edited());
    message
        .arg_integer(5)
        .unwrap()
        .arg_text(b"ok\0ignored")
        .unwrap();
    message.arg_real(2.25).unwrap();
    assert_eq!(text(&message), "5 items, ok, 2.25");
    assert_eq!(message.original().to_utf8(), "%d items, %s, %.2f");
    assert!(message.is_edited());
    // An argument without a pending specification of its kind changes nothing.
    message.arg_integer(9).unwrap();
    assert_eq!(text(&message), "5 items, ok, 2.25");
}

#[test]
fn string_overloads_share_the_string_specification() {
    let mut message = with("[%-6s][%s][%s][%s]");
    message.arg_ascii(&AsciiString::new("a")).unwrap();
    message.arg_h_ascii(&HAsciiString::new("b")).unwrap();
    message
        .arg_extended(&ExtendedString::from_utf8("\u{20ac}"))
        .unwrap();
    message
        .arg_h_extended(&HExtendedString::from_utf16(&[0x63]))
        .unwrap();
    assert_eq!(text(&message), "[a     ][b][\u{20ac}][c]");
}

#[test]
fn percent_pairs_collapse_and_still_open_a_specification() {
    let mut message = with("100%%d");
    assert_eq!(text(&message), "100%d");
    message.arg_integer(7).unwrap();
    assert_eq!(text(&message), "1007");
}

#[test]
fn glibc_layouts_are_reproduced() {
    let mut message = with("<%#g>");
    message.arg_real(999_999.875).unwrap();
    assert_eq!(text(&message), "<1.e+06>");
    let mut message = with("%ld");
    message.arg_integer(-1).unwrap();
    assert_eq!(text(&message), "4294967295");
}

#[test]
fn get_fills_unknown_and_keeps_stale_positions() {
    let mut message = with("a %d b %s");
    assert_eq!(message.get().unwrap().to_utf8(), "a UNKNOWN b UNKNOWN");
    let second = message.get().unwrap().to_utf8();
    assert_eq!(second, "a UNKNOWNKNOUNKNOWN b UNKNOWN");
    let mut message = with("%-100.50dx");
    assert_eq!(message.get().unwrap().to_utf8(), "UNKNOWNx");
    let failure = message.get().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    // The native loop edits in place: the failed Get leaves the truncated body.
    assert_eq!(text(&message), "");
}

#[test]
fn messages_come_from_a_catalog_or_c_text() {
    let catalog = MsgFile::new(Arc::new(FakeMsgFileSource::default()));
    catalog.add_msg(
        &AsciiString::new("greet"),
        &ExtendedString::from_utf8("Hello %s"),
    );
    let mut message = Msg::from_key_in(&catalog, &AsciiString::new("greet"));
    message.arg_text(b"you").unwrap();
    assert_eq!(text(&message), "Hello you");
    let unknown = Msg::from_key_in(&catalog, &AsciiString::new("nope"));
    assert_eq!(
        text(&unknown),
        "Unknown message invoked with the keyword nope"
    );
    let mut c_text = Msg::new();
    c_text.set_text("caf\u{e9} %d".as_bytes());
    assert_eq!(text(&c_text), "caf\u{e9} %d");
    let copy = c_text.clone();
    assert_eq!(copy.value(), c_text.value());
}
