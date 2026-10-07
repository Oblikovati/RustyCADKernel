//! Descriptor-level redirection of the standard streams into the DRAW log file
//! (`capture_start`/`capture_end` and the temporary log of `Draw_Interpretor.cxx`).
//!
//! The C++ code redirects descriptors 1 and 2 with `dup`/`dup2`, so everything any
//! library prints while a command runs lands in the log, not only DRAW's own output.
//! That needs the C library; this module is the only unsafe code of the crate, and each
//! block names the descriptor contract it relies on.
#![allow(unsafe_code)]

use std::ffi::CStr;
use std::fs::File;

/// Descriptor of the standard output stream (`STDOUT_FILENO`).
pub(crate) const STDOUT_DESCRIPTOR: i32 = 1;
/// Descriptor of the standard error stream (`STDERR_FILENO`).
pub(crate) const STDERR_DESCRIPTOR: i32 = 2;

/// `perror(message)`: the message, ": ", and the C library's text for `errno`.
fn report_os_error(message: &CStr) {
    // SAFETY: a live NUL-terminated string; perror only reads it and errno.
    unsafe { libc::perror(message.as_ptr()) };
}

/// `capture_start`: duplicates `standard` (to restore it later) and points it at `log`;
/// returns the saved descriptor, or `None` after printing the OS error, as C++ does.
pub(crate) fn capture_start(standard: i32, log: i32) -> Option<i32> {
    // SAFETY: dup only reads the descriptor table.
    let saved = unsafe { libc::dup(standard) };
    if saved < 0 {
        report_os_error(c"Error capturing standard stream to log: dup() returned");
        return None;
    }
    // SAFETY: both descriptors are open; dup2 atomically re-targets `standard`.
    if unsafe { libc::dup2(log, standard) } < 0 {
        // SAFETY: `saved` is the descriptor dup just created and owned here.
        unsafe { libc::close(saved) };
        report_os_error(c"Error capturing standard stream to log: dup2() returned");
        return None;
    }
    Some(saved)
}

/// `capture_end`: restores `standard` from `saved` and closes the saved copy.
pub(crate) fn capture_end(standard: i32, saved: &mut Option<i32>) {
    let Some(descriptor) = *saved else {
        return;
    };
    // SAFETY: `descriptor` is the open copy made by `capture_start`.
    if unsafe { libc::dup2(descriptor, standard) } < 0 {
        report_os_error(c"Error returning capturing standard stream to log: dup2() returned");
        return;
    }
    // SAFETY: the saved copy is owned here and closed exactly once.
    unsafe { libc::close(descriptor) };
    *saved = None;
}

/// The temporary log file and its C descriptor (`myFDLog`).
#[derive(Debug)]
pub(crate) struct LogFile {
    /// The open file; closing it deletes it.
    pub(crate) file: File,
    /// The descriptor `dup2` redirects the standard streams to.
    pub(crate) descriptor: i32,
}

/// Creates the anonymous temporary log file: `mkstemp("/tmp/occt_draw_XXXXXX")` then
/// `unlink`, so the file disappears when closed (Linux FHS 3.17 puts temporaries in
/// /tmp, as the C++ comment says). On failure prints the OS error and the path, as C++.
#[cfg(unix)]
pub(crate) fn create_log_file() -> Option<LogFile> {
    use std::os::fd::FromRawFd;
    let mut template = *b"/tmp/occt_draw_XXXXXX\0";
    // SAFETY: a writable NUL-terminated template that mkstemp edits in place.
    let descriptor = unsafe { libc::mkstemp(template.as_mut_ptr().cast()) };
    if descriptor < 0 {
        report_os_error(c"Error creating temporary file for capturing console output");
        println!(
            "path: {}",
            String::from_utf8_lossy(&template[..template.len() - 1])
        );
        return None;
    }
    // SAFETY: the template now names the file just created; unlink only reads it.
    unsafe { libc::unlink(template.as_ptr().cast()) };
    // SAFETY: mkstemp returned a fresh descriptor that nothing else owns.
    let file = unsafe { File::from_raw_fd(descriptor) };
    Some(LogFile { file, descriptor })
}

/// Windows has no `mkstemp`; the C++ code opens a `tmpnam` name with `O_TEMPORARY`, which
/// a create-new open in the temporary directory reproduces; the CRT descriptor that
/// `_dup2` needs is made once from the file handle.
#[cfg(not(unix))]
pub(crate) fn create_log_file() -> Option<LogFile> {
    use std::os::windows::io::AsRawHandle;
    let path = std::env::temp_dir().join(format!("occt_draw_{}", std::process::id()));
    let opened = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path);
    let file = match opened {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error creating temporary file for capturing console output: {error}");
            println!("path: {}", path.display());
            return None;
        }
    };
    // SAFETY: the handle is open and owned by `file`, which lives as long as the descriptor.
    let descriptor = unsafe { libc::open_osfhandle(file.as_raw_handle() as libc::intptr_t, 0) };
    Some(LogFile { file, descriptor })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::io::{Read, Seek, SeekFrom};

    #[test]
    fn the_log_file_is_anonymous_and_writable() {
        let mut log = create_log_file().expect("temporary log");
        std::io::Write::write_all(&mut log.file, b"abc").unwrap();
        log.file.seek(SeekFrom::Start(0)).unwrap();
        let mut text = String::new();
        log.file.read_to_string(&mut text).unwrap();
        assert_eq!(text, "abc");
        assert!(log.descriptor > STDERR_DESCRIPTOR);
    }

    #[test]
    fn capture_end_without_a_saved_descriptor_does_nothing() {
        let mut saved = None;
        capture_end(STDOUT_DESCRIPTOR, &mut saved);
        assert_eq!(saved, None);
    }
}
