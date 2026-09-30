//! The process-wide current format of Resource_Unicode (SetFormat/GetFormat/ReadFormat).
use super::{FORMAT, FORMAT_LOCK, FORMAT_READ, ResourceUnicode};
use crate::{
    resource::{FormatType, ResourceHost, ResourceManager},
    standard::Failure,
};
use std::sync::atomic::Ordering;

impl ResourceUnicode {
    /// Sets the format used by the format-less conversions and marks it as read;
    /// `ResourceUnicode::set_format(FormatType::Utf8)`.
    pub fn set_format(format: FormatType) {
        let _lock = FORMAT_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        FORMAT.store(format as i32, Ordering::Relaxed);
        FORMAT_READ.store(true, Ordering::Release);
    }
    /// The current format, read on first use from the `CharSet` resources of the process
    /// environment (native `GetFormat`); `ResourceUnicode::get_format()?`.
    pub fn get_format() -> Result<FormatType, Failure> {
        Self::get_format_with(&ResourceHost::native())
    }
    /// `get_format` reading the `CharSet` resources through injected host services;
    /// `ResourceUnicode::get_format_with(&host)?`.
    pub fn get_format_with(host: &ResourceHost) -> Result<FormatType, Failure> {
        if FORMAT_READ.load(Ordering::Acquire) {
            return Ok(current());
        }
        let _lock = FORMAT_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if !FORMAT_READ.load(Ordering::Relaxed) {
            read_from_config(host)?;
            FORMAT_READ.store(true, Ordering::Release);
        }
        Ok(current())
    }
    /// Re-reads `FormatType` from the `CharSet` resources of the process environment
    /// (native `ReadFormat`); `ResourceUnicode::read_format()?`.
    pub fn read_format() -> Result<(), Failure> {
        Self::read_format_with(&ResourceHost::native())
    }
    /// `read_format` through injected host services; `ResourceUnicode::read_format_with(&host)?`.
    pub fn read_format_with(host: &ResourceHost) -> Result<(), Failure> {
        let _lock = FORMAT_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        FORMAT_READ.store(false, Ordering::Relaxed);
        let result = read_from_config(host);
        // Native marks the format read after readFormatFromConfig returns; an exception
        // escapes before that, leaving it unread.
        if result.is_ok() {
            FORMAT_READ.store(true, Ordering::Release);
        }
        result
    }
}
fn current() -> FormatType {
    FormatType::from_raw(FORMAT.load(Ordering::Relaxed)).expect("stored from a FormatType")
}
/// Native `readFormatFromConfig`: only "SJIS", "EUC" and "GB" are recognized; anything else,
/// including a missing resource, selects ANSI.
fn read_from_config(host: &ResourceHost) -> Result<(), Failure> {
    let manager = ResourceManager::new("CharSet", false, host.clone())?;
    let mut format = FormatType::ANSI;
    if manager.find("FormatType") {
        format = match manager.value("FormatType")?.as_bytes() {
            b"SJIS" => FormatType::Sjis,
            b"EUC" => FormatType::Euc,
            b"GB" => FormatType::Gb,
            _ => FormatType::ANSI,
        };
    }
    FORMAT.store(format as i32, Ordering::Relaxed);
    Ok(())
}
