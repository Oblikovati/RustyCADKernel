//! FSD_File and FSD_CmpFile.
use super::fakes::{RefusingFileSystem, with_file};
use rustycad_tkernel::fsd::{CmpFile, File, MemoryFileSystem};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::storage::{
    BaseDriver, DataSections, DriverPrimitives, HeaderSections, OpenMode, StorageError,
    TableSections,
};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use std::rc::Rc;

fn name() -> AsciiString {
    AsciiString::new("doc")
}

fn written(write: impl FnOnce(&mut File)) -> Vec<u8> {
    let files = MemoryFileSystem::default();
    let mut file = File::new(Rc::new(files.clone()));
    assert_eq!(
        file.open(&name(), OpenMode::Write).unwrap(),
        StorageError::Ok
    );
    write(&mut file);
    file.close().unwrap();
    files.contents(&name()).unwrap()
}

#[test]
fn info_section_layout_and_scalars() {
    let bytes = written(|file| {
        file.begin_write_info_section().unwrap();
        let mut info = Sequence::default();
        info.append(AsciiString::new("line"));
        file.write_info(
            3,
            &"1.3".into(),
            &"d".into(),
            &"S".into(),
            &"".into(),
            &ExtendedString::new("AB"),
            &"v".into(),
            &ExtendedString::default(),
            &info,
        )
        .unwrap();
        file.end_write_info_section().unwrap();
        file.put_real(0.1).unwrap();
        file.put_character(0xB0).unwrap();
        file.put_ext_character(0xFFFF).unwrap();
        file.put_boolean(true).unwrap();
        file.put_short_real(0.5).unwrap();
    });
    let expected = b"FSDFILE\nBEGIN_INFO_SECTION\n3\n1.3\nd\nS\n\n\0A\0B\0\nv\n\0\n1\nline\n\
END_INFO_SECTION\n0.10000000000000001 65456 -1 1 0.5 ";
    assert_eq!(bytes, expected);
}

#[test]
fn extended_lines_read_back_as_decimal_digits() {
    // OCC regression kept on purpose: `buffer += (char16_t)i` binds to `operator+=(int)`.
    let file = b"FSDFILE\nBEGIN_INFO_SECTION\n0\n\n\n\n\n\0A\0B\0\n\n\0\n0\nEND_INFO_SECTION";
    let (_, files) = with_file("doc", file);
    let mut driver = File::new(files);
    driver.open(&name(), OpenMode::Read).unwrap();
    assert_eq!(driver.begin_read_info_section().unwrap(), StorageError::Ok);
    let (mut count, mut app) = (0, ExtendedString::default());
    let mut texts: [AsciiString; 5] = Default::default();
    let (mut kind, mut info) = (ExtendedString::default(), Sequence::default());
    let [a, b, c, d, e] = &mut texts;
    driver
        .read_info(&mut count, a, b, c, d, &mut app, e, &mut kind, &mut info)
        .unwrap();
    assert_eq!(app.to_utf8(), "6566");
    assert_eq!(
        driver.end_read_info_section().unwrap(),
        StorageError::SectionNotFound,
        "the tag is on the last line, so reading it reaches end of file"
    );
}

#[test]
fn compatible_files_write_counts_and_accept_carriage_returns() {
    let files = MemoryFileSystem::default();
    let mut driver = CmpFile::new(Rc::new(files.clone()));
    driver.open(&name(), OpenMode::Write).unwrap();
    driver.begin_write_info_section().unwrap();
    driver
        .write_comment(&{
            let mut comments = Sequence::default();
            comments.append(ExtendedString::new("Hi"));
            comments
        })
        .unwrap();
    driver.write_persistent_object_header(4, 2).unwrap();
    driver.close().unwrap();
    assert_eq!(
        files.contents(&name()).unwrap(),
        b"CMPFILE\nBEGIN_INFO_SECTION\n1\n2 72 105 \n\n#4%2 "
    );
    let (_, input) = with_file("doc", b" #7 % 3  \r\nrest");
    let mut reader = CmpFile::new(input);
    reader.open(&name(), OpenMode::Read).unwrap();
    let (mut reference, mut kind) = (0, 0);
    reader
        .read_persistent_object_header(&mut reference, &mut kind)
        .unwrap();
    reader.end_read_persistent_object_data().unwrap();
    assert_eq!((reference, kind), (7, 3));
    assert_eq!(CmpFile::magic_number(), "CMPFILE");
}

#[test]
fn object_framing_and_format_errors() {
    let (_, input) = with_file("doc", b"#1=%2( 5 ) )\n#x");
    let mut driver = File::new(input);
    driver.open(&name(), OpenMode::Read).unwrap();
    let (mut reference, mut kind, mut value) = (0, 0, 0);
    driver
        .read_persistent_object_header(&mut reference, &mut kind)
        .unwrap();
    driver.begin_read_persistent_object_data().unwrap();
    driver.get_integer(&mut value).unwrap();
    driver.end_read_object_data().unwrap();
    driver.end_read_persistent_object_data().unwrap();
    assert_eq!((reference, kind, value), (1, 2, 5));
    let failure = driver
        .read_persistent_object_header(&mut reference, &mut kind)
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Storage_StreamTypeMismatchError");
}

#[test]
fn getters_keep_native_conversions() {
    let long = "1".repeat(120);
    let text = format!("-80 70000 abc 1.5x {long} 7");
    let (_, input) = with_file("doc", text.as_bytes());
    let mut driver = File::new(input);
    driver.open(&name(), OpenMode::Read).unwrap();
    let (mut byte, mut real, mut value) = (0_u8, 0.0, 0);
    driver.get_character(&mut byte).unwrap();
    assert_eq!(byte, 0xB0);
    driver.get_character(&mut byte).unwrap();
    assert_eq!(
        byte, 0xFF,
        "an overflowing value is kept after clearing the stream"
    );
    assert!(driver.get_integer(&mut value).is_err());
    driver.stream().clear(0);
    driver.get_real(&mut real).unwrap_err();
    driver.get_real(&mut real).unwrap_err();
    assert_eq!(real, 1.5, "CStringToReal stores its parse before failing");
    driver.get_real(&mut real).unwrap();
    assert_eq!(
        real, 1.1111111111111112e98,
        "only 99 digits fit the native buffer"
    );
    driver.get_real(&mut real).unwrap();
    assert_eq!(real, 1.1111111111111111e20);
}

#[test]
fn opening_rules_and_tell() {
    let files = MemoryFileSystem::default();
    let mut driver = File::new(Rc::new(files.clone()));
    assert_eq!(
        driver.open(&name(), OpenMode::None).unwrap(),
        StorageError::Ok
    );
    assert_eq!(driver.close().unwrap(), StorageError::NotOpen);
    assert_eq!(
        driver.open(&name(), OpenMode::Read).unwrap(),
        StorageError::OpenError
    );
    let mut writer = File::new(Rc::new(files.clone()));
    writer.open(&name(), OpenMode::Write).unwrap();
    assert_eq!(
        writer.open(&name(), OpenMode::Write).unwrap(),
        StorageError::AlreadyOpen
    );
    writer.put_integer(12).unwrap();
    assert_eq!(writer.tell().unwrap(), 3);
    writer.destroy();
    assert_eq!(writer.open_mode(), OpenMode::None);
    assert_eq!(
        File::is_good_file_type(Rc::new(files.clone()), &name()).unwrap(),
        StorageError::FormatError
    );
    files.insert(&name(), b"FSDFILE\n".to_vec());
    assert_eq!(
        File::is_good_file_type(Rc::new(files.clone()), &name()).unwrap(),
        StorageError::Ok
    );
    assert_eq!(
        File::is_good_file_type(Rc::new(files), &"none".into()).unwrap(),
        StorageError::OpenError
    );
}

#[test]
fn refused_bytes_raise_write_errors() {
    let mut driver = File::new(Rc::new(RefusingFileSystem));
    driver.open(&name(), OpenMode::Write).unwrap();
    let failure = driver.begin_write_type_section().unwrap_err();
    assert_eq!(failure.exception_type(), "Storage_StreamWriteError");
    let (_, input) = with_file("doc", b"");
    let mut reader = File::new(input);
    reader.open(&name(), OpenMode::Read).unwrap();
    assert!(
        reader.put_reference(1).is_err(),
        "a read-only stream cannot write"
    );
}

#[test]
fn tables_round_trip() {
    let bytes = written(|file| {
        file.begin_write_type_section().unwrap();
        file.set_type_section_size(1).unwrap();
        file.write_type_informations(1, &"PColStd".into()).unwrap();
        file.end_write_type_section().unwrap();
        file.begin_write_root_section().unwrap();
        file.set_root_section_size(1).unwrap();
        file.write_root(&"doc".into(), 1, &"PDoc".into()).unwrap();
        file.end_write_root_section().unwrap();
        file.begin_write_ref_section().unwrap();
        file.set_ref_section_size(1).unwrap();
        file.write_reference_type(1, 1).unwrap();
        file.end_write_ref_section().unwrap();
    });
    let (_, input) = with_file("doc", &bytes);
    let mut driver = File::new(input);
    driver.open(&name(), OpenMode::Read).unwrap();
    assert_eq!(driver.begin_read_type_section().unwrap(), StorageError::Ok);
    assert_eq!(driver.type_section_size().unwrap(), 1);
    let (mut number, mut text, mut kind) = (0, AsciiString::default(), AsciiString::default());
    driver
        .read_type_informations(&mut number, &mut text)
        .unwrap();
    assert_eq!((number, text.as_bytes()), (1, b"PColStd".as_slice()));
    driver.end_read_type_section().unwrap();
    driver.begin_read_root_section().unwrap();
    assert_eq!(driver.root_section_size().unwrap(), 1);
    driver.read_root(&mut text, &mut number, &mut kind).unwrap();
    assert_eq!(kind.as_bytes(), b"PDoc");
    driver.end_read_root_section().unwrap();
    driver.begin_read_ref_section().unwrap();
    assert_eq!(driver.ref_section_size().unwrap(), 1);
    let mut second = 0;
    driver
        .read_reference_type(&mut number, &mut second)
        .unwrap();
    assert_eq!(driver.end_read_ref_section().unwrap(), StorageError::Ok);
    assert!(
        !driver.is_end().unwrap(),
        "the tag line ends with a newline"
    );
}
