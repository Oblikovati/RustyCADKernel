//! Regression tests for demand-driven POSIX directory consumption and ownership.
use super::fake_calls::{FakePosixCalls, fake_directory_cursor::DirectoryCounts};
use rustycad_tkernel::osd::{CadPath, DirectoryIterator, FileIterator};
use std::sync::{Arc, atomic::Ordering};

fn observed_calls() -> (Arc<FakePosixCalls>, Arc<DirectoryCounts>) {
    let counts = Arc::new(DirectoryCounts::default());
    let mut calls = FakePosixCalls::new()
        .with_directory("/d")
        .with_file("/d/a", b"")
        .with_file("/d/b", b"")
        .with_directory("/d/sub")
        .with_directory("/d/other")
        .with_order("/d", &["a", "sub", "b", "other"]);
    calls.directory_counts = Some(Arc::clone(&counts));
    (Arc::new(calls), counts)
}

#[test]
fn file_cursor_reads_only_through_the_selected_entry() {
    let (calls, counts) = observed_calls();
    let mut files = FileIterator::with_calls(&CadPath::new("/d/"), "*", calls);
    assert_eq!(counts.opened.load(Ordering::SeqCst), 0);
    assert!(files.more());
    assert_eq!(counts.read.load(Ordering::SeqCst), 1);
    assert!(files.more());
    assert_eq!(counts.read.load(Ordering::SeqCst), 1);
    files.next();
    assert_eq!(counts.read.load(Ordering::SeqCst), 3);
    assert_eq!(files.values().path().name().as_bytes(), b"b");
    files.next();
    assert!(!files.more());
    assert_eq!(counts.read.load(Ordering::SeqCst), 5);
    assert_eq!(counts.closed.load(Ordering::SeqCst), 1);
}

#[test]
fn directory_initialize_closes_then_reopens_on_more() {
    let (calls, counts) = observed_calls();
    let mut directories = DirectoryIterator::with_calls(&CadPath::new("/d/"), "*", calls);
    assert!(directories.more());
    assert_eq!(counts.read.load(Ordering::SeqCst), 2);
    directories.destroy();
    assert_eq!(counts.closed.load(Ordering::SeqCst), 0);
    directories.initialize(&CadPath::new("/d/"), "other");
    assert_eq!(counts.closed.load(Ordering::SeqCst), 1);
    assert_eq!(counts.opened.load(Ordering::SeqCst), 1);
    assert!(directories.more());
    assert_eq!(counts.read.load(Ordering::SeqCst), 6);
    drop(directories);
    assert_eq!(counts.closed.load(Ordering::SeqCst), 2);
}

#[test]
fn copied_iterator_shares_cursor_without_double_close() {
    let (calls, counts) = observed_calls();
    let mut first = FileIterator::with_calls(&CadPath::new("/d/"), "*", calls);
    assert!(first.more());
    let mut copied = first.clone();
    copied.next();
    assert_eq!(copied.values().path().name().as_bytes(), b"b");
    assert_eq!(first.values().path().name().as_bytes(), b"a");
    drop(first);
    assert_eq!(counts.closed.load(Ordering::SeqCst), 0);
    drop(copied);
    assert_eq!(counts.closed.load(Ordering::SeqCst), 1);
}
