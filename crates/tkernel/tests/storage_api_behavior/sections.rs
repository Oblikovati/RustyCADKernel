//! Storage_HeaderData/RootData/TypeData::Read against a scripted driver, and the driver
//! helpers of Storage_BaseDriver.
use super::fakes::ScriptedDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::storage::{
    BaseDriver, DriverBase, HeaderData, IoStream, OpenMode, RootData, StorageError,
    StreamException, TypeData,
};

#[test]
fn header_read_refuses_a_driver_not_open_for_reading() {
    let header = HeaderData::default();
    let mut driver = ScriptedDriver::opened(OpenMode::Write);
    assert!(!header.read(&mut driver).unwrap());
    assert_eq!(header.error_status(), StorageError::ModeError);
    assert_eq!(header.error_status_extension().as_bytes(), b"OpenMode");
    assert!(driver.log.is_empty());
}

#[test]
fn header_read_runs_info_then_comment_sections() {
    let header = HeaderData::default();
    let mut driver = ScriptedDriver::opened(OpenMode::ReadWrite);
    driver.info_count = 3;
    assert!(header.read(&mut driver).unwrap());
    assert_eq!(header.number_of_objects(), 3);
    assert_eq!(header.comments().length(), 1);
    let steps = [
        "BeginReadInfoSection",
        "ReadInfo",
        "EndReadInfoSection",
        "BeginReadCommentSection",
        "ReadComment",
        "EndReadCommentSection",
    ];
    assert_eq!(driver.log, steps);
}

#[test]
fn caught_stream_exceptions_become_the_status_with_the_step_name() {
    for (kind, status) in [
        (StreamException::TypeMismatch, StorageError::TypeMismatch),
        (
            StreamException::ExtCharParity,
            StorageError::ExtCharParityError,
        ),
    ] {
        let header = HeaderData::default();
        let mut driver = ScriptedDriver::opened(OpenMode::Read);
        driver.info_count = 7;
        driver.fail_at = Some(("ReadInfo", kind));
        assert!(!header.read(&mut driver).unwrap());
        assert_eq!(header.error_status(), status);
        assert_eq!(header.error_status_extension().as_bytes(), b"ReadInfo");
        assert_eq!(
            header.number_of_objects(),
            7,
            "fields read before the failure stay"
        );
    }
}

#[test]
fn other_exceptions_propagate_and_statuses_stop_the_read() {
    let header = HeaderData::default();
    let mut driver = ScriptedDriver::opened(OpenMode::Read);
    driver.fail_at = Some(("ReadComment", StreamException::Format));
    assert!(header.read(&mut driver).is_err());
    let mut driver = ScriptedDriver::opened(OpenMode::Read);
    driver.status_at = Some(("EndReadInfoSection", StorageError::SectionNotFound));
    assert!(!header.read(&mut driver).unwrap());
    assert_eq!(
        header.error_status_extension().as_bytes(),
        b"EndReadInfoSection"
    );
    header.clear_error_status();
    assert_eq!(header.error_status(), StorageError::Ok);
}

#[test]
fn root_and_type_reads_bind_every_entry() {
    let roots = RootData::default();
    let mut driver = ScriptedDriver::opened(OpenMode::Read);
    driver.roots = vec![("doc", 1, "PDoc"), ("aux", 2, "PAux")];
    assert!(roots.read(&mut driver).unwrap());
    assert_eq!(roots.number_of_roots(), 2);
    assert_eq!(roots.find(&"aux".into()).get().unwrap().reference(), 2);
    let types = TypeData::default();
    driver.types = vec![(1, "PDoc"), (2, "PAux")];
    assert!(types.read(&mut driver).unwrap());
    assert_eq!(types.type_number(&"PAux".into()).unwrap(), 2);
}

#[test]
fn a_type_mismatch_in_an_entry_stops_the_section() {
    let roots = RootData::default();
    let mut driver = ScriptedDriver::opened(OpenMode::Read);
    driver.roots = vec![("doc", 1, "PDoc")];
    driver.fail_at = Some(("ReadRoot", StreamException::TypeMismatch));
    assert!(!roots.read(&mut driver).unwrap());
    assert_eq!(roots.error_status(), StorageError::TypeMismatch);
    assert_eq!(roots.error_status_extension().as_bytes(), b"ReadRoot");
    let types = TypeData::default();
    driver.fail_at = Some(("TypeSectionSize", StreamException::TypeMismatch));
    assert!(
        types.read(&mut driver).is_err(),
        "section sizes are read outside the try"
    );
    let mut writer = ScriptedDriver::opened(OpenMode::Write);
    assert!(!types.read(&mut writer).unwrap());
    assert_eq!(types.error_status(), StorageError::ModeError);
    roots.clear_error_status();
    assert_eq!(roots.error_status(), StorageError::Ok);
}

#[test]
fn driver_base_and_the_stream_shorthands() {
    let mut driver = ScriptedDriver::default();
    driver.open(&"file".into(), OpenMode::Read).unwrap();
    assert_eq!(driver.name().as_bytes(), b"file");
    assert_eq!(driver.open_mode(), OpenMode::Read);
    let dynamic: &mut dyn BaseDriver = &mut driver;
    dynamic
        .put(1)
        .unwrap()
        .put(2.5_f64)
        .unwrap()
        .put(b'x')
        .unwrap();
    dynamic
        .put(0x20AC_u16)
        .unwrap()
        .put(true)
        .unwrap()
        .put(0.5_f32)
        .unwrap();
    let (mut count, mut flag) = (0, false);
    dynamic.get(&mut count).unwrap().get(&mut flag).unwrap();
    assert_eq!((count, flag), (12, true));
    assert_eq!(driver.log[1], "PutInteger 1");
    let _ = Handle::<HeaderData>::default();
}

#[test]
fn protected_driver_setters_change_state_without_opening_the_stream() {
    use rustycad_tkernel::fsd::{File, MemoryFileSystem};
    use std::rc::Rc;

    let mut file = File::new(Rc::new(MemoryFileSystem::default()));
    file.set_name(&"before".into());
    file.set_open_mode(OpenMode::Read);
    assert_eq!(file.name().as_bytes(), b"before");
    assert_eq!(
        file.open(&"request".into(), OpenMode::Write).unwrap(),
        StorageError::AlreadyOpen
    );
    assert_eq!(file.name().as_bytes(), b"request");
    assert_eq!(file.close().unwrap(), StorageError::Ok);
    assert_eq!(file.open_mode(), OpenMode::None);
    assert_eq!(file.name().as_bytes(), b"request");
}

#[test]
fn magic_number_reads_seven_characters_and_repeats_the_last_at_end_of_input() {
    let mut stream = IoStream::from_bytes(b"BINFILEextra".to_vec());
    assert_eq!(
        DriverBase::read_magic_number(&mut stream).as_bytes(),
        b"BINFILE"
    );
    let mut short = IoStream::from_bytes(b"BIN".to_vec());
    assert_eq!(
        DriverBase::read_magic_number(&mut short).as_bytes(),
        b"BINN"
    );
}
