use super::{
    WindowsCalls, WindowsErrorBuffers, close_with, error_with, module_cache_with, open_with,
    symbol_with,
};
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    sync::{Mutex, atomic::AtomicU32},
};

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
    format_override: RefCell<Option<Vec<u16>>>,
    format_fails: Cell<bool>,
    convert_fails: Cell<bool>,
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
    fn format_message(&self, code: u32, name: Option<&[u8]>, target: &mut [u16]) -> bool {
        self.formatted
            .borrow_mut()
            .push((code, name.map(<[u8]>::to_vec)));
        let message: Vec<u16> = self
            .format_override
            .borrow()
            .clone()
            .unwrap_or_else(|| format!("win32:{code}").encode_utf16().collect());
        let fits: bool = write_wide_buffer(target, &message);
        fits && !self.format_fails.get()
    }

    fn convert_message(&self, wide: &[u16], target: &mut [u8]) -> bool {
        let text: String = String::from_utf16_lossy(wide_c_prefix(wide));
        let fits: bool = write_utf8_buffer(target, text.as_bytes());
        fits && !self.convert_fails.get()
    }
}

fn wide_c_prefix(wide: &[u16]) -> &[u16] {
    let end: usize = wide
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(wide.len());
    &wide[..end]
}

fn write_wide_buffer(target: &mut [u16], text: &[u16]) -> bool {
    target.fill(0);
    let count: usize = text.len().min(target.len().saturating_sub(1));
    target[..count].copy_from_slice(&text[..count]);
    text.len() < target.len()
}

fn write_utf8_buffer(target: &mut [u8], text: &[u8]) -> bool {
    target.fill(0);
    let count: usize = text.len().min(target.len().saturating_sub(1));
    target[..count].copy_from_slice(&text[..count]);
    text.len() < target.len()
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
    let buffers: Mutex<WindowsErrorBuffers> = Mutex::new(WindowsErrorBuffers::new());
    assert!(
        open_with(
            &calls,
            Some(b"missing.dll"),
            std::ptr::null_mut(),
            &last_error
        )
        .is_null()
    );
    assert_error_details(&calls, &last_error, &buffers, b"missing.dll", 126);
}

fn assert_error_details(
    calls: &FakeWindowsCalls,
    last_error: &AtomicU32,
    buffers: &Mutex<WindowsErrorBuffers>,
    name: &[u8],
    code: u32,
) {
    let expected: String = format!("win32:{code}");
    assert_eq!(
        error_with(calls, last_error, buffers, Some(name)),
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
    let buffers: Mutex<WindowsErrorBuffers> = Mutex::new(WindowsErrorBuffers::new());
    let result: *mut c_void = open_with(
        &calls,
        Some(b"present.dll"),
        std::ptr::null_mut(),
        &last_error,
    );
    assert_eq!(result, loaded);
    let retained: Vec<u8> = error_with(&calls, &last_error, &buffers, Some(b"present.dll"));
    assert_eq!(retained, b"win32:126");
}

#[test]
fn failed_format_call_keeps_its_partial_wide_write() {
    let calls: FakeWindowsCalls = FakeWindowsCalls::default();
    let buffers: Mutex<WindowsErrorBuffers> = Mutex::new(WindowsErrorBuffers::new());
    let last_error: AtomicU32 = AtomicU32::new(120);
    let first: Vec<u8> = error_with(&calls, &last_error, &buffers, Some(b"first.dll"));
    *calls.format_override.borrow_mut() = Some("partial write".encode_utf16().collect());
    calls.format_fails.set(true);
    last_error.store(126, std::sync::atomic::Ordering::Relaxed);
    let second: Vec<u8> = error_with(&calls, &last_error, &buffers, Some(b"second.dll"));
    assert_eq!(first, b"win32:120");
    assert_eq!(second, b"partial write");
}

#[test]
fn failed_conversion_exposes_its_partial_utf8_write() {
    let calls: FakeWindowsCalls = FakeWindowsCalls::default();
    let buffers: Mutex<WindowsErrorBuffers> = Mutex::new(WindowsErrorBuffers::new());
    let last_error: AtomicU32 = AtomicU32::new(120);
    let first: Vec<u8> = error_with(&calls, &last_error, &buffers, Some(b"old.dll"));
    *calls.format_override.borrow_mut() = Some("new message".encode_utf16().collect());
    calls.convert_fails.set(true);
    let stale: Vec<u8> = error_with(&calls, &last_error, &buffers, Some(b"new.dll"));
    assert_eq!(first, b"win32:120");
    assert_eq!(stale, b"new message");
    calls.convert_fails.set(false);
    let updated: Vec<u8> = error_with(&calls, &last_error, &buffers, Some(b"new.dll"));
    assert_eq!(updated, b"new message");
}

#[test]
fn fixed_buffer_failures_keep_their_partial_output_writes() {
    let calls: FakeWindowsCalls = FakeWindowsCalls::default();
    let buffers: Mutex<WindowsErrorBuffers> = Mutex::new(WindowsErrorBuffers::new());
    let last_error: AtomicU32 = AtomicU32::new(1);
    *calls.format_override.borrow_mut() = Some("a".repeat(1023).encode_utf16().collect());
    let truncated: Vec<u8> = error_with(&calls, &last_error, &buffers, None);
    assert_eq!(truncated.len(), 1023);
    *calls.format_override.borrow_mut() = Some("b".repeat(1024).encode_utf16().collect());
    let wide_overflow: Vec<u8> = error_with(&calls, &last_error, &buffers, None);
    assert_eq!(wide_overflow, b"b".repeat(1023));
    *calls.format_override.borrow_mut() = Some("€".repeat(600).encode_utf16().collect());
    let encoding_overflow: Vec<u8> = error_with(&calls, &last_error, &buffers, None);
    assert_eq!(encoding_overflow, "€".repeat(600).as_bytes()[..1023]);
}

#[test]
fn close_forwards_the_cached_or_loaded_handle() {
    let calls: FakeWindowsCalls = FakeWindowsCalls::default();
    let handle: *mut c_void = 7_usize as *mut c_void;
    close_with(&calls, handle);
    assert_eq!(calls.closes.borrow().as_slice(), &[7]);
}
