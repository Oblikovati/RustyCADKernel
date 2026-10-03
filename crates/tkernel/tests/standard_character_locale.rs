#![allow(unsafe_code)]
#![cfg(unix)]
//! Checks locale-sensitive parity in a child process so global `LC_CTYPE` changes cannot race.

use rustycad_tkernel::standard::character::{
    is_alphabetic, is_alphanumeric, is_lower_case, is_printable, upper_case,
};
use std::ffi::CString;
use std::process::Command;

const CHILD_FLAG: &str = "RUSTYCAD_CHARACTER_LOCALE_CHILD";
const LATIN1_LOCALE: &str = "de_DE.ISO-8859-1";

#[test]
fn standard_character_matches_latin1_locale_in_isolated_process() {
    if std::env::var_os(CHILD_FLAG).is_some() {
        assert_latin1_behavior();
        return;
    }
    if !latin1_locale_available() {
        eprintln!("skipping locale assertions: {LATIN1_LOCALE} is unavailable");
        return;
    }
    run_child_locale_assertions();
}

fn latin1_locale_available() -> bool {
    let locale_name = CString::new(LATIN1_LOCALE).expect("locale name has no NUL");
    let locale = unsafe {
        libc::newlocale(
            libc::LC_CTYPE_MASK,
            locale_name.as_ptr(),
            std::ptr::null_mut(),
        )
    };
    if locale.is_null() {
        return false;
    }
    unsafe { libc::freelocale(locale) };
    true
}

fn run_child_locale_assertions() {
    let status = Command::new(std::env::current_exe().expect("test executable path"))
        .args([
            "--exact",
            "standard_character_matches_latin1_locale_in_isolated_process",
            "--nocapture",
        ])
        .env(CHILD_FLAG, "1")
        .status()
        .expect("spawn isolated locale test process");
    assert!(status.success(), "locale child exited with {status}");
}

fn assert_latin1_behavior() {
    let locale_name = CString::new(LATIN1_LOCALE).expect("locale name has no NUL");
    let active = unsafe { libc::setlocale(libc::LC_CTYPE, locale_name.as_ptr()) };
    assert!(!active.is_null(), "failed to activate {LATIN1_LOCALE}");
    assert!(is_alphabetic(0xE4));
    assert!(is_alphanumeric(0xE4));
    assert!(is_lower_case(0xDF));
    assert_eq!(upper_case(0xE4), 0xC4);
    assert!(is_printable(0xA0));
}
