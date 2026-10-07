//! `DE_ValidationUtils` through in-memory streams and temporary files; the expected values
//! are the native ones of tools/oracle/tkde_configuration_probe.cxx (`content.*`, `valid.*`).
mod support;

use rustycad_tkde::de::{
    DeProviderReadStreamList, DeProviderReadStreamNode, DeProviderWriteStreamList,
    DeProviderWriteStreamNode, DeValidationUtils,
};
use rustycad_tkernel::{
    ncollection::Buffer,
    osd::{IStream, StreamAccess},
    standard::Handle,
    tcollection::AsciiString,
};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use support::memory_stream::{memory_istream, memory_ostream};

fn text(value: &str) -> AsciiString {
    AsciiString::new(value)
}

#[test]
fn a_full_content_buffer_ends_with_a_nul_and_the_stream_is_rewound() {
    let mut stream = memory_istream(&[b'a'; 3000]);
    stream.seekg(5, rustycad_tkernel::osd::SeekDir::Beg);
    let mut buffer: Handle<Buffer> = Handle::default();
    assert_eq!(
        DeValidationUtils::create_content_buffer(&mut stream, &mut buffer),
        Ok(true)
    );
    let bytes = buffer.get().expect("a buffer").as_slice();
    assert_eq!((bytes.len(), bytes[2047], bytes[2046]), (2048, 0, b'a'));
    assert_eq!(stream.tellg(), 5);
}

#[test]
fn a_short_content_buffer_is_terminated_after_the_read_bytes() {
    let mut stream = memory_istream(b"abc");
    let mut buffer: Handle<Buffer> = Handle::default();
    assert_eq!(
        DeValidationUtils::create_content_buffer(&mut stream, &mut buffer),
        Ok(true)
    );
    assert_eq!(buffer.get().expect("a buffer").as_slice()[..4], *b"abc\0");
    assert!(stream.good());
    assert_eq!(stream.tellg(), 0);
}

#[test]
fn a_missing_file_gives_no_content_buffer() {
    let mut buffer: Handle<Buffer> = Handle::default();
    let created =
        DeValidationUtils::create_content_buffer_from_path(&text("/no/such/file.stp"), &mut buffer);
    assert_eq!(created, Ok(false));
    assert!(buffer.is_null());
}

#[test]
fn file_checks_follow_the_native_results() {
    let directory = std::env::temp_dir().join(format!("tkde-valid-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("a temporary directory");
    let readable = directory.join("readable.txt");
    std::fs::write(&readable, "x").expect("a readable file");
    let readable = text(readable.to_str().expect("a UTF-8 path"));
    let context = text("ctx");
    assert!(!DeValidationUtils::validate_file_for_reading(
        &text(""),
        &context,
        false
    ));
    assert!(!DeValidationUtils::validate_file_for_reading(
        &text("/no/such/file"),
        &context,
        true
    ));
    assert!(DeValidationUtils::validate_file_for_reading(
        &readable, &context, false
    ));
    assert!(!DeValidationUtils::validate_file_for_writing(
        &text(""),
        &context,
        false
    ));
    assert!(!DeValidationUtils::validate_file_for_writing(
        &text("/no/such/dir/file"),
        &context,
        false
    ));
    assert!(DeValidationUtils::validate_file_for_writing(
        &readable, &context, false
    ));
    std::fs::remove_dir_all(&directory).expect("cleanup");
}

#[test]
fn stream_lists_need_a_first_stream_that_has_not_failed() {
    let context = text("ctx");
    let mut good = memory_istream(b"x");
    let mut bad = memory_istream(b"");
    let mut byte = [0_u8; 1];
    bad.read(&mut byte);
    assert!(bad.fail());
    let empty = DeProviderReadStreamList::default();
    assert!(!DeValidationUtils::validate_read_stream_list(
        &empty, &context, true
    ));
    let mut list = DeProviderReadStreamList::default();
    list.append(DeProviderReadStreamNode::new(&text("a"), &mut good));
    assert!(DeValidationUtils::validate_read_stream_list(
        &list, &context, true
    ));
    let mut failed = DeProviderReadStreamList::default();
    failed.append(DeProviderReadStreamNode::new(&text(""), &mut bad));
    assert!(!DeValidationUtils::validate_read_stream_list(
        &failed, &context, true
    ));
}

#[test]
fn write_stream_lists_and_documents_are_checked() {
    let context = text("ctx");
    let mut out = memory_ostream();
    let mut second = memory_ostream();
    assert!(!DeValidationUtils::validate_write_stream_list(
        &DeProviderWriteStreamList::default(),
        &context,
        false
    ));
    let mut list = DeProviderWriteStreamList::default();
    list.append(DeProviderWriteStreamNode::new(&text("w"), &mut out));
    list.append(DeProviderWriteStreamNode::new(&text("x"), &mut second));
    assert!(DeValidationUtils::validate_write_stream_list(
        &list, &context, true
    ));
    let document: Handle<TDocStdDocument> = Handle::default();
    assert!(!DeValidationUtils::validate_document(
        &document, &context, true
    ));
    assert!(DeValidationUtils::warn_length_unit_not_supported(
        2.0, &context, true
    ));
}
