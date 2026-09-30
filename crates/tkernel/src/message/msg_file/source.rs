//! File-system and environment seam of the message catalog.
use std::fs;

/// Where `MsgFile` reads files and environment variables; inject a named fake in tests.
pub trait MsgFileSource: Send + Sync {
    /// The complete file content, or `None` when it cannot be opened or read (`fopen`/`fread`
    /// failure); `source.read_file("messages.us")`.
    fn read_file(&self, path: &str) -> Option<Vec<u8>>;
    /// An environment variable (`OSD_Environment::Value`); `source.variable("CSF_LANGUAGE")`.
    fn variable(&self, name: &str) -> Option<String>;
}

/// Host file system and process environment; `Arc::new(NativeMsgFileSource)`.
pub struct NativeMsgFileSource;
impl MsgFileSource for NativeMsgFileSource {
    fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        fs::read(path).ok()
    }
    fn variable(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
}
