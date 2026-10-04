use super::fakes::with_file;
use rustycad_tkernel::fsd::{TextDialect, TextFile};
use rustycad_tkernel::storage::{
    BaseDriver, HeaderSections, OpenMode, StorageError, TableSections,
};
use rustycad_tkernel::tcollection::AsciiString;

#[derive(Default)]
struct StatefulDialect {
    word: String,
    read_char_calls: usize,
    read_word_calls: usize,
    flush_calls: usize,
    find_tag_calls: usize,
}

impl StatefulDialect {
    fn new(word: &str) -> Self {
        Self {
            word: word.to_owned(),
            ..Self::default()
        }
    }
}

impl TextDialect for StatefulDialect {
    const MAGIC_NUMBER: &'static str = "FSDFILE";

    fn read_char(file: &mut TextFile<Self>, size: usize) -> AsciiString {
        file.dialect_mut().unwrap().read_char_calls += 1;
        let mut output = AsciiString::default();
        let mut byte = 0;
        for _ in 0..size {
            if file.stream().eof() {
                break;
            }
            file.stream().get(&mut byte);
            output.assign_cat(byte);
        }
        output
    }

    fn read_word(file: &mut TextFile<Self>) -> AsciiString {
        let word = {
            let dialect = file.dialect_mut().unwrap();
            dialect.read_word_calls += 1;
            dialect.word.clone()
        };
        AsciiString::new(&word)
    }

    fn flush_end_of_line(file: &mut TextFile<Self>) {
        file.dialect_mut().unwrap().flush_calls += 1;
        Self::read_line(file);
    }

    fn find_tag(file: &mut TextFile<Self>, tag: &str) -> StorageError {
        file.dialect_mut().unwrap().find_tag_calls += 1;
        let mut line = file.read_string();
        while line.as_bytes() != tag.as_bytes() && !file.stream().eof() {
            line = file.read_string();
        }
        if file.stream().eof() {
            StorageError::SectionNotFound
        } else {
            StorageError::Ok
        }
    }
}

fn opened_dialect(bytes: &[u8], word: &str) -> TextFile<StatefulDialect> {
    let (_, filesystem) = with_file("custom", bytes);
    let mut file = TextFile::with_dialect(filesystem, StatefulDialect::new(word));
    file.open(&"custom".into(), OpenMode::Read).unwrap();
    file
}

#[test]
fn custom_dialect_values_are_owned_per_instance() {
    let mut first = opened_dialect(b"unused", "first");
    let mut second = opened_dialect(b"unused", "second");
    assert_eq!(first.read_word().as_bytes(), b"first");
    assert_eq!(second.read_word().as_bytes(), b"second");
    assert_eq!(first.dialect().unwrap().read_word_calls, 1);
    assert_eq!(second.dialect().unwrap().read_word_calls, 1);
}

#[test]
fn static_good_file_probe_uses_the_base_reader_without_dialect_state() {
    let (_, filesystem) = with_file("custom", b"FSDFILE");
    let status =
        TextFile::<StatefulDialect>::is_good_file_type(filesystem, &"custom".into()).unwrap();
    assert_eq!(status, StorageError::Ok);
}

#[test]
fn virtual_primitives_dispatch_from_direct_and_internal_driver_calls() {
    assert_direct_virtual_calls();
    assert_internal_virtual_calls();
}

fn assert_direct_virtual_calls() {
    let mut direct = opened_dialect(b"xy rest", "direct");
    assert_eq!(direct.read_char(2).as_bytes(), b"xy");
    assert_eq!(direct.read_word().as_bytes(), b"direct");
    direct.flush_end_of_line();
    assert_eq!(direct.find_tag("absent"), StorageError::SectionNotFound);
    let dialect = direct.dialect().unwrap();
    assert_eq!(dialect.flush_calls, 1);
    assert_eq!(dialect.find_tag_calls, 1);
}

fn assert_internal_virtual_calls() {
    let mut file = opened_dialect(b"FSDFILE\nBEGIN_INFO_SECTION\n1\n2 ignored\n", "override");
    assert_eq!(file.begin_read_info_section().unwrap(), StorageError::Ok);
    assert_eq!(file.root_section_size().unwrap(), 1);
    let (mut name, mut reference, mut kind) = (AsciiString::default(), 0, AsciiString::default());
    file.read_root(&mut name, &mut reference, &mut kind)
        .unwrap();
    assert_eq!(
        (name.as_bytes(), reference, kind.as_bytes()),
        (b"override".as_slice(), 2, b"override".as_slice())
    );
    let dialect = file.dialect().unwrap();
    assert_eq!(dialect.read_char_calls, 1);
    assert_eq!(dialect.find_tag_calls, 1);
    assert_eq!(dialect.flush_calls, 1);
    assert_eq!(dialect.read_word_calls, 2);
}
