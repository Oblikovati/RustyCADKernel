//! Unit tests of `Interface_Check`.
use super::{InterfaceCheck, InterfaceCheckStatus};
use rustycad_tkernel::{
    message::Msg,
    standard::{Handle, StandardTransient},
    tcollection::{AsciiString, HAsciiString},
};

fn text(value: &str) -> Handle<HAsciiString> {
    Handle::new(HAsciiString::new(value))
}

fn finals(list: Vec<Handle<HAsciiString>>) -> Vec<String> {
    list.iter()
        .map(|h| String::from_utf8_lossy(h.string().as_bytes()).into_owned())
        .collect()
}

#[test]
fn messages_and_status() {
    let check = InterfaceCheck::new();
    assert_eq!(check.status(), InterfaceCheckStatus::CheckOK);
    check.add_warning_text("w1", "");
    check.add_warning_text("", "ignored");
    assert_eq!(check.status(), InterfaceCheckStatus::CheckWarning);
    assert!(check.complies(InterfaceCheckStatus::CheckMessage));
    assert!(check.complies(InterfaceCheckStatus::CheckNoFail));
    check.add_fail(&text("f1"));
    assert!(check.has_failed() && check.complies(InterfaceCheckStatus::CheckFail));
    assert_eq!(check.c_warning(1, false).unwrap().as_bytes(), b"w1");
    assert_eq!(
        check.fail(2, true).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        check.info_msg(1, true).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    let mut msg = Msg::new();
    msg.set_text(b"info");
    check.send_msg(&msg);
    check.send_fail(&msg);
    check.send_warning(&msg);
    assert_eq!(
        (check.nb_fails(), check.nb_warnings(), check.nb_info_msgs()),
        (2, 2, 1)
    );
    assert_eq!(check.c_info_msg(1, false).unwrap().as_bytes(), b"info");
    assert_eq!(finals(check.infos_for_test()), vec!["info"]);
}

impl InterfaceCheck {
    fn infos_for_test(&self) -> Vec<Handle<HAsciiString>> {
        self.info_msgs(true)
    }
}

#[test]
fn entity_and_clear() {
    let check = InterfaceCheck::with_entity(&Handle::new(StandardTransient::default()));
    assert!(check.has_entity());
    let other = Handle::new(StandardTransient::default());
    check.get_entity(&other);
    assert!(check.entity() != other);
    check.add_fail_text("x", "y");
    check.clear_fails();
    assert!(!check.has_failed());
    check.add_warning(&text("w"));
    check.clear_warnings();
    check.clear_info_msgs();
    check.clear();
    assert!(!check.has_entity());
    check.set_entity(&other);
    assert!(check.entity() == other);
}

#[test]
fn complies_and_remove_scan_the_warnings() {
    let check = InterfaceCheck::new();
    check.add_warning(&text("abc"));
    check.add_fail(&text("zzz"));
    let abc = text("abc");
    assert!(
        check
            .complies_message(&abc, 0, InterfaceCheckStatus::CheckWarning)
            .unwrap()
    );
    // The fail criterion scans the warnings natively.
    assert!(
        check
            .complies_message(&abc, 0, InterfaceCheckStatus::CheckFail)
            .unwrap()
    );
    assert!(
        !check
            .complies_message(&text("zzz"), 0, InterfaceCheckStatus::CheckFail)
            .unwrap()
    );
    assert!(
        !check
            .complies_message(&abc, 1, InterfaceCheckStatus::CheckAny)
            .unwrap()
    );
    assert!(
        check
            .remove(&abc, 0, InterfaceCheckStatus::CheckFail)
            .unwrap()
    );
    assert_eq!(check.nb_fails(), 0);
    assert!(check.has_failed());
    assert!(
        check
            .remove(&abc, 0, InterfaceCheckStatus::CheckWarning)
            .unwrap()
    );
    assert_eq!(check.nb_warnings(), 0);
}

#[test]
fn mend_prefixes_and_codes() {
    let check = InterfaceCheck::new();
    check.add_fail_text("a", "");
    check.add_fail_text("b", "B");
    assert!(check.mend(Some("FM"), 1).unwrap());
    assert_eq!(finals(check.fails(true)), vec!["b", "Mended : a"]);
    assert!(check.mend(Some("Fix"), 0).unwrap());
    assert_eq!(
        finals(check.fails(false)),
        vec!["Fix : Mended : a", "Fix : B"]
    );
    assert!(!check.mend(Some("CF"), 5).unwrap());
    assert!(check.mend(Some("CF"), 1).unwrap());
    assert_eq!(check.nb_fails(), 1);
    check.add_warning_text("w", "");
    assert!(check.mend(Some("CW"), 0).unwrap());
    assert!(!check.has_warnings());
    assert!(check.mend(Some("CA"), 0).unwrap());
    assert!(!check.has_failed());
}

#[test]
fn copies_between_checks_and_print() {
    let source = InterfaceCheck::new();
    source.add_fail_text("f", "F");
    source.add_warning_text("w", "");
    let target = InterfaceCheck::new();
    target.get_messages(&source);
    assert_eq!((target.nb_fails(), target.nb_warnings()), (1, 1));
    let warned = InterfaceCheck::new();
    warned.get_as_warning(&source, true);
    assert_eq!(finals(warned.warnings(false)), vec!["F", "w"]);
    let empty = InterfaceCheck::new();
    empty.get_as_warning(&InterfaceCheck::new(), false);
    assert!(empty.has_warnings());
    let mut out = Vec::new();
    target.print(&mut out, 2, -1).unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), "F\nw\n");
    let mut none = Vec::new();
    target.print(&mut none, 0, 1).unwrap();
    assert!(none.is_empty());
    target.trace(-1, 1);
    assert_eq!(target.c_fail(1, true).unwrap(), AsciiString::new("f"));
}
