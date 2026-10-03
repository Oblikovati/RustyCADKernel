//! Owns the libc boundary used by NCollection_UtfStringTool on Unix.
#![allow(unsafe_code)]

use crate::ncollection::WideChar;
use std::sync::Mutex;

// mblen has hidden conversion state; serialize calls made by this adapter.
static MULTIBYTE_STATE: Mutex<()> = Mutex::new(());

unsafe extern "C" {
    fn mblen(text: *const libc::c_char, limit: libc::size_t) -> libc::c_int;
    fn mbstowcs(
        output: *mut libc::wchar_t,
        text: *const libc::c_char,
        limit: libc::size_t,
    ) -> libc::size_t;
}

pub(super) fn decode(text: &[u8]) -> Vec<WideChar> {
    let end = text
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(text.len());
    let mut terminated = text[..end].to_vec();
    terminated.push(0);
    let _guard = MULTIBYTE_STATE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    // The terminator bounds every read; a valid character cannot exceed MB_CUR_MAX.
    let count = unsafe { mblen(terminated.as_ptr().cast(), terminated.len()) };
    if count <= 0 {
        return Vec::new();
    }
    decode_wide(&terminated, count as usize)
}

fn decode_wide(terminated: &[u8], count: usize) -> Vec<WideChar> {
    // OCCT leaves storage uninitialized after an invalid later character. Zeroing
    // keeps Rust safe; those upstream undefined cases cannot prove behavioral parity.
    let mut wide = vec![0 as libc::wchar_t; count + 1];
    unsafe { mbstowcs(wide.as_mut_ptr(), terminated.as_ptr().cast(), count) };
    wide.into_iter().map(|unit| unit as WideChar).collect()
}

pub(super) fn encode(wide: &[WideChar], buffer: &mut [u8]) -> usize {
    let mut terminated: Vec<libc::wchar_t> =
        wide.iter().map(|unit| *unit as libc::wchar_t).collect();
    terminated.push(0);
    // Both buffers are owned, terminated, and the output bound is its allocation size.
    unsafe {
        libc::wcstombs(
            buffer.as_mut_ptr().cast(),
            terminated.as_ptr(),
            buffer.len(),
        )
    }
}
