#![cfg(all(unix, not(target_os = "android")))]
#![allow(unsafe_code)]
//! Runs locale mutation in a child process because `setlocale` changes process state.

use rustycad_tkernel::standard::c_locale_sentry::CLocaleSentry;
use rustycad_tkernel::standard::character::{is_alphabetic, upper_case};
use std::ffi::CString;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::Command;

const CHILD_FLAG: &str = "RUSTYCAD_C_LOCALE_SENTRY_CHILD";
const LATIN1_LOCALE: &str = "de_DE.ISO-8859-1";

#[test]
fn sentry_switches_and_restores_thread_locale() {
    if std::env::var_os(CHILD_FLAG).is_some() {
        assert_sentry_behavior();
        return;
    }
    if !latin1_locale_available() {
        eprintln!("skipping locale assertions: {LATIN1_LOCALE} is unavailable");
        return;
    }
    let status = Command::new(std::env::current_exe().expect("test executable path"))
        .args([
            "--exact",
            "sentry_switches_and_restores_thread_locale",
            "--nocapture",
        ])
        .env(CHILD_FLAG, "1")
        .status()
        .expect("spawn isolated locale test process");
    assert!(status.success(), "locale child exited with {status}");
}

fn latin1_locale_available() -> bool {
    let name = CString::new(LATIN1_LOCALE).expect("locale name has no NUL");
    let locale =
        unsafe { libc::newlocale(libc::LC_CTYPE_MASK, name.as_ptr(), std::ptr::null_mut()) };
    if locale.is_null() {
        return false;
    }
    unsafe { libc::freelocale(locale) };
    true
}

fn assert_sentry_behavior() {
    activate_latin1_locale();
    assert!(is_alphabetic(0xE4));
    assert_eq!(upper_case(0xE4), 0xC4);
    assert_nested_locale_restoration();
    assert_unwind_locale_restoration();
    assert_thread_locale_isolation();
    assert!(is_alphabetic(0xE4));
}

fn activate_latin1_locale() {
    let name = CString::new(LATIN1_LOCALE).expect("locale name has no NUL");
    let active = unsafe { libc::setlocale(libc::LC_CTYPE, name.as_ptr()) };
    assert!(!active.is_null(), "failed to activate {LATIN1_LOCALE}");
}

fn assert_nested_locale_restoration() {
    let outer = CLocaleSentry::new();
    assert_c_locale_behavior();
    {
        let _inner = CLocaleSentry::new();
        assert_c_locale_behavior();
    }
    assert_c_locale_behavior();
    drop(outer);
    assert_latin1_behavior();
}

fn assert_unwind_locale_restoration() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _sentry = CLocaleSentry::new();
        assert_c_locale_behavior();
        panic!("exercise sentry drop during unwinding");
    }));
    assert!(result.is_err());
    assert_latin1_behavior();
}

fn assert_thread_locale_isolation() {
    std::thread::spawn(|| {
        assert_latin1_behavior();
        {
            let _sentry = CLocaleSentry::new();
            assert_c_locale_behavior();
        }
        assert_latin1_behavior();
    })
    .join()
    .expect("thread locale assertions");
    assert_latin1_behavior();
}

fn assert_c_locale_behavior() {
    assert!(!is_alphabetic(0xE4));
    assert_eq!(upper_case(0xE4), 0xE4);
}

fn assert_latin1_behavior() {
    assert!(is_alphabetic(0xE4));
    assert_eq!(upper_case(0xE4), 0xC4);
}
