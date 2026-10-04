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
