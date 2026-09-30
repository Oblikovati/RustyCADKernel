//! `Standard::StackTrace`: appends the caller's backtrace to a bounded text buffer.
//!
//! The glibc branch of Standard_StackTrace.cxx is ported with the same `backtrace()` and
//! `backtrace_symbols()` calls, so the text has the native shape
//! (`binary(symbol+0xoffset) [0xaddress]` per frame). The frames themselves are Rust
//! frames, so their count and names differ from a C++ caller's.
#![allow(unsafe_code)]

/// Header line the trace starts with (14 bytes).
const BACKTRACE_HEADER: &str = "\n==Backtrace==";
/// Footer line the trace ends with when room remains (14 bytes).
const BACKTRACE_FOOTER: &str = "\n=============";

/// Appends up to `nb_traces` frames of the calling thread's stack to `buffer`, keeping
/// its byte length below `buffer_size` (a C buffer of that size, with its NUL); the
/// frames of this function and `nb_top_skip` more are skipped. Returns false, leaving
/// `buffer` unchanged, for `buffer_size < 1`, `nb_traces < 1`, `nb_top_skip < 0`, a
/// buffer already at least `buffer_size` bytes long (C++ would overflow it), a stack
/// shallower than the skipped frames, or a platform without glibc's `backtrace`.
/// The C++ `theContext` argument is ignored on this platform, so it has no Rust parameter.
/// `stack_trace(&mut text, 2048, 10, 0)`.
pub fn stack_trace(
    buffer: &mut String,
    buffer_size: i32,
    nb_traces: i32,
    nb_top_skip: i32,
) -> bool {
    if buffer_size < 1 || nb_traces < 1 || nb_top_skip < 0 {
        return false;
    }
    let Some(limit) = (buffer_size as usize).checked_sub(buffer.len() + 1) else {
        return false;
    };
    // One more skipped frame for this function, as the C++ `theNbTopSkip + 1`.
    let Some(frames) = native_frames(nb_traces, nb_top_skip + 1) else {
        return false;
    };
    append_frames(buffer, limit, &frames);
    true
}
/// Appends the header, every frame that fits and the footer, as the C++ `strcat` loop.
fn append_frames(buffer: &mut String, mut limit: usize, frames: &[String]) {
    if limit > BACKTRACE_HEADER.len() {
        buffer.push_str(BACKTRACE_HEADER);
        limit -= BACKTRACE_HEADER.len();
    }
    for frame in frames {
        if frame.len() + 1 >= limit {
            break;
        }
        buffer.push('\n');
        buffer.push_str(frame);
        limit -= frame.len() + 1;
    }
    if limit > BACKTRACE_FOOTER.len() {
        buffer.push_str(BACKTRACE_FOOTER);
    }
}
/// Symbolized frames below the `top_skip` innermost ones, or `None` as the C++ returns false.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
#[inline(never)]
fn native_frames(nb_traces: i32, top_skip: i32) -> Option<Vec<String>> {
    let capacity: i32 = nb_traces.checked_add(top_skip)?;
    let mut addresses: Vec<*mut libc::c_void> = vec![std::ptr::null_mut(); capacity as usize];
    // SAFETY: `addresses` holds `capacity` writable pointer slots.
    let captured: i32 = unsafe { libc::backtrace(addresses.as_mut_ptr(), capacity) };
    if captured <= 1 {
        return None;
    }
    let count: i32 = captured - top_skip;
    if count < 0 {
        // glibc cannot allocate a negative symbol table and returns NULL.
        return None;
    }
    symbolize(&addresses[top_skip as usize..], count)
}
/// `backtrace_symbols` of the first `count` addresses, copied out and freed.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn symbolize(addresses: &[*mut libc::c_void], count: i32) -> Option<Vec<String>> {
    // SAFETY: `addresses` holds at least `count` captured return addresses.
    let symbols: *mut *mut libc::c_char =
        unsafe { libc::backtrace_symbols(addresses.as_ptr(), count) };
    if symbols.is_null() {
        return None;
    }
    let lines: Vec<String> = (0..count as usize)
        .map(|index| {
            // SAFETY: glibc returns `count` valid NUL-terminated strings in one block.
            let line = unsafe { std::ffi::CStr::from_ptr(*symbols.add(index)) };
            line.to_string_lossy().into_owned()
        })
        .collect();
    // SAFETY: the table is one malloc'd block that the caller must free, as the C++ does.
    unsafe { libc::free(symbols.cast()) };
    Some(lines)
}
/// Platforms without glibc's `backtrace`: no trace, as the C++ unsupported-platform branches.
#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
fn native_frames(_nb_traces: i32, _top_skip: i32) -> Option<Vec<String>> {
    None
}
