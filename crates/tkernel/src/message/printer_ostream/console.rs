//! `Message_PrinterOStream::SetConsoleTextColor` per platform.
#![allow(unsafe_code)]
use super::ConsoleColor;
use std::io::Write;

/// ANSI escape of a color (the non-Windows branch), bold when intense.
#[cfg(not(windows))]
fn escape(color: ConsoleColor, is_intense: bool) -> &'static [u8] {
    let (normal, intense): (&[u8], &[u8]) = match color {
        ConsoleColor::Default => (b"\x1b[0m", b"\x1b[0;1m"),
        ConsoleColor::Black => (b"\x1b[30m", b"\x1b[30;1m"),
        ConsoleColor::Red => (b"\x1b[31m", b"\x1b[31;1m"),
        ConsoleColor::Green => (b"\x1b[32m", b"\x1b[32;1m"),
        ConsoleColor::Yellow => (b"\x1b[33m", b"\x1b[33;1m"),
        ConsoleColor::Blue => (b"\x1b[34m", b"\x1b[34;1m"),
        ConsoleColor::Magenta => (b"\x1b[35m", b"\x1b[35;1m"),
        ConsoleColor::Cyan => (b"\x1b[36m", b"\x1b[36;1m"),
        ConsoleColor::White => (b"\x1b[37m", b"\x1b[37;1m"),
    };
    if is_intense { intense } else { normal }
}

/// Writes the ANSI escape into the stream; nothing without a stream.
#[cfg(not(windows))]
pub(super) fn set_text_color(
    stream: Option<&mut dyn Write>,
    color: ConsoleColor,
    is_intense: bool,
) {
    if let Some(stream) = stream {
        // Stream write failures are ignored as `std::ostream <<` state bits are natively.
        let _ = stream.write_all(escape(color, is_intense));
    }
}

#[cfg(windows)]
mod win32 {
    pub(super) type Handle = *mut core::ffi::c_void;
    pub(super) const STD_OUTPUT_HANDLE: u32 = -11_i32 as u32;
    pub(super) const FOREGROUND_BLUE: u16 = 0x0001;
    pub(super) const FOREGROUND_GREEN: u16 = 0x0002;
    pub(super) const FOREGROUND_RED: u16 = 0x0004;
    pub(super) const FOREGROUND_INTENSITY: u16 = 0x0008;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub(super) fn GetStdHandle(which: u32) -> Handle;
        pub(super) fn SetConsoleTextAttribute(console: Handle, attributes: u16) -> i32;
    }
}

/// Console attribute of a color (the Windows branch).
#[cfg(windows)]
fn attributes(color: ConsoleColor, is_intense: bool) -> u16 {
    use win32::{FOREGROUND_BLUE, FOREGROUND_GREEN, FOREGROUND_INTENSITY, FOREGROUND_RED};
    let intensity = if is_intense { FOREGROUND_INTENSITY } else { 0 };
    intensity
        | match color {
            ConsoleColor::Default | ConsoleColor::White => {
                FOREGROUND_RED | FOREGROUND_GREEN | FOREGROUND_BLUE
            }
            ConsoleColor::Black => 0,
            ConsoleColor::Red => FOREGROUND_RED,
            ConsoleColor::Green => FOREGROUND_GREEN,
            ConsoleColor::Blue => FOREGROUND_BLUE,
            ConsoleColor::Yellow => FOREGROUND_RED | FOREGROUND_GREEN,
            ConsoleColor::Cyan => FOREGROUND_GREEN | FOREGROUND_BLUE,
            ConsoleColor::Magenta => FOREGROUND_RED | FOREGROUND_BLUE,
        }
}

/// Sets the attribute of the process console, ignoring the stream as natively.
#[cfg(windows)]
pub(super) fn set_text_color(
    _stream: Option<&mut dyn Write>,
    color: ConsoleColor,
    is_intense: bool,
) {
    // SAFETY: plain Win32 calls; a null or invalid handle makes them fail harmlessly.
    unsafe {
        let console = win32::GetStdHandle(win32::STD_OUTPUT_HANDLE);
        if !console.is_null() {
            win32::SetConsoleTextAttribute(console, attributes(color, is_intense));
        }
    }
}
