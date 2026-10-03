#![allow(unsafe_code)]

use libc::{c_char, locale_t};
use std::ffi::CString;
use std::marker::PhantomData;
use std::rc::Rc;

/// Owns a C locale handle and restores the locale that was active on this thread.
#[derive(Debug)]
pub(super) struct ThreadLocaleScope {
    c_locale: locale_t,
    previous_locale: locale_t,
    _thread_bound: PhantomData<Rc<()>>,
}

impl ThreadLocaleScope {
    pub(super) fn enter() -> Self {
        let c_locale = create_c_locale();
        let previous_locale = activate_c_locale(c_locale);
        Self {
            c_locale,
            previous_locale,
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
        unsafe { libc::freelocale(self.c_locale) };
    }
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
        unsafe { libc::freelocale(c_locale) };
        panic!("uselocale(C) failed: {}", std::io::Error::last_os_error());
    }
    previous
}
