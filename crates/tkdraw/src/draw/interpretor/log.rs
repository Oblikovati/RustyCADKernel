//! The command log: `SetDoLog`, `ResetLog`, `AddLog`, `GetLog`, `GetLogFileDescriptor`.
//!
//! The log is an anonymous temporary file. While logging is on, each command's standard
//! output and error are redirected into it at descriptor level, so it holds everything
//! the command printed, from any library.
use super::DrawInterpretor;
use crate::draw::stdio_capture::create_log_file;
use std::io::{Read, Seek, SeekFrom, Write};

impl DrawInterpretor {
    /// `SetDoLog`: enables or disables logging; the log file is created on first use.
    /// When it cannot be created the OS error is printed and logging stays off.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{DrawInterpretor, RecordingConsole};
    /// use std::rc::Rc;
    /// let di = DrawInterpretor::new(Rc::new(RecordingConsole::default()));
    /// di.set_do_log(true);
    /// di.add_log("line\n");
    /// assert_eq!(di.get_log(), "line\n");
    /// ```
    pub fn set_do_log(&self, do_log: bool) {
        if self.core.do_log.get() == do_log {
            return;
        }
        if do_log && self.core.log.borrow().is_none() {
            let Some(log) = create_log_file() else {
                return;
            };
            *self.core.log.borrow_mut() = Some(log);
        }
        self.core.do_log.set(do_log);
    }

    /// `GetLogFileDescriptor`: the log's C descriptor, `None` before the log exists
    /// (the C++ value is then -1).
    pub fn log_file_descriptor(&self) -> Option<i32> {
        self.core.log.borrow().as_ref().map(|log| log.descriptor)
    }

    /// `ResetLog`: truncates the log (if it exists) to zero size.
    pub fn reset_log(&self) {
        let mut log = self.core.log.borrow_mut();
        let Some(log) = log.as_mut() else {
            return;
        };
        self.console().flush();
        let truncated = log
            .file
            .seek(SeekFrom::Start(0))
            .and_then(|_| log.file.set_len(0));
        if let Err(error) = truncated {
            eprintln!("Error truncating the console log: {error}");
        }
    }

    /// `AddLog`: writes `text` to the log (if it exists), without a newline.
    pub fn add_log(&self, text: &str) {
        let mut log = self.core.log.borrow_mut();
        let Some(log) = log.as_mut().filter(|_| !text.is_empty()) else {
            return;
        };
        self.console().flush();
        if let Err(error) = log.file.write_all(text.as_bytes()) {
            eprintln!("Error writing to console log: {error}");
        }
    }

    /// `GetLog`: the whole content of the log; empty when there is no log.
    pub fn get_log(&self) -> String {
        let mut log = self.core.log.borrow_mut();
        let Some(log) = log.as_mut() else {
            return String::new();
        };
        self.console().flush();
        let mut bytes: Vec<u8> = Vec::new();
        // A read error ends the text, as the C++ loop stops at `read() <= 0`.
        let _ = log
            .file
            .seek(SeekFrom::Start(0))
            .and_then(|_| log.file.read_to_end(&mut bytes));
        String::from_utf8_lossy(&c_string_chunks(&bytes)).into_owned()
    }
}

/// The C++ loop reads 4096-byte chunks and appends each as a C string, so a NUL byte
/// drops the rest of its own chunk only.
fn c_string_chunks(bytes: &[u8]) -> Vec<u8> {
    let chunk_text = |chunk: &[u8]| {
        chunk
            .split(|&byte| byte == 0)
            .next()
            .unwrap_or_default()
            .to_vec()
    };
    bytes.chunks(4096).flat_map(chunk_text).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_nul_byte_cuts_only_its_chunk() {
        let mut bytes = vec![b'a'; 4096];
        bytes[10] = 0;
        bytes.extend_from_slice(b"tail");
        let text = c_string_chunks(&bytes);
        assert_eq!(text.len(), 14);
        assert!(text.ends_with(b"tail"));
    }
}
