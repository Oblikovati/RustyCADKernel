//! Keep native bytes intact while offering the existing Rust text presentation.
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum FailureText {
    Utf8(Arc<str>),
    Bytes {
        original: Arc<[u8]>,
        display: Arc<str>,
    },
}

impl FailureText {
    pub(super) fn from_bytes(bytes: &[u8]) -> Self {
        let end = bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(bytes.len());
        let prefix = &bytes[..end];
        match std::str::from_utf8(prefix) {
            Ok(text) => Self::Utf8(Arc::from(text)),
            Err(_) => Self::Bytes {
                original: Arc::from(prefix),
                display: Arc::from(String::from_utf8_lossy(prefix).as_ref()),
            },
        }
    }

    pub(super) fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Utf8(text) => text.as_bytes(),
            Self::Bytes { original, .. } => original,
        }
    }

    pub(super) fn as_str(&self) -> &str {
        match self {
            Self::Utf8(text) => text,
            Self::Bytes { display, .. } => display,
        }
    }
}
