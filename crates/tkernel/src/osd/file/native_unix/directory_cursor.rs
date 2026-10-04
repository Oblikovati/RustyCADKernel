//! A single owned POSIX stream; copying each name avoids libc's reused dirent storage.
use super::{CallResult, c_path, last_errno};
use crate::osd::file::DirectoryEntries;
use std::{ffi::CStr, ptr::NonNull};

struct UnixDirectoryCursor(NonNull<libc::DIR>);
// POSIX directory streams have no thread affinity. Every read requires exclusive access;
// the owning shared cursor serializes accesses and Drop cannot race a reader.
unsafe impl Send for UnixDirectoryCursor {}

pub(super) fn open_cursor(path: &[u8]) -> CallResult<Box<dyn DirectoryEntries>> {
    let name = c_path(path);
    // SAFETY: name is a live NUL-terminated string, and the returned stream has one owner.
    let stream = unsafe { libc::opendir(name.as_ptr()) };
    NonNull::new(stream)
        .map(|stream| Box::new(UnixDirectoryCursor(stream)) as Box<dyn DirectoryEntries>)
        .ok_or_else(last_errno)
}

impl DirectoryEntries for UnixDirectoryCursor {
    fn next_name(&mut self) -> Option<Vec<u8>> {
        // SAFETY: the stream remains open and &mut self prevents concurrent readdir calls.
        let entry = unsafe { libc::readdir(self.0.as_ptr()) };
        let entry = NonNull::new(entry)?;
        // SAFETY: readdir returned a valid dirent with a NUL-terminated d_name.
        let name = unsafe { CStr::from_ptr((*entry.as_ptr()).d_name.as_ptr()) };
        Some(name.to_bytes().to_vec())
    }
}

impl Drop for UnixDirectoryCursor {
    fn drop(&mut self) {
        // SAFETY: this is the sole owning cursor and all readers have finished.
        unsafe { libc::closedir(self.0.as_ptr()) };
    }
}
