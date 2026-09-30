//! `Message_MsgFile`: the keyword-to-text message catalog and its file loader.
use crate::ncollection::DataMap;
use crate::standard::Failure;
use crate::tcollection::{AsciiString, ExtendedString};
use std::sync::{Arc, Mutex, OnceLock};
mod loader;
mod source;
pub use source::{MsgFileSource, NativeMsgFileSource};

/// A message catalog with injected file and environment access; `MsgFile::global()`.
///
/// OCCT keeps one process-wide catalog behind static functions; [`MsgFile::global`] is that
/// catalog (with native I/O), and [`MsgFile::new`] builds an independent one, e.g. over a
/// fake file system in tests.
pub struct MsgFile {
    messages: Mutex<DataMap<AsciiString, ExtendedString>>,
    source: Arc<dyn MsgFileSource>,
}
impl MsgFile {
    /// Creates an empty catalog reading through `source`; `MsgFile::new(Arc::new(NativeMsgFileSource))`.
    pub fn new(source: Arc<dyn MsgFileSource>) -> Self {
        Self {
            messages: Mutex::new(DataMap::default()),
            source,
        }
    }
    /// The process-wide catalog used by `Msg::from_key`; `MsgFile::global().has_msg(&key)`.
    pub fn global() -> &'static MsgFile {
        static GLOBAL: OnceLock<MsgFile> = OnceLock::new();
        GLOBAL.get_or_init(|| MsgFile::new(Arc::new(NativeMsgFileSource)))
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, DataMap<AsciiString, ExtendedString>> {
        // A panicking loader cannot leave the map half-updated: every mutation is one bind.
        self.messages
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    /// Binds (or rebinds) a message; always true; `catalog.add_msg(&key, &text)`.
    pub fn add_msg(&self, key: &AsciiString, text: &ExtendedString) -> bool {
        self.lock().bind(key.clone(), text.clone());
        true
    }
    /// Tests for a bound keyword; `catalog.has_msg(&key)`.
    pub fn has_msg(&self, key: &AsciiString) -> bool {
        self.lock().is_bound(key)
    }
    /// Returns the text of a keyword; `catalog.msg(&key)`.
    ///
    /// An unknown keyword is bound on the fly to the `Message_Msg_BadKeyword` text (or
    /// "Unknown message invoked with the keyword ") followed by the keyword, as natively.
    pub fn msg(&self, key: &AsciiString) -> ExtendedString {
        let mut messages = self.lock();
        if let Ok(text) = messages.find(key) {
            return text.clone();
        }
        let prefix_key = AsciiString::new("Message_Msg_BadKeyword");
        let mut text = messages.find(&prefix_key).map_or_else(
            |_| AsciiString::new("Unknown message invoked with the keyword "),
            AsciiString::from,
        );
        text.append(key);
        let text = ExtendedString::from_ascii(&text);
        messages.bind(key.clone(), text.clone());
        text
    }
    /// `Msg(const char*)`: the keyword as C text; `catalog.msg_c_str(b"key")`.
    pub fn msg_c_str(&self, key: &[u8]) -> ExtendedString {
        self.msg(&AsciiString::from_bytes(key))
    }
}

impl MsgFile {
    /// Loads `file_name` from every directory of a space/tab/newline separated list;
    /// false when an argument is missing or any file failed; `catalog.load(Some("a b"), Some("f"))?`.
    pub fn load(
        &self,
        directories: Option<&str>,
        file_name: Option<&str>,
    ) -> Result<bool, Failure> {
        let (Some(directories), Some(file_name)) = (directories, file_name) else {
            return Ok(false);
        };
        let list = AsciiString::new(directories);
        let mut all_loaded = true;
        for index in 1.. {
            let mut path = list.token(" \t\n", index);
            if path.is_empty() {
                break;
            }
            path.assign_cat(if cfg!(windows) { "\\" } else { "/" });
            path.assign_cat(file_name);
            all_loaded &= self.load_file(Some(&String::from_utf8_lossy(path.as_bytes())))?;
        }
        Ok(all_loaded)
    }
    /// Reads and parses one message file (UTF-16 with a BOM, else 8-bit/UTF-8 text);
    /// `catalog.load_file(Some("messages.us"))?`.
    ///
    /// A line holding only `.` raises `Standard_OutOfRange` natively (from
    /// `TCollection_ExtendedString::Split`), returned as an error.
    pub fn load_file(&self, file_name: Option<&str>) -> Result<bool, Failure> {
        let Some(file_name) = file_name.filter(|name| !name.is_empty()) else {
            return Ok(false);
        };
        let Some(content) = self.source.read_file(file_name) else {
            return Ok(false);
        };
        if content.is_empty() || content.len() > i32::MAX as usize {
            return Ok(false);
        }
        loader::load_buffer(self, &content)
    }
    /// Loads `<$env/>file_name.<lang>`; the language defaults to `CSF_LANGUAGE`, then `us`;
    /// `catalog.load_from_env(Some("CSF_Msg"), "App", Some(""))?`.
    pub fn load_from_env(
        &self,
        env_name: Option<&str>,
        file_name: &str,
        lang_ext: Option<&str>,
    ) -> Result<bool, Failure> {
        let language = self.language(lang_ext);
        let mut path = file_name.to_owned();
        let directory = self.directory(env_name);
        if !directory.is_empty() {
            if !directory.ends_with('/') {
                path.insert(0, '/');
            }
            path.insert_str(0, &directory);
        }
        if !language.starts_with('.') {
            path.push('.');
        }
        path.push_str(&language);
        self.load_file(Some(&path))
    }
    /// The language extension: the argument, else `CSF_LANGUAGE`, else `us`.
    fn language(&self, lang_ext: Option<&str>) -> String {
        let language = lang_ext.unwrap_or_default().to_owned();
        if !language.is_empty() {
            return language;
        }
        let language = self.source.variable("CSF_LANGUAGE").unwrap_or_default();
        if language.is_empty() {
            "us".to_owned()
        } else {
            language
        }
    }
    /// The directory named by the environment variable, empty when unnamed or unset.
    fn directory(&self, env_name: Option<&str>) -> String {
        env_name
            .filter(|name| !name.is_empty())
            .and_then(|name| self.source.variable(name))
            .unwrap_or_default()
    }
    /// Parses messages from memory; a negative length reads up to the first NUL;
    /// `catalog.load_from_string(b".key\ntext", -1)?`.
    pub fn load_from_string(&self, content: &[u8], length: i32) -> Result<bool, Failure> {
        let size = match usize::try_from(length) {
            Ok(size) => size.min(content.len()),
            Err(_) => content
                .iter()
                .position(|&byte| byte == 0)
                .unwrap_or(content.len()),
        };
        if size == 0 {
            return Ok(false);
        }
        loader::load_bytes(self, &content[..size])
    }
}
