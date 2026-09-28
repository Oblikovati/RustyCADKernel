//! OSD_FileNode, OSD_File, OSD_Directory and the iterators over the named in-memory
//! `FakePosixCalls`: every public member and the native quirks each keeps. The native
//! behavior is compared with OCCT by the `osd_files_*` probes.
#![cfg(target_os = "linux")]
#[path = "osd_files/fake_calls.rs"]
mod fake_calls;

use fake_calls::{EACCES, EINVAL, ENOENT, FakeNode, FakePosixCalls};
use rustycad_tkernel::{
    osd::{
        CadPath, Directory, DirectoryIterator, File, FileIterator, FromWhere, KindFile, LockType,
        OpenMode, OsdErrorAccess, PosixFileCalls, Protection, RecordLock, SingleProtection,
        mask_matches,
    },
    quantity::Date,
    tcollection::AsciiString,
};
use std::sync::Arc;

fn fake() -> Arc<FakePosixCalls> {
    Arc::new(FakePosixCalls::new().with_directory("/w"))
}

fn file(calls: &Arc<FakePosixCalls>, name: &str) -> File {
    File::with_calls(
        CadPath::new(name),
        Arc::clone(calls) as Arc<dyn PosixFileCalls>,
    )
}

#[test]
fn build_write_read_and_close_follow_the_descriptor() {
    let calls = fake();
    let mut target = file(&calls, "/w/a.txt");
    target
        .build(OpenMode::ReadWrite, &Protection::new())
        .unwrap();
    assert!(target.is_open());
    assert_eq!(calls.mode("/w/a.txt"), Some(0o644 & !0o022));
    target.write(&AsciiString::new("one\ntwo\n"), 8).unwrap();
    target.seek(0, FromWhere::FromBeginning).unwrap();
    let mut text = AsciiString::default();
    target.read(&mut text, 3).unwrap();
    assert_eq!(text.as_bytes(), b"one");
    assert!(!target.is_at_end().unwrap());
    target.read(&mut text, 100).unwrap();
    assert_eq!(text.as_bytes(), b"\ntwo\n");
    assert!(target.is_at_end().unwrap());
    target.seek(0, FromWhere::FromBeginning).unwrap();
    assert!(target.is_at_end().unwrap(), "native Seek keeps myIO == EOF");
    let mut raw = [0u8; 4];
    assert_eq!(target.read_bytes(&mut raw, 4).unwrap(), 4);
    assert_eq!(&raw, b"one\n");
    assert_eq!(target.size().unwrap(), 8);
    target.close().unwrap();
    assert!(!target.is_open());
    assert_eq!(calls.open_descriptors(), 0);
    assert!(target.is_at_end().is_err());
}

#[test]
fn read_line_reads_through_the_stdio_buffer() {
    let calls = Arc::new(FakePosixCalls::with_block_size(4));
    calls.add(b"/l", FakeNode::File(b"ab\ncdef\ngh".to_vec()), 0o644);
    let mut lines = file(&calls, "/l");
    lines.open(OpenMode::ReadOnly, &Protection::new()).unwrap();
    let mut line = AsciiString::new("old");
    assert_eq!(lines.read_line(&mut line, 100).unwrap(), 3);
    assert_eq!(line.as_bytes(), b"ab\n");
    let mut raw = [0u8; 2];
    assert_eq!(lines.read_bytes(&mut raw, 2).unwrap(), 2);
    assert_eq!(
        &raw, b"de",
        "the stream buffered 4 bytes ahead of the descriptor"
    );
    assert_eq!(lines.read_line(&mut line, 100).unwrap(), 3);
    assert_eq!(
        line.as_bytes(),
        b"cf\n",
        "the buffered c, then the descriptor reads on"
    );
    assert_eq!(lines.read_line(&mut line, 3).unwrap(), 2);
    assert_eq!(line.as_bytes(), b"gh", "fgets reads count - 1 bytes");
    lines.rewind();
    assert_eq!(lines.read_line(&mut line, 100).unwrap(), 3);
    assert_eq!(line.as_bytes(), b"ab\n");
}

#[test]
fn rewind_inside_the_buffer_resyncs_the_descriptor_to_its_end() {
    let calls = Arc::new(FakePosixCalls::with_block_size(8));
    calls.add(b"/s", FakeNode::File(b"xy\n".to_vec()), 0o644);
    let mut short = file(&calls, "/s");
    short.open(OpenMode::ReadOnly, &Protection::new()).unwrap();
    let mut line = AsciiString::default();
    assert_eq!(short.read_line(&mut line, 100).unwrap(), 3);
    short.rewind();
    assert_eq!(
        short.read_line(&mut line, 100).unwrap(),
        3,
        "the first rewind seeks: the offset was unknown"
    );
    short.rewind();
    let mut raw = [0u8; 3];
    assert_eq!(
        short.read_bytes(&mut raw, 3).unwrap(),
        0,
        "the descriptor sits after the buffer"
    );
    assert_eq!(short.read_line(&mut line, 100).unwrap(), 3);
    assert_eq!(line.as_bytes(), b"xy\n");
}

#[test]
fn read_line_at_end_is_sticky_until_rewind() {
    let calls = fake();
    calls.add(b"/w/e", FakeNode::File(b"x".to_vec()), 0o644);
    let mut target = file(&calls, "/w/e");
    target
        .open(OpenMode::ReadWrite, &Protection::new())
        .unwrap();
    let mut line = AsciiString::default();
    assert_eq!(target.read_line(&mut line, 10).unwrap(), 1);
    assert_eq!(target.read_line(&mut line, 10).unwrap(), 0);
    assert!(line.is_empty());
    assert!(target.is_at_end().unwrap());
    target.write_bytes(b"yz", 2).unwrap();
    assert_eq!(
        target.read_line(&mut line, 10).unwrap(),
        0,
        "glibc end of file is sticky"
    );
    assert!(!target.read_last_line(&mut line, 0, 2).unwrap());
    target.rewind();
    assert!(target.read_last_line(&mut line, 0, 1).unwrap());
    assert_eq!(line.as_bytes(), b"xyz");
}

#[test]
fn open_failures_are_recorded_and_raised_by_the_next_operation() {
    let calls = fake();
    let mut missing = file(&calls, "/w/missing");
    missing
        .open(OpenMode::ReadOnly, &Protection::new())
        .unwrap();
    assert!(!missing.is_open());
    assert!(missing.failed());
    assert_eq!(missing.error(), 0);
    let failure = missing.perror().unwrap_err();
    assert_eq!(failure.exception_type(), "OSD_OSDError");
    assert_eq!(failure.message(), " :\n Invalid path (empty string).\n\n");
    assert_eq!(missing.error(), 9);
    missing.reset();
    assert!(!missing.failed());
    let error = file(&calls, "/w/")
        .build(OpenMode::ReadWrite, &Protection::new())
        .unwrap_err();
    assert_eq!(error.message(), "OSD_File::Build(): it is a directory");
    let error = file(&calls, "/nowhere/")
        .open(OpenMode::ReadOnly, &Protection::new())
        .unwrap_err();
    assert_eq!(error.message(), "OSD_File::Open(): no name was given");
    let mut writer = file(&calls, "/w/wo");
    writer
        .build(OpenMode::WriteOnly, &Protection::new())
        .unwrap();
    let error = writer.read(&mut AsciiString::default(), 1).unwrap_err();
    assert_eq!(error.message(), "OSD_File::Read(): file is Write only");
    assert_eq!(
        writer.write_bytes(b"", 0).unwrap_err().message(),
        "OSD_File::Write(): theNbBytes is null"
    );
    assert!(
        writer.write_bytes(b"ab", 3).is_err(),
        "count past the buffer is refused"
    );
    let error = writer
        .build(OpenMode::ReadWrite, &Protection::new())
        .unwrap_err();
    assert_eq!(
        error.message(),
        "OSD_File::Build(): incorrect call - file already opened"
    );
    let mut reader = file(&calls, "/w/wo");
    reader.open(OpenMode::ReadOnly, &Protection::new()).unwrap();
    assert_eq!(
        reader.write_bytes(b"a", 1).unwrap_err().message(),
        "OSD_File::Write(): file is Read only"
    );
}

#[test]
fn append_creates_then_appends() {
    let calls = fake();
    let mut target = file(&calls, "/w/log");
    target
        .append(OpenMode::WriteOnly, &Protection::new())
        .unwrap();
    target.write_bytes(b"a", 1).unwrap();
    target.close().unwrap();
    target
        .append(OpenMode::ReadWrite, &Protection::new())
        .unwrap();
    target.seek(0, FromWhere::FromBeginning).unwrap();
    target.write_bytes(b"b", 1).unwrap();
    assert_eq!(calls.contents("/w/log").unwrap(), b"ab");
    assert_eq!(target.kind_of_file(), KindFile::File);
    assert!(target.is_readable() && target.is_writeable() && !target.is_executable());
    assert!(!target.edit());
}

#[test]
fn node_operations_record_the_native_errors() {
    let calls = fake();
    calls.add(b"/w/ro", FakeNode::File(Vec::new()), 0o444);
    calls.add(b"/w/sock", FakeNode::Socket, 0o644);
    let mut read_only = file(&calls, "/w/ro");
    read_only.remove();
    assert_eq!(
        read_only.osd_error().errno(),
        EACCES,
        "Remove checks the node itself"
    );
    let mut socket = file(&calls, "/w/sock");
    assert_eq!(socket.kind_of_file(), KindFile::Socket);
    socket.remove();
    assert_eq!(socket.osd_error().errno(), EINVAL);
    let mut missing = file(&calls, "/w/none");
    missing.copy_to(&CadPath::new("/w/copy"));
    assert_eq!(missing.osd_error().errno(), -1);
    assert_eq!(
        missing.perror().unwrap_err().message(),
        " :\n Unknowm error #-1.\n\n"
    );
    missing.move_to(&CadPath::new("/w/other"));
    assert_eq!(missing.osd_error().errno(), ENOENT);
    let mut data = file(&calls, "/w/ro");
    data.copy_to(&CadPath::new("/w/copy"));
    assert!(!data.failed());
    assert_eq!(calls.mode("/w/copy"), Some(0o644 & !0o022));
    data.move_to(&CadPath::new("/w/moved"));
    assert!(!data.exists() && file(&calls, "/w/moved").exists());
}

#[test]
fn protection_maps_mode_bits_with_system_equal_to_group() {
    let calls = fake();
    calls.add(b"/w/p", FakeNode::File(Vec::new()), 0o751);
    let mut node = file(&calls, "/w/p");
    let protection = node.protection();
    use SingleProtection::*;
    assert_eq!(protection.values(), (Rx, Rwx, Rx, X));
    node.set_protection(&Protection::with_values(None, Rwd, R, W));
    assert_eq!(calls.mode("/w/p"), Some(0o642));
    let mut missing = file(&calls, "/w/q");
    assert_eq!(missing.protection().values(), (None, None, None, None));
    assert!(missing.failed());
}

#[test]
fn moments_read_the_change_time_or_the_1979_epoch() {
    let calls = fake();
    calls.add(b"/w/t", FakeNode::File(Vec::new()), 0o644);
    let expected = Date::new(1, 29, 1980, 13, 20, 0, 0, 0).unwrap();
    assert_eq!(file(&calls, "/w/t").creation_moment().unwrap(), expected);
    assert_eq!(file(&calls, "/w/t").access_moment().unwrap(), expected);
    assert_eq!(
        file(&calls, "/w/x").access_moment().unwrap(),
        Date::default()
    );
}

#[test]
fn locks_follow_the_sysv_branch() {
    let calls = fake();
    let mut target = file(&calls, "/w/lock");
    target
        .build(OpenMode::ReadWrite, &Protection::new())
        .unwrap();
    target.set_lock(LockType::WriteLock).unwrap();
    assert_eq!(target.lock(), LockType::WriteLock);
    assert!(
        !target.is_locked(),
        "IsLocked reads only the exclusive-lock flag"
    );
    target.set_lock(LockType::ExclusiveLock).unwrap();
    assert!(target.is_locked());
    assert_eq!(calls.mode("/w/lock").unwrap() & 0o2000, 0o2000);
    target.unlock().unwrap();
    assert_eq!(target.lock(), LockType::NoLock);
    assert!(!target.is_locked());
    assert_eq!(calls.mode("/w/lock").unwrap() & 0o2000, 0);
    target.set_lock(LockType::NoLock).unwrap();
    let requested: Vec<RecordLock> = calls.locks().into_iter().map(|(_, lock)| lock).collect();
    assert_eq!(
        requested,
        [RecordLock::Write, RecordLock::Write, RecordLock::Unlock]
    );
    assert!(file(&calls, "/w/z").set_lock(LockType::ReadLock).is_err());
}

#[test]
fn temporaries_use_the_tmp_templates() {
    let calls = Arc::new(
        FakePosixCalls::new()
            .with_directory("/w")
            .with_directory("/tmp"),
    );
    let mut temporary = File::with_calls(
        CadPath::default(),
        Arc::clone(&calls) as Arc<dyn PosixFileCalls>,
    );
    temporary.build_temporary().unwrap();
    assert!(temporary.is_open());
    assert_eq!(temporary.path().system_name().as_bytes(), b"/tmp/CSF000001");
    let directory = Directory::build_temporary_with(calls);
    assert_eq!(directory.path().system_name().as_bytes(), b"/tmp/CSF000002");
}

#[test]
fn directory_build_clears_the_umask_and_builds_missing_parents() {
    let calls = fake();
    let mut nested = Directory::with_calls(
        CadPath::new("/w/a/b/c"),
        Arc::clone(&calls) as Arc<dyn PosixFileCalls>,
    );
    nested.build(&Protection::new()).unwrap();
    assert!(!nested.failed());
    assert_eq!(calls.umask_value(), 0);
    assert_eq!(calls.mode("/w/a/b/c"), Some(0o644));
    assert!(calls.mode("/w/a/b").is_some());
    nested.build(&Protection::new()).unwrap();
    assert!(!nested.failed(), "an existing node is not an error");
    let mut empty = Directory::with_calls(CadPath::new(""), calls);
    assert_eq!(
        empty
            .build(&Protection::new())
            .unwrap_err()
            .exception_type(),
        "Standard_NumericError"
    );
}

#[test]
fn iterators_keep_readdir_order_and_split_at_the_first_dot() {
    let calls = FakePosixCalls::new()
        .with_directory("/d")
        .with_directory("/d/sub.dir")
        .with_file("/d/b.tar.gz", b"")
        .with_file("/d/a.step", b"")
        .with_order("/d", &["b.tar.gz", ".", "sub.dir", "a.step", ".."]);
    let calls: Arc<dyn PosixFileCalls> = Arc::new(calls);
    let mut files = FileIterator::with_calls(&CadPath::new("/d/"), "*", Arc::clone(&calls));
    let mut names: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    while files.more() {
        let value = files.values().path();
        names.push((
            value.name().as_bytes().to_vec(),
            value.extension().as_bytes().to_vec(),
        ));
        files.next();
    }
    assert_eq!(
        names,
        [
            (b"b".to_vec(), b".tar.gz".to_vec()),
            (b"a".to_vec(), b".step".to_vec())
        ]
    );
    let mut directories =
        DirectoryIterator::with_calls(&CadPath::new("/d/"), "*", Arc::clone(&calls));
    let mut found: Vec<Vec<u8>> = Vec::new();
    while directories.more() {
        found.push(
            directories
                .values()
                .path()
                .system_name()
                .as_bytes()
                .to_vec(),
        );
        directories.next();
    }
    assert_eq!(found, [b".".to_vec(), b"sub.dir".to_vec(), b"..".to_vec()]);
    let mut none = FileIterator::with_calls(&CadPath::new("/missing/"), "*", calls);
    assert!(!none.more());
    none.destroy();
    assert!(!none.failed() && none.error() == 0);
    assert!(!DirectoryIterator::empty().more());
}

#[test]
fn mask_matching_follows_strcmp_joker() {
    assert!(mask_matches(b"*", b""));
    assert!(mask_matches(b"*.c", b"a.c"));
    assert!(mask_matches(b"*.c", b".c"));
    assert!(
        !mask_matches(b"*x", b""),
        "a star followed by text needs a nonempty suffix"
    );
    assert!(mask_matches(b"a*b*c", b"abxbc"));
    assert!(!mask_matches(b"abc", b"ab"));
    assert!(!mask_matches(b"", b"a"));
    assert!(mask_matches(b"a**", b"a"));
    assert!(mask_matches(b"?", b"?") && !mask_matches(b"?", b"x"));
}
