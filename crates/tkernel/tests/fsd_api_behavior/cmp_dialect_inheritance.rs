use super::fakes::with_file;
use rustycad_tkernel::fsd::{DriverFileSystem, TextDialect, TextFile};
use rustycad_tkernel::storage::{BaseDriver, OpenMode};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use std::rc::Rc;

#[derive(Default)]
struct DerivedCmpDialect {
    flushes: usize,
}

impl TextDialect for DerivedCmpDialect {
    const MAGIC_NUMBER: &'static str = "CUSTOM";
    const USE_CMP_TEXT_DEFAULTS: bool = true;

    fn flush_end_of_line(file: &mut TextFile<Self>) {
        file.dialect_mut().unwrap().flushes += 1;
        Self::read_line(file);
    }
}

#[derive(Default)]
struct DelegatingCmpDialect {
    line_calls: usize,
    string_calls: usize,
    extended_read_calls: usize,
    extended_write_calls: usize,
    flush_calls: usize,
}

impl TextDialect for DelegatingCmpDialect {
    const MAGIC_NUMBER: &'static str = "DELEGATE";
    const USE_CMP_TEXT_DEFAULTS: bool = true;

    fn read_line(file: &mut TextFile<Self>) -> AsciiString {
        file.dialect_mut().unwrap().line_calls += 1;
        file.read_cmp_line_base()
    }

    fn read_string(file: &mut TextFile<Self>) -> AsciiString {
        file.dialect_mut().unwrap().string_calls += 1;
        file.read_cmp_string_base()
    }

    fn read_extended_line(
        file: &mut TextFile<Self>,
        text: &mut ExtendedString,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        file.dialect_mut().unwrap().extended_read_calls += 1;
        file.read_cmp_extended_line_base(text)
    }

    fn write_extended_line(
        file: &mut TextFile<Self>,
        text: &ExtendedString,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        file.dialect_mut().unwrap().extended_write_calls += 1;
        file.write_cmp_extended_line_base(text)
    }

    fn flush_end_of_line(file: &mut TextFile<Self>) {
        file.dialect_mut().unwrap().flush_calls += 1;
        file.flush_fsd_end_of_line_base();
    }
}

fn opened_custom(bytes: &[u8]) -> TextFile<DerivedCmpDialect> {
    let (_, filesystem) = with_file("custom-cmp", bytes);
    opened_with(filesystem, OpenMode::Read)
}

fn opened_with(
    filesystem: Rc<dyn DriverFileSystem>,
    mode: OpenMode,
) -> TextFile<DerivedCmpDialect> {
    let mut file = TextFile::with_dialect(filesystem, DerivedCmpDialect::default());
    file.open(&"custom-cmp".into(), mode).unwrap();
    file
}

#[test]
fn custom_dialect_inherits_cmp_line_and_string_defaults() {
    let mut file = opened_custom(b"one\r\n  two\r\n");
    assert_eq!(file.read_line().as_bytes(), b"one");
    assert_eq!(file.read_string().as_bytes(), b"two");
}

#[test]
fn custom_dialect_inherits_cmp_extended_read_and_dispatches_its_flush_hook() {
    let mut file = opened_custom(b"0\n  tail\r\n");
    file.read_extended_line(&mut ExtendedString::default())
        .unwrap();
    assert_eq!(file.dialect().unwrap().flushes, 1);
    assert_eq!(file.read_line().as_bytes(), b"  tail");
}

#[test]
fn custom_dialect_inherits_cmp_extended_write_default() {
    let files = rustycad_tkernel::fsd::MemoryFileSystem::default();
    let filesystem = Rc::new(files.clone());
    let mut file = opened_with(filesystem as Rc<dyn DriverFileSystem>, OpenMode::Write);
    file.write_extended_line(&ExtendedString::new("AB"))
        .unwrap();
    file.close().unwrap();
    assert_eq!(
        files.contents(&AsciiString::new("custom-cmp")).unwrap(),
        b"2 65 66 \n"
    );
}

#[test]
fn custom_overrides_delegate_to_cmp_bodies_and_keep_nested_dispatch() {
    let (_, filesystem) = with_file("custom-cmp", b"first\r\n second\r\n0\n last\r\n");
    let mut file = TextFile::with_dialect(filesystem, DelegatingCmpDialect::default());
    file.open(&"custom-cmp".into(), OpenMode::Read).unwrap();
    assert_eq!(file.read_line().as_bytes(), b"first");
    assert_eq!(file.read_string().as_bytes(), b"second");
    assert_cmp_extended_read_dispatch(&mut file);
}

fn assert_cmp_extended_read_dispatch(file: &mut TextFile<DelegatingCmpDialect>) {
    file.read_extended_line(&mut ExtendedString::default())
        .unwrap();
    assert_eq!(file.read_line().as_bytes(), b" last");
    let dialect = file.dialect().unwrap();
    assert_eq!(dialect.line_calls, 3);
    assert_eq!(dialect.string_calls, 1);
    assert_eq!(dialect.extended_read_calls, 1);
    assert_eq!(dialect.flush_calls, 1);
}

#[test]
fn custom_write_override_delegates_to_cmp_base_body() {
    let files = rustycad_tkernel::fsd::MemoryFileSystem::default();
    let filesystem = Rc::new(files.clone());
    let mut file = TextFile::with_dialect(filesystem, DelegatingCmpDialect::default());
    file.open(&"delegating-cmp-write".into(), OpenMode::Write)
        .unwrap();
    file.write_extended_line(&ExtendedString::new("AB"))
        .unwrap();
    assert_eq!(file.dialect().unwrap().extended_write_calls, 1);
    file.close().unwrap();
    assert_eq!(
        files
            .contents(&AsciiString::new("delegating-cmp-write"))
            .unwrap(),
        b"2 65 66 \n"
    );
}
