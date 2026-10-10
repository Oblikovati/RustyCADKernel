//! Direct Win32 loader diagnostic, independent of Martensite's loader wrapper.
use windows_sys::Win32::{
    Foundation::GetLastError,
    System::{
        Diagnostics::Debug::{
            FORMAT_MESSAGE_ARGUMENT_ARRAY, FORMAT_MESSAGE_FROM_SYSTEM, FormatMessageW,
        },
        LibraryLoader::{LOAD_WITH_ALTERED_SEARCH_PATH, LoadLibraryExW},
    },
};

pub(super) fn missing_library_error() -> String {
    let error: u32 = load_absent_library();
    let mut buffer: [u16; 1024] = [0; 1024];
    let flags: u32 = FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ARGUMENT_ARRAY;
    // OSD_SharedLibrary.cxx uses neutral-language FormatMessageW and retains CRLF.
    // SAFETY: writable buffer; the independently checked error has no inserts.
    let count = unsafe {
        FormatMessageW(
            flags,
            std::ptr::null(),
            error,
            0,
            buffer.as_mut_ptr(),
            1024,
            std::ptr::null(),
        )
    };
    assert!(
        count > 0,
        "FormatMessageW returned {count}; expected a diagnostic"
    );
    String::from_utf16(&buffer[..count as usize]).unwrap()
}

fn load_absent_library() -> u32 {
    let path: Vec<u16> = "/nonexistent/libnothing.so\0".encode_utf16().collect();
    // SAFETY: terminated UTF-16 path; last error is read immediately after the call.
    let (library, error) = unsafe {
        let library = LoadLibraryExW(
            path.as_ptr(),
            std::ptr::null_mut(),
            LOAD_WITH_ALTERED_SEARCH_PATH,
        );
        (library, GetLastError())
    };
    assert!(library.is_null(), "nonexistent library unexpectedly loaded");
    assert_eq!(
        error, 126,
        "expected ERROR_MOD_NOT_FOUND for the absent library"
    );
    error
}
