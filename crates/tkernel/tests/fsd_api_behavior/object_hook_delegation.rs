use super::fakes::{RefusingFileSystem, with_file};
use rustycad_tkernel::fsd::{DriverFileSystem, TextDialect, TextFile};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::{BaseDriver, DataSections, OpenMode, StorageError};
use std::rc::Rc;

#[derive(Default, Debug, Eq, PartialEq)]
struct ObjectHookCalls {
    write_header: usize,
    begin_write: usize,
    end_write: usize,
    read_header: usize,
    begin_read: usize,
    end_read: usize,
}

#[derive(Default)]
struct FsdObjectDialect {
    calls: ObjectHookCalls,
}

#[derive(Default)]
struct CmpObjectDialect {
    calls: ObjectHookCalls,
}

impl TextDialect for FsdObjectDialect {
    const MAGIC_NUMBER: &'static str = "FSDOBJECT";

    fn write_object_header(
        file: &mut TextFile<Self>,
        reference: i32,
        type_number: i32,
    ) -> Result<(), Failure> {
        file.dialect_mut().unwrap().calls.write_header += 1;
        file.write_fsd_object_header_base(reference, type_number)
    }

    fn begin_write_object(file: &mut TextFile<Self>, member: &str) -> Result<(), Failure> {
        file.dialect_mut().unwrap().calls.begin_write += 1;
        file.begin_fsd_write_object_base(member)
    }

    fn end_write_object(
        file: &mut TextFile<Self>,
        last: bool,
        member: &str,
    ) -> Result<(), Failure> {
        file.dialect_mut().unwrap().calls.end_write += 1;
        file.end_fsd_write_object_base(last, member)
    }

    fn read_object_header(
        file: &mut TextFile<Self>,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure> {
        file.dialect_mut().unwrap().calls.read_header += 1;
        file.read_fsd_object_header_base(reference, type_number)
    }

    fn begin_read_object(file: &mut TextFile<Self>) -> Result<(), Failure> {
        file.dialect_mut().unwrap().calls.begin_read += 1;
        file.begin_fsd_read_object_base()
    }

    fn end_read_object(file: &mut TextFile<Self>, last: bool) -> Result<(), Failure> {
        file.dialect_mut().unwrap().calls.end_read += 1;
        file.end_fsd_read_object_base(last)
    }
}

impl TextDialect for CmpObjectDialect {
    const MAGIC_NUMBER: &'static str = "CMPOBJECT";
    const USE_CMP_OBJECT_DEFAULTS: bool = true;

    fn write_object_header(
        file: &mut TextFile<Self>,
        reference: i32,
        type_number: i32,
    ) -> Result<(), Failure> {
        file.dialect_mut().unwrap().calls.write_header += 1;
        file.write_cmp_object_header_base(reference, type_number)
    }

    fn begin_write_object(file: &mut TextFile<Self>, member: &str) -> Result<(), Failure> {
        file.dialect_mut().unwrap().calls.begin_write += 1;
        file.begin_cmp_write_object_base(member)
    }

    fn end_write_object(
        file: &mut TextFile<Self>,
        last: bool,
        member: &str,
    ) -> Result<(), Failure> {
        file.dialect_mut().unwrap().calls.end_write += 1;
        file.end_cmp_write_object_base(last, member)
    }

    fn read_object_header(
        file: &mut TextFile<Self>,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure> {
        file.dialect_mut().unwrap().calls.read_header += 1;
        file.read_cmp_object_header_base(reference, type_number)
    }

    fn begin_read_object(file: &mut TextFile<Self>) -> Result<(), Failure> {
        file.dialect_mut().unwrap().calls.begin_read += 1;
        file.begin_cmp_read_object_base()
    }

    fn end_read_object(file: &mut TextFile<Self>, last: bool) -> Result<(), Failure> {
        file.dialect_mut().unwrap().calls.end_read += 1;
        file.end_cmp_read_object_base(last)
    }
}

fn write_record<D: TextDialect>(file: &mut TextFile<D>) -> Result<(), Failure> {
    file.begin_write_data_section()?;
    file.write_persistent_object_header(42, 3)?;
    file.begin_write_persistent_object_data()?;
    file.begin_write_object_data()?;
    file.end_write_object_data()?;
    file.end_write_persistent_object_data()?;
    file.end_write_data_section()?;
    Ok(())
}

fn read_record<D: TextDialect>(file: &mut TextFile<D>) -> Result<(i32, i32), Failure> {
    if file.begin_read_data_section()? != StorageError::Ok {
        return Ok((0, 0));
    }
    let (mut reference, mut type_number) = (0, 0);
    file.read_persistent_object_header(&mut reference, &mut type_number)?;
    file.begin_read_persistent_object_data()?;
    file.begin_read_object_data()?;
    file.end_read_object_data()?;
    file.end_read_persistent_object_data()?;
    file.end_read_data_section()?;
    Ok((reference, type_number))
}

#[test]
fn fsd_object_overrides_delegate_to_qualified_bodies_for_write_and_read() {
    assert_fsd_object_write();
    assert_fsd_object_read();
}

fn assert_fsd_object_write() {
    let files = rustycad_tkernel::fsd::MemoryFileSystem::default();
    let filesystem: Rc<dyn DriverFileSystem> = Rc::new(files.clone());
    let mut writer = TextFile::with_dialect(filesystem, FsdObjectDialect::default());
    writer.open(&"object.fsd".into(), OpenMode::Write).unwrap();
    write_record(&mut writer).unwrap();
    assert_eq!(writer.dialect().unwrap().calls.write_header, 1);
    assert_eq!(writer.dialect().unwrap().calls.begin_write, 2);
    assert_eq!(writer.dialect().unwrap().calls.end_write, 2);
    writer.close().unwrap();
    assert_eq!(
        files.contents(&"object.fsd".into()).unwrap(),
        b"BEGIN_DATA_SECTION\n#42=%3( ( ) )\nEND_DATA_SECTION\n"
    );
}

fn assert_fsd_object_read() {
    let (_, filesystem) = with_file(
        "object.fsd",
        b"BEGIN_DATA_SECTION\n#42=%3( ( ) )\nEND_DATA_SECTION\n",
    );
    let mut reader = TextFile::with_dialect(filesystem, FsdObjectDialect::default());
    reader.open(&"object.fsd".into(), OpenMode::Read).unwrap();
    assert_eq!(read_record(&mut reader).unwrap(), (42, 3));
    assert_eq!(reader.dialect().unwrap().calls.read_header, 1);
    assert_eq!(reader.dialect().unwrap().calls.begin_read, 2);
    assert_eq!(reader.dialect().unwrap().calls.end_read, 2);
}

#[test]
fn cmp_object_overrides_delegate_to_qualified_bodies_for_write_and_read() {
    assert_cmp_object_write();
    assert_cmp_object_read();
}

fn assert_cmp_object_write() {
    let files = rustycad_tkernel::fsd::MemoryFileSystem::default();
    let filesystem: Rc<dyn DriverFileSystem> = Rc::new(files.clone());
    let mut writer = TextFile::with_dialect(filesystem, CmpObjectDialect::default());
    writer.open(&"object.cmp".into(), OpenMode::Write).unwrap();
    write_record(&mut writer).unwrap();
    assert_eq!(writer.dialect().unwrap().calls.write_header, 1);
    assert_eq!(writer.dialect().unwrap().calls.begin_write, 2);
    assert_eq!(writer.dialect().unwrap().calls.end_write, 2);
    writer.close().unwrap();
    assert_eq!(
        files.contents(&"object.cmp".into()).unwrap(),
        b"BEGIN_DATA_SECTION\n#42%3 \nEND_DATA_SECTION\n"
    );
}

fn assert_cmp_object_read() {
    let (_, filesystem) = with_file(
        "object.cmp",
        b"BEGIN_DATA_SECTION\n#42%3 \nEND_DATA_SECTION\n",
    );
    let mut reader = TextFile::with_dialect(filesystem, CmpObjectDialect::default());
    reader.open(&"object.cmp".into(), OpenMode::Read).unwrap();
    assert_eq!(read_record(&mut reader).unwrap(), (42, 3));
    assert_eq!(reader.dialect().unwrap().calls.read_header, 1);
    assert_eq!(reader.dialect().unwrap().calls.begin_read, 2);
    assert_eq!(reader.dialect().unwrap().calls.end_read, 2);
}

#[test]
fn object_header_body_delegation_preserves_stream_write_error() {
    assert_header_write_failure::<FsdObjectDialect>();
    assert_header_write_failure::<CmpObjectDialect>();
}

fn assert_header_write_failure<D: TextDialect + Default>() {
    let mut writer = TextFile::with_dialect(Rc::new(RefusingFileSystem), D::default());
    writer.open(&"refused".into(), OpenMode::Write).unwrap();
    let failure = writer.write_persistent_object_header(7, 2).unwrap_err();
    assert_eq!(failure.exception_type(), "Storage_StreamWriteError");
}

#[test]
fn object_read_helpers_keep_native_header_errors() {
    assert_bad_header::<FsdObjectDialect>(b"#x=%2");
    assert_bad_header::<CmpObjectDialect>(b"#x%2 ");
}

fn assert_bad_header<D: TextDialect + Default>(bytes: &[u8]) {
    let (_, filesystem) = with_file("bad-header", bytes);
    let mut reader = TextFile::with_dialect(filesystem, D::default());
    reader.open(&"bad-header".into(), OpenMode::Read).unwrap();
    let (mut reference, mut type_number) = (0, 0);
    let failure = reader
        .read_persistent_object_header(&mut reference, &mut type_number)
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Storage_StreamTypeMismatchError");
}
