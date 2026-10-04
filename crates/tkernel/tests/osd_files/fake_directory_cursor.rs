//! Counts cursor calls independently of filesystem I/O.
use rustycad_tkernel::osd::DirectoryEntries;
use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Default)]
pub struct DirectoryCounts {
    pub opened: AtomicUsize,
    pub read: AtomicUsize,
    pub closed: AtomicUsize,
}

pub struct FakeDirectoryCursor {
    names: VecDeque<Vec<u8>>,
    counts: Option<Arc<DirectoryCounts>>,
}
impl FakeDirectoryCursor {
    pub fn new(names: Vec<Vec<u8>>, counts: Option<Arc<DirectoryCounts>>) -> Self {
        if let Some(counts) = &counts {
            counts.opened.fetch_add(1, Ordering::SeqCst);
        }
        Self {
            names: names.into(),
            counts,
        }
    }
}
impl DirectoryEntries for FakeDirectoryCursor {
    fn next_name(&mut self) -> Option<Vec<u8>> {
        if let Some(counts) = &self.counts {
            counts.read.fetch_add(1, Ordering::SeqCst);
        }
        self.names.pop_front()
    }
}
impl Drop for FakeDirectoryCursor {
    fn drop(&mut self) {
        if let Some(counts) = &self.counts {
            counts.closed.fetch_add(1, Ordering::SeqCst);
        }
    }
}
