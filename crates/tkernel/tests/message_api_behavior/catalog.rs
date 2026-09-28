//! Message_MsgFile over a fake file system and environment.
use super::fakes::FakeMsgFileSource;
use rustycad_tkernel::message::MsgFile;
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::Arc;

fn catalog(files: &[(&str, &[u8])], variables: &[(&str, &str)]) -> MsgFile {
    let mut source = FakeMsgFileSource::default();
    for (path, content) in files {
        source.files.insert((*path).to_owned(), content.to_vec());
    }
    for (name, value) in variables {
        source
            .variables
            .insert((*name).to_owned(), (*value).to_owned());
    }
    MsgFile::new(Arc::new(source))
}

fn message(catalog: &MsgFile, key: &str) -> String {
    catalog.msg(&AsciiString::new(key)).to_utf8()
}

#[test]
fn continuation_lines_keep_relative_indentation() {
    let text = b"! comment\n.first\n  line one\n    deeper\n\tafter tab\n.second\r\nB\r\n";
    let catalog = catalog(&[], &[]);
    assert!(catalog.load_from_string(text, -1).unwrap());
    assert_eq!(
        message(&catalog, "first"),
        "line one\n  deeper\n      after tab"
    );
    assert_eq!(message(&catalog, "second"), "B");
    assert!(catalog.has_msg(&AsciiString::new("second")));
    assert_eq!(catalog.msg_c_str(b"second\0x").to_utf8(), "B");
}

#[test]
fn lengths_empty_input_and_the_lone_dot_error() {
    let catalog = catalog(&[], &[]);
    assert!(!catalog.load_from_string(b".k\nv", 0).unwrap());
    assert!(catalog.load_from_string(b".k1\nvalue", 6).unwrap());
    assert_eq!(message(&catalog, "k1"), "va");
    let failure = catalog.load_from_string(b".\nnever", -1).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
}

#[test]
fn files_with_byte_order_marks_and_directory_lists() {
    let utf16: Vec<u8> = [0xFF, 0xFE]
        .into_iter()
        .chain(".w\nwide".encode_utf16().flat_map(u16::to_le_bytes))
        .collect();
    let big: Vec<u8> = [0xFE, 0xFF]
        .into_iter()
        .chain(".b\nbig".encode_utf16().flat_map(u16::to_be_bytes))
        .collect();
    let catalog = catalog(&[("d1/f", &utf16), ("d2/f", &big), ("empty", b"")], &[]);
    assert!(!catalog.load(Some("d1 d2 missing"), Some("f")).unwrap());
    assert_eq!(message(&catalog, "w"), "wide");
    assert_eq!(message(&catalog, "b"), "big");
    assert!(!catalog.load_file(Some("empty")).unwrap());
    assert!(!catalog.load_file(Some("")).unwrap());
    assert!(!catalog.load_file(None).unwrap());
    assert!(!catalog.load(None, Some("f")).unwrap());
}

#[test]
fn environment_selects_directory_and_language() {
    let catalog = catalog(
        &[
            ("dir/app.fr", b".e\nfrench"),
            ("dir/app.us", b".u\nenglish"),
        ],
        &[("MSG_DIR", "dir"), ("CSF_LANGUAGE", "fr")],
    );
    assert!(catalog.load_from_env(Some("MSG_DIR"), "app", None).unwrap());
    assert_eq!(message(&catalog, "e"), "french");
    assert!(
        catalog
            .load_from_env(Some("MSG_DIR"), "app", Some(".us"))
            .unwrap()
    );
    assert_eq!(message(&catalog, "u"), "english");
    assert!(
        !catalog
            .load_from_env(Some("UNSET"), "app", Some("us"))
            .unwrap()
    );
}

#[test]
fn unknown_keys_bind_the_bad_keyword_text() {
    let catalog = catalog(&[], &[]);
    assert!(!catalog.has_msg(&AsciiString::new("x")));
    assert_eq!(
        message(&catalog, "x"),
        "Unknown message invoked with the keyword x"
    );
    assert!(catalog.has_msg(&AsciiString::new("x")));
    catalog
        .load_from_string(b".Message_Msg_BadKeyword\nMissing: ", -1)
        .unwrap();
    assert_eq!(message(&catalog, "y"), "Missing: y");
}
