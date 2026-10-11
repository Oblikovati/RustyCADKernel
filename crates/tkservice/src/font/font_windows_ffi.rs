//! Native Windows font discovery isolated behind the project-owned registry interface.
#![allow(unsafe_code)]
use super::font_windows::WindowsFontRegistry;
use windows_sys::Win32::{
    Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS},
    System::{Registry::*, SystemInformation::GetSystemWindowsDirectoryA},
};
pub(super) struct SystemFontRegistry;
struct FontRegistryKey(HKEY);
impl Drop for FontRegistryKey {
    fn drop(&mut self) {
        // SAFETY: this key was opened successfully and is closed exactly once.
        unsafe { RegCloseKey(self.0) };
    }
}
impl WindowsFontRegistry for SystemFontRegistry {
    fn system_windows_directory(&self) -> Option<Vec<u8>> {
        // SAFETY: a null buffer queries its required size, including terminator.
        let size = unsafe { GetSystemWindowsDirectoryA(std::ptr::null_mut(), 0) };
        if size == 0 {
            return None;
        }
        let mut path = vec![0; size as usize];
        // SAFETY: the allocated buffer has the requested capacity.
        let written = unsafe { GetSystemWindowsDirectoryA(path.as_mut_ptr(), size) };
        if written == 0 || written >= size {
            return None;
        }
        path.truncate(written as usize);
        Some(path)
    }
    fn registered_font_paths(&self) -> Vec<Vec<u8>> {
        let Some(key) = open_font_registry_key() else {
            return Vec::new();
        };
        let mut paths = Vec::new();
        for index in 0..u32::MAX {
            match read_font_registry_entry(&key, index) {
                FontRegistryEntry::End => break,
                FontRegistryEntry::Skipped => (),
                FontRegistryEntry::Path(path) => paths.push(path),
            }
        }
        paths
    }
}
fn open_font_registry_key() -> Option<FontRegistryKey> {
    let mut key = std::ptr::null_mut();
    // SAFETY: the native ASCII registry key is terminated and `key` is writable.
    let status = unsafe {
        RegOpenKeyExA(
            HKEY_LOCAL_MACHINE,
            c"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts"
                .as_ptr()
                .cast(),
            0,
            KEY_READ,
            &mut key,
        )
    };
    (status == ERROR_SUCCESS).then(|| FontRegistryKey(key))
}
enum FontRegistryEntry {
    End,
    Skipped,
    Path(Vec<u8>),
}
fn read_font_registry_entry(key: &FontRegistryKey, index: u32) -> FontRegistryEntry {
    let (mut name, mut path) = ([0; 256], [0; 256]);
    let (mut name_size, mut path_size) = (256, 256);
    // SAFETY: the buffers and byte counts agree; the opened key remains alive.
    let status = unsafe {
        RegEnumValueA(
            key.0,
            index,
            name.as_mut_ptr(),
            &mut name_size,
            std::ptr::null(),
            std::ptr::null_mut(),
            path.as_mut_ptr(),
            &mut path_size,
        )
    };
    decode_registry_font_entry(status, &path, path_size)
}

fn decode_registry_font_entry(status: u32, path: &[u8], path_size: u32) -> FontRegistryEntry {
    if status == ERROR_NO_MORE_ITEMS {
        return FontRegistryEntry::End;
    }
    // The native fixed buffer may be stale on errors; do not expose uninitialized paths.
    if status != ERROR_SUCCESS {
        return FontRegistryEntry::Skipped;
    }
    let limit = (path_size as usize).min(path.len());
    let length = path[..limit]
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(limit);
    FontRegistryEntry::Path(path[..length].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_entry_decoding_preserves_bytes_and_rejects_error_buffers() {
        assert!(matches!(
            decode_registry_font_entry(ERROR_NO_MORE_ITEMS, b"stale", 5),
            FontRegistryEntry::End
        ));
        assert!(matches!(
            decode_registry_font_entry(234, b"stale", 5),
            FontRegistryEntry::Skipped
        ));
        for (bytes, size, expected) in [
            (b"a.ttf\0stale".as_slice(), 11, b"a.ttf".as_slice()),
            (b"abc", 99, b"abc"),
            (b"abc", 2, b"ab"),
        ] {
            let FontRegistryEntry::Path(path) =
                decode_registry_font_entry(ERROR_SUCCESS, bytes, size)
            else {
                panic!("expected successful registry bytes")
            };
            assert_eq!(path, expected);
        }
    }
}
