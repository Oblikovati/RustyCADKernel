//! The file variants of BinTools::Write and BinTools::Read, on injected OSD file
//! systems.
use super::fakes::{MemoryFileSystem, RecordingConsole, RefusingFileSystem, written};
use super::support::make_box;
use rustycad_tkbrep::bintools::{self, WriteOptions};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::Arc;

#[test]
fn a_shape_written_to_a_file_has_the_stream_bytes_and_reads_back() {
    let files = MemoryFileSystem::default();
    let solid = make_box(1.0, 2.0, 3.0);
    let name = AsciiString::new("box.bin");
    let range = ProgressRange::default();
    assert!(
        bintools::write_file_on(&files, &solid, &name, WriteOptions::default(), &range).unwrap()
    );
    let expected =
        written(|stream| bintools::write(&solid, stream, WriteOptions::default(), &range).unwrap());
    assert_eq!(files.contents("box.bin").unwrap(), expected);
    let mut back = Shape::default();
    let console = Arc::new(RecordingConsole::default());
    // The shape reference ends the file with an exact read, which leaves the stream good.
    let good = bintools::read_file_on(&files, &mut back, &name, &range, console.clone()).unwrap();
    assert!(good);
    assert_eq!(back.nb_children(), 1);
    assert_eq!(console.text(), "");
}

#[test]
fn a_file_system_that_opens_nothing_makes_both_variants_return_false() {
    let range = ProgressRange::default();
    let name = AsciiString::new("none.bin");
    let wrote = bintools::write_file_on(
        &RefusingFileSystem,
        &Shape::default(),
        &name,
        WriteOptions::default(),
        &range,
    );
    assert!(!wrote.unwrap());
    let mut shape = Shape::default();
    let console = Arc::new(RecordingConsole::default());
    assert!(
        !bintools::read_file_on(&RefusingFileSystem, &mut shape, &name, &range, console).unwrap()
    );
}

#[test]
fn a_failing_flush_makes_the_write_false() {
    let files = MemoryFileSystem::failing_flush();
    let range = ProgressRange::default();
    let name = AsciiString::new("flush.bin");
    assert!(
        !bintools::write_file_on(
            &files,
            &make_box(1.0, 1.0, 1.0),
            &name,
            WriteOptions::default(),
            &range
        )
        .unwrap()
    );
}

#[test]
fn a_missing_file_is_not_read() {
    let files = MemoryFileSystem::default();
    files.insert("other.bin", Vec::new());
    let mut shape = Shape::default();
    let console = Arc::new(RecordingConsole::default());
    let name = AsciiString::new("missing.bin");
    assert!(
        !bintools::read_file_on(
            &files,
            &mut shape,
            &name,
            &ProgressRange::default(),
            console
        )
        .unwrap()
    );
}

#[test]
fn an_empty_file_prints_the_version_message_then_fails_on_the_reference() {
    let files = MemoryFileSystem::default();
    files.insert("empty.bin", Vec::new());
    let mut shape = Shape::default();
    let console = Arc::new(RecordingConsole::default());
    let name = AsciiString::new("empty.bin");
    let error = bintools::read_file_on(
        &files,
        &mut shape,
        &name,
        &ProgressRange::default(),
        console.clone(),
    )
    .unwrap_err();
    assert_eq!(error.exception_type(), "Storage_StreamTypeMismatchError");
    assert!(
        console
            .text()
            .contains("File was not written with this version")
    );
}
