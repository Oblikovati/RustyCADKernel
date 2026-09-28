//! TCollection_HExtendedString members, handle sharing and the handle hash policy.
use super::{class, string, units};
use rustycad_tkernel::{
    ncollection::HashPolicy,
    standard::{Handle, RuntimeTyped},
    tcollection::{ExtendedString, HAsciiString, HExtendedString, HExtendedStringHasher},
};

fn shared(text: &str) -> Handle<HExtendedString> {
    Handle::new(HExtendedString::from(string(text)))
}

#[test]
fn constructors_wrap_the_extended_string_constructors() {
    assert!(HExtendedString::default().is_empty());
    assert_eq!(
        *HExtendedString::from_c_string(b"\xC3\xA9").string(),
        string("\u{C3}\u{A9}")
    );
    assert_eq!(
        *HExtendedString::from_utf16(&[65, 0, 66]).string(),
        string("A")
    );
    assert!(HExtendedString::from_char(0).is_empty());
    assert_eq!(HExtendedString::filled(2, 45).unwrap().length(), 2);
    assert_eq!(
        class(HExtendedString::filled(-1, 45).map(|_| ())),
        "Standard_OutOfRange"
    );
    assert!(HExtendedString::from_h_ascii_string(&HAsciiString::default()).is_empty());
    assert_eq!(*HExtendedString::from(&string("ab")).string(), string("ab"));
    let source = shared("ab");
    let copy = HExtendedString::from_h_extended_string(&source);
    source.assign_cat(&source);
    assert_eq!((copy.length(), source.length()), (2, 4));
}
#[test]
fn edits_are_seen_through_every_handle() {
    let first = shared("hy not ?");
    let second = first.clone();
    first.insert_char(1, 87).unwrap();
    assert_eq!(*second.string(), string("Why not ?"));
    second.insert(4, &shared("!")).unwrap();
    second.set_value(1, 119).unwrap();
    second.set_value_string(9, &shared("?!")).unwrap();
    assert_eq!(*first.string(), string("why! not?!"));
    first.remove(4, 1).unwrap();
    first.remove_all(33);
    first.change_all(111, 48);
    assert_eq!(first.string().to_utf8(), "why n0t?");
    assert_eq!(class(first.remove(0, 1)), "Standard_OutOfRange");
    first.trunc(3).unwrap();
    assert_eq!(first.to_ext_string(), [119, 104, 121, 0]);
    first.clear();
    assert!(second.is_empty());
}
#[test]
fn queries_and_results_in_new_objects() {
    let text = shared("abcab");
    let other = shared("ab");
    assert!(text.is_greater(&other) && other.is_less(&text) && text.is_ascii());
    assert_eq!((text.search(&other), text.search_from_end(&other)), (1, 4));
    let joined = text.cat(&other);
    assert_eq!((joined.length(), text.length()), (7, 5));
    assert_eq!(*text.token(&units("c"), 2).string(), string("ab"));
    let tail = text.split(3).unwrap();
    assert_eq!(
        (tail.string().clone(), text.value(3).unwrap()),
        (string("ab"), 99)
    );
    assert_eq!(class(text.split(3).map(|_| ())), "Standard_OutOfRange");
    assert_eq!(class(text.value(4)), "Standard_OutOfRange");
    assert!(shared("x").is_same_state(&shared("x")));
    let mut printed = Vec::new();
    shared("é").print(&mut printed).unwrap();
    assert_eq!(printed, "begin class HExtendedString \né".as_bytes());
}
#[test]
fn transient_type_and_handle_policy() {
    assert_eq!(
        HExtendedString::static_type().name(),
        "TCollection_HExtendedString"
    );
    let (first, twin, null) = (
        shared("k"),
        shared("k"),
        Handle::<HExtendedString>::default(),
    );
    let policy = HExtendedStringHasher;
    assert_eq!(
        policy.hash(&first),
        ExtendedString::from_char(107).hash_code()
    );
    assert_eq!(policy.hash(&null), 0);
    assert!(policy.equal(&first, &twin) && policy.equal(&null, &null));
    assert!(!policy.equal(&first, &null) && !policy.equal(&null, &twin));
}
