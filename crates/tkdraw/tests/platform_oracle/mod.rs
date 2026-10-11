//! Host-dependent fragments of the pinned DRAW captures. Query the C/OS boundary
//! directly so a Linux capture cannot dictate Darwin loader text or ARM NaN signs.
#![allow(unsafe_code)]

use std::ffi::CStr;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows::missing_library_error;

/// Substitute only host C/loader fragments; `expected_on_host(recorded)`.
pub fn expected_on_host(recorded: &str) -> String {
    // Undo fixture checkout transport before inserting literal host loader CRLF.
    let recorded = recorded.replace("\r\n", "\n");
    let expected = recorded.replace(
        "dval 0/0 = -nan\n",
        &format!("dval 0/0 = {}\n", zero_divided_by_zero()),
    );
    const LOADER_ROW: &str = "dtryload: Loading /nonexistent/libnothing.so failed: /nonexistent/libnothing.so: cannot open shared object file: No such file or directory\n";
    if !expected.contains(LOADER_ROW) {
        return expected;
    }
    expected.replace(
        LOADER_ROW,
        &format!(
            "dtryload: Loading /nonexistent/libnothing.so failed: {}\n",
            missing_library_error()
        ),
    )
}

fn zero_divided_by_zero() -> String {
    let numerator: f64 = std::hint::black_box(0.0);
    let denominator: f64 = std::hint::black_box(0.0);
    let mut buffer: [libc::c_char; 64] = [0; 64];
    // OCCT Draw_Interpretor::Append(double) calls Sprintf("%.17g", value).
    // SAFETY: the bounded writable buffer and double match the C format.
    let count = unsafe {
        libc::snprintf(
            buffer.as_mut_ptr(),
            buffer.len(),
            c"%.17g".as_ptr(),
            numerator / denominator,
        )
    };
    assert!(
        (1..64).contains(&count),
        "NaN formatted into {count} bytes; expected 1..64"
    );
    // SAFETY: successful snprintf terminated the buffer.
    unsafe { CStr::from_ptr(buffer.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

#[cfg(unix)]
fn missing_library_error() -> String {
    // SAFETY: fixed NUL-terminated path; dlerror is copied before another loader call.
    unsafe {
        libc::dlerror();
        let library = libc::dlopen(c"/nonexistent/libnothing.so".as_ptr(), libc::RTLD_NOW);
        assert!(library.is_null(), "nonexistent library unexpectedly loaded");
        let message = libc::dlerror();
        assert!(
            !message.is_null(),
            "failed dlopen must provide a diagnostic"
        );
        CStr::from_ptr(message).to_str().unwrap().to_owned()
    }
}

#[test]
fn unrelated_capture_rows_remain_byte_exact() {
    let recorded = "dval x/y = -1.6000000000000001\nerror -nan elsewhere\n";
    assert_eq!(expected_on_host(recorded), recorded);
}

#[test]
fn host_fragments_keep_the_surrounding_command_output() {
    let expected = expected_on_host("before\r\ndval 0/0 = -nan\r\nafter\r\n");
    assert_eq!(
        expected,
        format!("before\ndval 0/0 = {}\nafter\n", zero_divided_by_zero())
    );
    assert!(!missing_library_error().is_empty());
}

#[cfg(windows)]
#[test]
fn loader_message_retains_its_literal_crlf_after_fixture_transport() {
    let recorded = "dtryload: Loading /nonexistent/libnothing.so failed: /nonexistent/libnothing.so: cannot open shared object file: No such file or directory\r\n";
    assert_eq!(
        expected_on_host(recorded),
        format!(
            "dtryload: Loading /nonexistent/libnothing.so failed: {}\n",
            missing_library_error()
        )
    );
    assert!(missing_library_error().ends_with("\r\n"));
}
