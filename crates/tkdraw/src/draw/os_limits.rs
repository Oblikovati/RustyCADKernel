//! The C library calls of `cpulimit` and `dlocale`: `setrlimit(RLIMIT_CPU)`, the
//! `SIGXCPU` handler and `setlocale`. Each unsafe block names its contract.
#![allow(unsafe_code)]

use std::ffi::{CStr, CString, c_int};
use std::sync::atomic::{AtomicI64, Ordering};

/// `CPU_LIMIT`: the limit in seconds the handler reports (-1 for none).
static CPU_LIMIT: AtomicI64 = AtomicI64::new(0);

/// Stores the limit reported by the `SIGXCPU` handler and the elapsed-time watchdog.
pub(crate) fn set_reported_cpu_limit(seconds: i64) {
    CPU_LIMIT.store(seconds, Ordering::Relaxed);
}

/// `CPU_LIMIT`.
pub(crate) fn reported_cpu_limit() -> i64 {
    CPU_LIMIT.load(Ordering::Relaxed)
}

/// `setrlimit(RLIMIT_CPU, {limit, RLIM_INFINITY})`; `None` (infinite) when `limit` is
/// `None`. Returns the C status (0 on success).
#[cfg(unix)]
pub(crate) fn set_cpu_rlimit(limit: Option<u64>) -> i32 {
    let limits = libc::rlimit {
        rlim_cur: limit.map_or(libc::RLIM_INFINITY, |seconds| seconds as libc::rlim_t),
        rlim_max: libc::RLIM_INFINITY,
    };
    // SAFETY: a valid rlimit structure; setrlimit only reads it.
    unsafe { libc::setrlimit(libc::RLIMIT_CPU, &limits) }
}

/// The `SIGXCPU` handler: reports the limit and ends the process with status 2. The C++
/// handler uses `std::cout` and `exit`; here only async-signal-safe calls are made
/// (`write`, `_exit`).
#[cfg(unix)]
extern "C" fn cpu_limit_handler(_signal: c_int) {
    let mut buffer = [0u8; 64];
    let length = limit_message(reported_cpu_limit(), &mut buffer);
    // SAFETY: write and _exit are async-signal-safe; the buffer is valid for `length`.
    unsafe {
        libc::write(1, buffer.as_ptr().cast(), length);
        libc::_exit(2);
    }
}

/// Writes `Process killed by CPU limit  (N sec)\n` into `buffer` without allocating
/// (the handler may not allocate); returns the length.
#[cfg(any(unix, test))]
fn limit_message(limit: i64, buffer: &mut [u8; 64]) -> usize {
    let mut length = 0;
    let mut push = |bytes: &[u8]| {
        let end = (length + bytes.len()).min(buffer.len());
        buffer[length..end].copy_from_slice(&bytes[..end - length]);
        length = end;
    };
    push(b"Process killed by CPU limit  (");
    if limit < 0 {
        push(b"-");
    }
    let mut digits = [0u8; 20];
    let (mut value, mut count) = (limit.unsigned_abs(), 0);
    loop {
        digits[19 - count] = b'0' + (value % 10) as u8;
        count += 1;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    push(&digits[20 - count..]);
    push(b" sec)\n");
    length
}

/// Installs the `SIGXCPU` handler (`sigaction`).
#[cfg(unix)]
pub(crate) fn install_cpu_limit_handler() {
    // SAFETY: a zeroed sigaction is a valid "no flags, empty mask" value.
    let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
    action.sa_sigaction = cpu_limit_handler as extern "C" fn(c_int) as usize;
    // SAFETY: valid action; the old action is not requested.
    unsafe { libc::sigaction(libc::SIGXCPU, &action, std::ptr::null_mut()) };
}

/// `setlocale(category, locale)`: the resulting locale name, `None` on refusal.
pub(crate) fn set_locale(category: c_int, locale: Option<&str>) -> Option<String> {
    let locale = locale
        .map(|text| CString::new(text.split('\0').next().unwrap_or_default()).unwrap_or_default());
    let pointer = locale
        .as_ref()
        .map_or(std::ptr::null(), |text| text.as_ptr());
    // SAFETY: a null pointer queries; otherwise a live C string; the result is copied
    // at once (it is a static buffer overwritten by the next call).
    unsafe {
        let result = libc::setlocale(category, pointer);
        (!result.is_null()).then(|| CStr::from_ptr(result).to_string_lossy().into_owned())
    }
}

/// The `LC_*` category named `name`.
pub(crate) fn locale_category(name: &str) -> Option<c_int> {
    match name {
        "LC_ALL" => Some(libc::LC_ALL),
        "LC_COLLATE" => Some(libc::LC_COLLATE),
        "LC_CTYPE" => Some(libc::LC_CTYPE),
        "LC_MONETARY" => Some(libc::LC_MONETARY),
        "LC_NUMERIC" => Some(libc::LC_NUMERIC),
        "LC_TIME" => Some(libc::LC_TIME),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limit_message_is_built_without_allocation() {
        let mut buffer = [0u8; 64];
        let length = limit_message(-1, &mut buffer);
        assert_eq!(
            &buffer[..length],
            b"Process killed by CPU limit  (-1 sec)\n"
        );
        let length = limit_message(120, &mut buffer);
        assert_eq!(
            &buffer[..length],
            b"Process killed by CPU limit  (120 sec)\n"
        );
    }

    #[test]
    fn the_locale_can_be_queried() {
        assert!(set_locale(libc::LC_NUMERIC, None).is_some());
        assert_eq!(locale_category("LC_TIME"), Some(libc::LC_TIME));
        assert_eq!(locale_category("LC_BAD"), None);
    }
}
