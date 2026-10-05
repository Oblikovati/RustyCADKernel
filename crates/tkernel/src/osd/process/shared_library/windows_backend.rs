//! Source-ordered Windows loader calls, separately injectable for Linux tests.
use crate::{
    osd::{CadPath, PathSyntax},
    tcollection::{AsciiString, ExtendedString},
};
use std::{
    ffi::c_void,
    sync::{Mutex, atomic::AtomicU32},
};

pub(super) trait WindowsCalls {
    fn module_handle(&self, name: &[u16]) -> *mut c_void;
    fn load_library(&self, path: &[u16]) -> *mut c_void;
    fn symbol(&self, handle: *mut c_void, name: &[u8]) -> *mut c_void;
    fn close(&self, handle: *mut c_void);
    fn last_error_code(&self) -> u32;
    fn format_message(&self, code: u32, name: Option<&[u8]>, target: &mut [u16]) -> bool;
    fn convert_message(&self, wide: &[u16], target: &mut [u8]) -> bool;
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
    buffers: &Mutex<WindowsErrorBuffers>,
    name: Option<&[u8]>,
) -> Vec<u8> {
    let mut current = lock_error_buffers(buffers);
    update_wide_buffer(calls, &mut current, last_error, name);
    update_utf8_buffer(calls, &mut current);
    buffer_content(&current.utf8).to_vec()
}

fn lock_error_buffers(
    buffers: &Mutex<WindowsErrorBuffers>,
) -> std::sync::MutexGuard<'_, WindowsErrorBuffers> {
    buffers
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn update_wide_buffer(
    calls: &impl WindowsCalls,
    buffers: &mut WindowsErrorBuffers,
    last_error: &AtomicU32,
    name: Option<&[u8]>,
) {
    let mut attempt: [u16; 1024] = buffers.wide;
    let code: u32 = last_error.load(std::sync::atomic::Ordering::Relaxed);
    if calls.format_message(code, name, &mut attempt) {
        buffers.wide = attempt;
    }
}

fn update_utf8_buffer(calls: &impl WindowsCalls, buffers: &mut WindowsErrorBuffers) {
    let mut attempt: [u8; 1024] = buffers.utf8;
    if calls.convert_message(&buffers.wide, &mut attempt) {
        buffers.utf8 = attempt;
    }
}

pub(super) struct WindowsErrorBuffers {
    wide: [u16; 1024],
    utf8: [u8; 1024],
}

impl WindowsErrorBuffers {
    pub(super) const fn new() -> Self {
        Self {
            wide: [0; 1024],
            utf8: [0; 1024],
        }
    }
}

fn buffer_content(buffer: &[u8]) -> &[u8] {
    let end: usize = buffer
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(buffer.len());
    &buffer[..end]
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

    fn format_message(&self, code: u32, name: Option<&[u8]>, target: &mut [u16]) -> bool {
        system::format_message(code, name, target)
    }

    fn convert_message(&self, wide: &[u16], target: &mut [u8]) -> bool {
        system::convert_message(wide, target)
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

    pub fn format_message(code: u32, name: Option<&[u8]>, target: &mut [u16]) -> bool {
        let name = name.and_then(|bytes| CString::new(bytes).ok());
        let mut argument = name.as_ref().map_or(std::ptr::null(), |text| text.as_ptr());
        let arguments = &mut argument as *mut *const i8 as *const *const i8;
        let written: u32 = unsafe {
            FormatMessageW(
                FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ARGUMENT_ARRAY,
                std::ptr::null(),
                code,
                0,
                target.as_mut_ptr(),
                target.len() as u32,
                arguments,
            )
        };
        written > 0
    }

    pub fn convert_message(wide: &[u16], target: &mut [u8]) -> bool {
        let written: i32 = unsafe {
            WideCharToMultiByte(
                CP_UTF8,
                0,
                wide.as_ptr(),
                -1,
                target.as_mut_ptr(),
                target.len() as i32,
                std::ptr::null(),
                std::ptr::null_mut(),
            )
        };
        written > 0
    }
}

#[cfg(test)]
mod tests;
