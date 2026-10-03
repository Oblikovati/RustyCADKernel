#![allow(unsafe_code)]

use libc::{c_char, locale_t};
use std::ffi::CString;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::OnceLock;

/// Owns the shared C locale for process lifetime; POSIX locale objects are immutable after creation.
#[derive(Debug)]
struct SharedCLocale {
    handle: locale_t,
}

// SAFETY: the locale handle is initialized once, never mutated or freed, and POSIX permits its
// immutable locale object to be activated concurrently by separate threads.
unsafe impl Send for SharedCLocale {}
// SAFETY: same invariant as Send; access only passes the private handle to uselocale.
unsafe impl Sync for SharedCLocale {}

static C_LOCALE: OnceLock<SharedCLocale> = OnceLock::new();

/// Restores the locale that was active on this thread when the scope was entered.
#[derive(Debug)]
pub(super) struct ThreadLocaleScope {
    previous_locale: locale_t,
    _thread_bound: PhantomData<Rc<()>>,
}

impl ThreadLocaleScope {
    pub(super) fn enter() -> Self {
        Self {
            previous_locale: activate_c_locale(c_locale_handle()),
            _thread_bound: PhantomData,
        }
    }
}

impl Drop for ThreadLocaleScope {
    fn drop(&mut self) {
        let restored = unsafe { libc::uselocale(self.previous_locale) };
        debug_assert!(
            !restored.is_null(),
            "uselocale failed while restoring the prior thread locale"
        );
    }
}

fn c_locale_handle() -> locale_t {
    C_LOCALE
        .get_or_init(|| SharedCLocale {
            handle: create_c_locale(),
        })
        .handle
}

fn create_c_locale() -> locale_t {
    let name = CString::new("C").expect("static locale name contains no NUL");
    let locale = unsafe {
        libc::newlocale(
            libc::LC_ALL_MASK,
            name.as_ptr() as *const c_char,
            std::ptr::null_mut(),
        )
    };
    if locale.is_null() {
        panic!(
            "newlocale(LC_ALL_MASK, C) failed: {}",
            std::io::Error::last_os_error()
        );
    }
    locale
}

fn activate_c_locale(c_locale: locale_t) -> locale_t {
    let previous = unsafe { libc::uselocale(c_locale) };
    if previous.is_null() {
        panic!("uselocale(C) failed: {}", std::io::Error::last_os_error());
    }
    previous
}

#[cfg(test)]
mod tests {
    use super::{ThreadLocaleScope, c_locale_handle};

    #[test]
    fn nested_scopes_activate_the_same_cached_handle() {
        let outer = ThreadLocaleScope::enter();
        let first = unsafe { libc::uselocale(std::ptr::null_mut()) };
        let inner = ThreadLocaleScope::enter();
        let second = unsafe { libc::uselocale(std::ptr::null_mut()) };
        assert_eq!(first, c_locale_handle());
        assert_eq!(second, first);
        drop(inner);
        assert_eq!(unsafe { libc::uselocale(std::ptr::null_mut()) }, first);
        drop(outer);
    }
}
