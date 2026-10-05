//! Source-ordered Windows loader calls, separately injectable for Linux tests.
use crate::{
    osd::{CadPath, PathSyntax},
    tcollection::{AsciiString, ExtendedString},
};
use std::{ffi::c_void, sync::atomic::AtomicU32};

pub(super) trait WindowsCalls {
    fn module_handle(&self, name: &[u16]) -> *mut c_void;
    fn load_library(&self, path: &[u16]) -> *mut c_void;
    fn symbol(&self, handle: *mut c_void, name: &[u8]) -> *mut c_void;
    fn close(&self, handle: *mut c_void);
    fn last_error_code(&self) -> u32;
    fn format_error(&self, code: u32, name: Option<&[u8]>) -> Vec<u8>;
}

pub(super) fn module_cache_with(calls: &impl WindowsCalls, path: &[u8]) -> *mut c_void {
    calls.module_handle(&module_basename_wide(path))
}

pub(super) fn open_with(
    calls: &impl WindowsCalls,
    name: Option<&[u8]>,
    current: *mut c_void,
    last_error: &AtomicU32,
) -> *mut c_void {
    if !current.is_null() {
        return current;
    }
    open_uncached(calls, name.unwrap_or_default(), last_error)
}

fn open_uncached(calls: &impl WindowsCalls, name: &[u8], last_error: &AtomicU32) -> *mut c_void {
    let handle = calls.load_library(&library_name_wide(name));
    if handle.is_null() {
        last_error.store(
            calls.last_error_code(),
            std::sync::atomic::Ordering::Relaxed,
        );
    }
    handle
}

pub(super) fn symbol_with(
    calls: &impl WindowsCalls,
    handle: *mut c_void,
    name: &[u8],
    last_error: &AtomicU32,
) -> *mut c_void {
    let address = calls.symbol(handle, crate::standard::c_string::c_str_prefix(name));
    if address.is_null() {
        last_error.store(
            calls.last_error_code(),
            std::sync::atomic::Ordering::Relaxed,
        );
    }
    address
}

pub(super) fn close_with(calls: &impl WindowsCalls, handle: *mut c_void) {
    calls.close(handle);
}

pub(super) fn error_with(
    calls: &impl WindowsCalls,
    last_error: &AtomicU32,
    name: Option<&[u8]>,
) -> Vec<u8> {
    calls.format_error(last_error.load(std::sync::atomic::Ordering::Relaxed), name)
}

fn module_basename_wide(path: &[u8]) -> Vec<u16> {
    let components = CadPath::with_syntax(path, PathSyntax::Windows).values();
    let filename = [components.name.as_bytes(), components.extension.as_bytes()].concat();
    ascii_name_wide(&filename)
}

fn library_name_wide(name: &[u8]) -> Vec<u16> {
    let extended = ExtendedString::from_c_string(name, true);
    extended.as_utf16_with_nul().to_vec()
}

fn ascii_name_wide(name: &[u8]) -> Vec<u16> {
    let ascii = AsciiString::from_raw(name);
    ExtendedString::from_ascii(&ascii)
        .as_utf16_with_nul()
        .to_vec()
}

#[cfg(windows)]
pub(super) struct SystemWindowsCalls;

#[cfg(windows)]
#[allow(unsafe_code)]
impl WindowsCalls for SystemWindowsCalls {
    fn module_handle(&self, name: &[u16]) -> *mut c_void {
        unsafe { system::get_module_handle(name) }
    }

    fn load_library(&self, path: &[u16]) -> *mut c_void {
        unsafe { system::load_library(path) }
    }

    fn symbol(&self, handle: *mut c_void, name: &[u8]) -> *mut c_void {
        unsafe { system::symbol(handle, name) }
    }

    fn close(&self, handle: *mut c_void) {
        unsafe { system::close(handle) }
    }

    fn last_error_code(&self) -> u32 {
        unsafe { system::last_error_code() }
    }

    fn format_error(&self, code: u32, name: Option<&[u8]>) -> Vec<u8> {
        system::format_error(code, name)
    }
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod system {
    use std::ffi::{CString, c_void};
    use windows_sys::Win32::{
        Foundation::{FreeLibrary, GetLastError, HMODULE},
        Globalization::{CP_UTF8, WideCharToMultiByte},
        System::{
            Diagnostics::Debug::{
                FORMAT_MESSAGE_ARGUMENT_ARRAY, FORMAT_MESSAGE_FROM_SYSTEM, FormatMessageW,
            },
            LibraryLoader::{
                GetModuleHandleW, GetProcAddress, LOAD_WITH_ALTERED_SEARCH_PATH, LoadLibraryExW,
            },
        },
    };

    pub unsafe fn get_module_handle(name: &[u16]) -> *mut c_void {
        unsafe { GetModuleHandleW(name.as_ptr()) }
    }

    pub unsafe fn load_library(path: &[u16]) -> *mut c_void {
        unsafe {
            LoadLibraryExW(
                path.as_ptr(),
                std::ptr::null_mut(),
                LOAD_WITH_ALTERED_SEARCH_PATH,
            )
        }
    }

    pub unsafe fn symbol(handle: *mut c_void, name: &[u8]) -> *mut c_void {
        let Ok(name) = CString::new(name) else {
            return std::ptr::null_mut();
        };
        unsafe { GetProcAddress(handle as HMODULE, name.as_ptr().cast()) }
            .map_or(std::ptr::null_mut(), |function| {
                function as *const () as *mut c_void
            })
    }

    pub unsafe fn close(handle: *mut c_void) {
        unsafe { FreeLibrary(handle as HMODULE) };
    }

    pub unsafe fn last_error_code() -> u32 {
        unsafe { GetLastError() }
    }

    pub fn format_error(code: u32, name: Option<&[u8]>) -> Vec<u8> {
        let name = name.and_then(|bytes| CString::new(bytes).ok());
        let mut argument = name.as_ref().map_or(std::ptr::null(), |text| text.as_ptr());
        let arguments = &mut argument as *mut *const i8 as *const *const i8;
        let wide = formatted_message(code, arguments);
        to_utf8(&wide)
    }

    fn formatted_message(code: u32, arguments: *const *const i8) -> Vec<u16> {
        let mut buffer: Vec<u16> = vec![0; 1024];
        let written = unsafe {
            FormatMessageW(
                FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ARGUMENT_ARRAY,
                std::ptr::null(),
                code,
                0,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                arguments,
            )
        } as usize;
        buffer.truncate(written.saturating_add(1).min(buffer.len()));
        buffer
    }

    fn to_utf8(wide: &[u16]) -> Vec<u8> {
        if wide.is_empty() {
            return Vec::new();
        }
        let mut buffer: Vec<u8> = vec![0; 1024];
        let written: i32 = convert_wide(wide, &mut buffer);
        if written <= 0 {
            return Vec::new();
        }
        buffer.truncate(written as usize - 1);
        buffer
    }

    fn convert_wide(wide: &[u16], buffer: &mut [u8]) -> i32 {
        unsafe {
            WideCharToMultiByte(
                CP_UTF8,
                0,
                wide.as_ptr(),
                -1,
                buffer.as_mut_ptr(),
                buffer.len() as i32,
                std::ptr::null(),
                std::ptr::null_mut(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{WindowsCalls, close_with, error_with, module_cache_with, open_with, symbol_with};
    use std::{cell::RefCell, ffi::c_void, sync::atomic::AtomicU32};

    #[derive(Default)]
    struct FakeWindowsCalls {
        cache: *mut c_void,
        load: *mut c_void,
        symbol: *mut c_void,
        error: u32,
        module_names: RefCell<Vec<Vec<u16>>>,
        load_names: RefCell<Vec<Vec<u16>>>,
        symbols: RefCell<Vec<Vec<u8>>>,
        closes: RefCell<Vec<usize>>,
        formatted: RefCell<Vec<(u32, Option<Vec<u8>>)>>,
    }

    impl WindowsCalls for FakeWindowsCalls {
        fn module_handle(&self, name: &[u16]) -> *mut c_void {
            self.module_names.borrow_mut().push(name.to_vec());
            self.cache
        }
        fn load_library(&self, path: &[u16]) -> *mut c_void {
            self.load_names.borrow_mut().push(path.to_vec());
            self.load
        }
        fn symbol(&self, _handle: *mut c_void, name: &[u8]) -> *mut c_void {
            self.symbols.borrow_mut().push(name.to_vec());
            self.symbol
        }
        fn close(&self, handle: *mut c_void) {
            self.closes.borrow_mut().push(handle as usize);
        }
        fn last_error_code(&self) -> u32 {
            self.error
        }
        fn format_error(&self, code: u32, name: Option<&[u8]>) -> Vec<u8> {
            self.formatted
                .borrow_mut()
                .push((code, name.map(<[u8]>::to_vec)));
            format!("win32:{code}").into_bytes()
        }
    }

    #[test]
    fn set_name_uses_windows_path_basename_for_module_cache() {
        let calls: FakeWindowsCalls = FakeWindowsCalls::default();
        module_cache_with(&calls, b"C:\\Windows\\System32\\kernel32.dll");
        assert_eq!(
            calls.module_names.borrow()[0],
            "kernel32.dll\0".encode_utf16().collect::<Vec<_>>()
        );
    }

    #[test]
    fn opening_reuses_cached_module_before_loading() {
        let cached: *mut c_void = 5_usize as *mut c_void;
        let calls: FakeWindowsCalls = FakeWindowsCalls {
            cache: cached,
            ..Default::default()
        };
        let result = open_with(&calls, Some(b"new.dll"), cached, &AtomicU32::new(0));
        assert_eq!(result, cached);
        assert!(calls.load_names.borrow().is_empty());
    }

    #[test]
    fn uncached_open_converts_path_and_returns_loaded_handle() {
        let loaded: *mut c_void = 9_usize as *mut c_void;
        let calls: FakeWindowsCalls = FakeWindowsCalls {
            load: loaded,
            ..Default::default()
        };
        let handle = open_with(
            &calls,
            Some("C:\\CAD\\插件.dll".as_bytes()),
            std::ptr::null_mut(),
            &AtomicU32::new(0),
        );
        assert_eq!(handle, loaded);
        assert_eq!(
            calls.load_names.borrow()[0],
            "C:\\CAD\\插件.dll\0".encode_utf16().collect::<Vec<_>>()
        );
    }

    #[test]
    fn failed_open_retains_native_error_code_and_formats_name() {
        let calls: FakeWindowsCalls = FakeWindowsCalls {
            error: 126,
            ..Default::default()
        };
        let last_error: AtomicU32 = AtomicU32::new(0);
        assert!(
            open_with(
                &calls,
                Some(b"missing.dll"),
                std::ptr::null_mut(),
                &last_error
            )
            .is_null()
        );
        assert_error_details(&calls, &last_error, b"missing.dll", 126);
    }

    fn assert_error_details(
        calls: &FakeWindowsCalls,
        last_error: &AtomicU32,
        name: &[u8],
        code: u32,
    ) {
        let expected: String = format!("win32:{code}");
        assert_eq!(
            error_with(calls, last_error, Some(name)),
            expected.as_bytes()
        );
        assert_eq!(calls.formatted.borrow()[0], (code, Some(name.to_vec())));
        assert_eq!(last_error.load(std::sync::atomic::Ordering::Relaxed), code);
    }

    #[test]
    fn failed_symbol_uses_c_prefix_and_records_native_error() {
        let calls: FakeWindowsCalls = FakeWindowsCalls {
            error: 127,
            ..Default::default()
        };
        let last_error: AtomicU32 = AtomicU32::new(0);
        let result: *mut c_void =
            symbol_with(&calls, std::ptr::null_mut(), b"bad\0suffix", &last_error);
        assert!(result.is_null());
        assert_eq!(calls.symbols.borrow()[0], b"bad");
        assert_eq!(last_error.load(std::sync::atomic::Ordering::Relaxed), 127);
    }

    #[test]
    fn successful_load_does_not_clear_the_previous_windows_error_code() {
        let last_error: AtomicU32 = AtomicU32::new(126);
        let loaded: *mut c_void = 11_usize as *mut c_void;
        let calls: FakeWindowsCalls = FakeWindowsCalls {
            load: loaded,
            error: 0,
            ..Default::default()
        };
        let result: *mut c_void = open_with(
            &calls,
            Some(b"present.dll"),
            std::ptr::null_mut(),
            &last_error,
        );
        assert_eq!(result, loaded);
        assert_eq!(
            error_with(&calls, &last_error, Some(b"present.dll")),
            b"win32:126"
        );
    }

    #[test]
    fn close_forwards_the_cached_or_loaded_handle() {
        let calls: FakeWindowsCalls = FakeWindowsCalls::default();
        let handle: *mut c_void = 7_usize as *mut c_void;
        close_with(&calls, handle);
        assert_eq!(calls.closes.borrow().as_slice(), &[7]);
    }
}
