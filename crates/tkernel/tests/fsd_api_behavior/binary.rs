//! FSD_BinaryFile, its stream overloads and ReadCompleteInfo.
use super::fakes::{RefusingFileSystem, with_file};
use rustycad_tkernel::fsd::{BinaryFile, FileHeader, MemoryFileSystem};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::storage::{
    BaseDriver, Data, DataSections, DriverPrimitives, HeaderData, HeaderSections, IoStream,
    OpenMode, StorageError, TableSections,
};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use std::rc::Rc;

fn name() -> AsciiString {
    AsciiString::new("doc")
}

fn int(value: i32) -> Vec<u8> {
    value.to_ne_bytes().to_vec()
}

#[test]
fn sections_record_offsets_and_the_header_is_rewritten_at_the_end() {
    let files = MemoryFileSystem::default();
    let mut driver = BinaryFile::new(Rc::new(files.clone()));
    driver.open(&name(), OpenMode::Write).unwrap();
    driver.begin_write_info_section().unwrap();
    driver
        .write_info(
            1,
            &"1.3".into(),
            &"".into(),
            &"S".into(),
            &"".into(),
            &ExtendedString::new("A"),
            &"".into(),
            &ExtendedString::default(),
            &Sequence::default(),
        )
        .unwrap();
    driver.end_write_info_section().unwrap();
    for step in [
        BinaryFile::begin_write_comment_section,
        BinaryFile::end_write_comment_section,
        BinaryFile::begin_write_type_section,
        BinaryFile::end_write_type_section,
        BinaryFile::begin_write_root_section,
        BinaryFile::end_write_root_section,
        BinaryFile::begin_write_ref_section,
        BinaryFile::end_write_ref_section,
        BinaryFile::begin_write_data_section,
        BinaryFile::end_write_data_section,
    ] {
        step(&mut driver).unwrap();
    }
    let header = driver.file_header();
    driver.close().unwrap();
    let bytes = files.contents(&name()).unwrap();
    assert_eq!(&bytes[..7], b"BINFILE");
    assert_eq!(header.test_indian, i32::from_ne_bytes([1, 2, 3, 4]));
    assert_eq!(header.begin_info, 7);
    assert_eq!(header.end_data as usize, bytes.len());
    let mut stored = FileHeader::default();
    let mut stream = IoStream::from_bytes(bytes[7..].to_vec());
    BinaryFile::read_header_stream(&mut stream, &mut stored).unwrap();
    assert_eq!(stored, header);
}

#[test]
fn short_reads_keep_the_bytes_they_got() {
    let (_, input) = with_file("doc", &[0xAA, 0xBB]);
    let mut driver = BinaryFile::new(input);
    driver.open(&name(), OpenMode::Read).unwrap();
    let mut value = 0x1111_1111;
    let failure = driver.get_integer(&mut value).unwrap_err();
    assert_eq!(failure.exception_type(), "Storage_StreamTypeMismatchError");
    let mut expected = 0x1111_1111_i32.to_ne_bytes();
    expected[..2].copy_from_slice(&[0xAA, 0xBB]);
    assert_eq!(value, i32::from_ne_bytes(expected));
    assert!(driver.is_end().unwrap());
    assert_eq!(
        driver.end_read_info_section().unwrap(),
        StorageError::SectionNotFound,
        "the unset header offset is -1"
    );
}

#[test]
fn strings_stop_at_nul_and_short_strings_raise_write_errors() {
    let mut bytes = int(3);
    bytes.extend_from_slice(b"a\0b");
    bytes.extend(int(9));
    let (_, input) = with_file("doc", &bytes);
    let mut driver = BinaryFile::new(input);
    driver.open(&name(), OpenMode::Read).unwrap();
    let (mut number, mut text) = (0, AsciiString::default());
    driver
        .read_type_informations(&mut number, &mut text)
        .unwrap_err();
    driver.close().unwrap();
    let mut bytes = int(1);
    bytes.extend(int(3));
    bytes.extend_from_slice(b"a\0b");
    let (_, input) = with_file("doc", &bytes);
    let mut driver = BinaryFile::new(input);
    driver.open(&name(), OpenMode::Read).unwrap();
    driver
        .read_type_informations(&mut number, &mut text)
        .unwrap();
    assert_eq!((number, text.as_bytes()), (1, b"a".as_slice()));
    let failure = driver
        .read_type_informations(&mut number, &mut text)
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Storage_StreamTypeMismatchError");
}

#[test]
fn magic_number_checks_and_open_rules() {
    let (files, shared) = with_file("doc", b"BINFILExx");
    assert_eq!(
        BinaryFile::is_good_file_type(shared.clone(), &name()).unwrap(),
        StorageError::Ok
    );
    files.insert(&name(), b"BIN".to_vec());
    assert_eq!(
        BinaryFile::is_good_file_type(shared.clone(), &name()).unwrap(),
        StorageError::FormatError
    );
    assert_eq!(
        BinaryFile::is_good_file_type(shared.clone(), &"none".into()).unwrap(),
        StorageError::OpenError
    );
    let mut driver = BinaryFile::new(shared);
    assert_eq!(
        driver.open(&name(), OpenMode::None).unwrap(),
        StorageError::OpenError
    );
    assert_eq!(driver.close().unwrap(), StorageError::NotOpen);
    assert_eq!(
        driver.tell().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    driver.open(&name(), OpenMode::ReadWrite).unwrap();
    assert_eq!(
        driver.open(&name(), OpenMode::Read).unwrap(),
        StorageError::AlreadyOpen
    );
    driver.put_boolean(true).unwrap();
    assert_eq!(driver.tell().unwrap(), 4);
    driver.skip_object().unwrap();
    driver.destroy();
    assert_eq!(BinaryFile::magic_number(), "BINFILE");
}

#[test]
fn refused_writes_and_read_only_files() {
    let mut driver = BinaryFile::new(Rc::new(RefusingFileSystem));
    driver.open(&name(), OpenMode::Write).unwrap();
    assert_eq!(
        driver.put_short_real(1.0).unwrap_err().exception_type(),
        "Storage_StreamWriteError"
    );
    let mut reader = BinaryFile::new(Rc::new(MemoryFileSystem::default()));
    reader.open(&name(), OpenMode::Write).unwrap();
    let mut value = 0.0;
    assert!(
        reader.get_real(&mut value).is_err(),
        "a write-only file cannot be read"
    );
}

#[test]
fn every_scalar_round_trips_in_native_order() {
    let files = MemoryFileSystem::default();
    let mut driver = BinaryFile::new(Rc::new(files.clone()));
    driver.open(&name(), OpenMode::Write).unwrap();
    driver.put_reference(5).unwrap();
    driver.put_character(b'z').unwrap();
    driver.put_ext_character(0x20AC).unwrap();
    driver.put_real(-2.5).unwrap();
    driver.put_short_real(0.25).unwrap();
    driver.write_persistent_object_header(1, 2).unwrap();
    driver.close().unwrap();
    let (_, input) = with_file("doc", &files.contents(&name()).unwrap());
    let mut reader = BinaryFile::new(input);
    reader.open(&name(), OpenMode::Read).unwrap();
    let (mut reference, mut byte, mut unit, mut real, mut short) = (0, 0, 0, 0.0, 0.0);
    reader.get_reference(&mut reference).unwrap();
    reader.get_character(&mut byte).unwrap();
    reader.get_ext_character(&mut unit).unwrap();
    reader.get_real(&mut real).unwrap();
    reader.get_short_real(&mut short).unwrap();
    let (mut object, mut kind) = (0, 0);
    reader
        .read_persistent_object_header(&mut object, &mut kind)
        .unwrap();
    for frame in [
        BinaryFile::begin_read_persistent_object_data,
        BinaryFile::begin_read_object_data,
        BinaryFile::end_read_object_data,
        BinaryFile::end_read_persistent_object_data,
    ] {
        frame(&mut reader).unwrap();
    }
    assert_eq!(
        (reference, byte, unit, real, short, object, kind),
        (5, b'z', 0x20AC, -2.5, 0.25, 1, 2)
    );
}

#[test]
fn stream_writers_report_sizes_and_can_only_count() {
    let mut stream = IoStream::from_bytes(Vec::new());
    let mut info = Sequence::default();
    info.append(AsciiString::new("ab"));
    let size = BinaryFile::write_info_stream(
        &mut stream,
        1,
        &"1.3".into(),
        &"".into(),
        &"".into(),
        &"".into(),
        &ExtendedString::new("A"),
        &"".into(),
        &ExtendedString::default(),
        &info,
        true,
    )
    .unwrap();
    assert_eq!(size, 4 + 7 + 4 + 4 + 4 + 6 + 4 + 4 + 4 + 6);
    assert_eq!(stream.tellp(), 0, "counting writes nothing");
    let mut comments = Sequence::default();
    comments.append(ExtendedString::new("c"));
    assert_eq!(
        BinaryFile::write_comment_stream(&mut stream, &comments, false).unwrap(),
        10
    );
    assert_eq!(
        BinaryFile::write_header_stream(&mut stream, &FileHeader::unset(), false).unwrap(),
        52
    );
    let mut driver = BinaryFile::new(Rc::new(MemoryFileSystem::default()));
    assert_eq!(
        driver.end_write_comment_section_stream(&mut stream),
        StorageError::Ok
    );
    assert_eq!(driver.file_header().end_comment, 62);
    driver.begin_write_comment_section_stream(&mut stream);
    driver.end_write_info_section_stream(&mut stream);
    assert_eq!(driver.file_header().end_info, 62);
}

#[test]
fn stream_readers_decode_the_info_section() {
    let mut stream = IoStream::from_bytes(Vec::new());
    BinaryFile::write_info_stream(
        &mut stream,
        2,
        &"1.3".into(),
        &"d".into(),
        &"S".into(),
        &"v".into(),
        &ExtendedString::new("App"),
        &"av".into(),
        &ExtendedString::new("T"),
        &Sequence::default(),
        false,
    )
    .unwrap();
    let bytes = stream.into_device().map(|_| ()).is_some();
    assert!(bytes);
    let mut encoded = Vec::new();
    encoded.extend(int(2));
    for text in ["1.3", "d", "S", "v"] {
        encoded.extend(int(text.len() as i32));
        encoded.extend_from_slice(text.as_bytes());
    }
    encoded.extend(int(1));
    encoded.extend_from_slice(&0x41_u16.to_ne_bytes());
    encoded.extend(int(0));
    encoded.extend(int(0));
    encoded.extend(int(1));
    encoded.extend(int(1));
    encoded.push(b'u');
    let header = Handle::new(HeaderData::default());
    BinaryFile::read_header_data(&mut IoStream::from_bytes(encoded), &header).unwrap();
    let header = header.get().unwrap();
    assert_eq!(
        (header.number_of_objects(), header.schema_name().as_bytes()),
        (2, b"S".as_slice())
    );
    assert_eq!(header.application_name().as_utf16(), &[0x41]);
    assert_eq!(header.user_info().length(), 1);
}

#[test]
fn stream_string_readers_raise_read_errors_on_short_input() {
    let mut bytes = int(4);
    bytes.extend_from_slice(b"ab");
    let mut text = AsciiString::default();
    let failure =
        BinaryFile::read_string_stream(&mut IoStream::from_bytes(bytes), &mut text).unwrap_err();
    assert_eq!(failure.exception_type(), "Storage_StreamReadError");
    let mut value = 0;
    let failure = BinaryFile::get_reference_stream(&mut IoStream::from_bytes(vec![1]), &mut value)
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Storage_StreamTypeMismatchError");
    let mut stream = IoStream::from_bytes([int(1), int(7), int(9)].concat());
    let (mut reference, mut kind) = (0, 0);
    assert_eq!(BinaryFile::ref_section_size_stream(&mut stream).unwrap(), 1);
    BinaryFile::read_reference_type_stream(&mut stream, &mut reference, &mut kind).unwrap();
    assert_eq!((reference, kind), (7, 9));
}

#[test]
fn complete_info_reads_a_container_without_objects() {
    let files = MemoryFileSystem::default();
    let data = Handle::new(Data::default());
    data.get()
        .unwrap()
        .add_named_root(&"doc".into(), &Handle::default());
    let mut writer = BinaryFile::new(Rc::new(files.clone()));
    writer.open(&name(), OpenMode::Write).unwrap();
    rustycad_tkernel::storage::Schema::default()
        .write(&mut writer, &data)
        .unwrap();
    writer.close().unwrap();
    let bytes = files.contents(&name()).unwrap();
    let mut stream = IoStream::from_bytes(bytes);
    let magic = rustycad_tkernel::storage::DriverBase::read_magic_number(&mut stream);
    assert_eq!(magic.as_bytes(), b"BINFILE");
    let mut read: Handle<Data> = Handle::default();
    let mut driver = BinaryFile::new(Rc::new(files));
    let failure = driver
        .read_complete_info(&mut stream, &mut read)
        .unwrap_err();
    assert_eq!(
        failure.exception_type(),
        "Standard_OutOfRange",
        "the written root reference 1 has no object in an empty read array"
    );
    assert_eq!(read.get().unwrap().number_of_roots(), 1);
}
