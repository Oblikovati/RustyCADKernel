//! Windows active-code-page conversion through the project-owned Win32 boundary.
#![allow(unsafe_code)] // Win32 calls stay inside this injectable system-adapter module.

use crate::ncollection::{UtfUnit, WideChar};

trait WindowsLocaleCalls {
    fn decode_size(&self, input: &[u8]) -> i32;
    fn decode_into(&self, input: &[u8], output: &mut [u16]) -> i32;
    fn encode_into(&self, input: &[u16], output: &mut [u8]) -> i32;
}

#[cfg(windows)]
struct SystemWindowsLocaleCalls;

#[cfg(windows)]
impl WindowsLocaleCalls for SystemWindowsLocaleCalls {
    fn decode_size(&self, input: &[u8]) -> i32 {
        use windows_sys::Win32::Globalization::{CP_ACP, MB_PRECOMPOSED, MultiByteToWideChar};
        // The input is project-owned and NUL-terminated; null output with zero capacity is Win32's size query.
        unsafe {
            MultiByteToWideChar(
                CP_ACP,
                MB_PRECOMPOSED,
                input.as_ptr(),
                -1,
                std::ptr::null_mut(),
                0,
            )
        }
    }

    fn decode_into(&self, input: &[u8], output: &mut [u16]) -> i32 {
        use windows_sys::Win32::Globalization::{CP_ACP, MB_PRECOMPOSED, MultiByteToWideChar};
        // Both slices remain live; decode_size supplied the bounded wide output capacity.
        unsafe {
            MultiByteToWideChar(
                CP_ACP,
                MB_PRECOMPOSED,
                input.as_ptr(),
                -1,
                output.as_mut_ptr(),
                output.len() as i32,
            )
        }
    }

    fn encode_into(&self, input: &[u16], output: &mut [u8]) -> i32 {
        use windows_sys::Win32::Globalization::{CP_ACP, WideCharToMultiByte};
        // Both slices are NUL-terminated/owned and output length fits Win32's signed size.
        unsafe {
            WideCharToMultiByte(
                CP_ACP,
                0,
                input.as_ptr(),
                -1,
                output.as_mut_ptr(),
                output.len() as i32,
                std::ptr::null(),
                std::ptr::null_mut(),
            )
        }
    }
}

#[cfg(windows)]
pub(super) fn decode(text: &[u8]) -> Vec<WideChar> {
    decode_with(&SystemWindowsLocaleCalls, text)
}

#[cfg(windows)]
pub(super) fn encode(wide: &[WideChar], buffer: &mut [u8]) -> usize {
    encode_with(&SystemWindowsLocaleCalls, wide, buffer)
}

fn decode_with(calls: &impl WindowsLocaleCalls, text: &[u8]) -> Vec<WideChar> {
    let input = nul_terminated_bytes(text);
    let size = calls.decode_size(&input);
    let Ok(capacity) = usize::try_from(size) else {
        return Vec::new();
    };
    if capacity == 0 {
        return Vec::new();
    }
    let mut output = vec![0_u16; capacity.saturating_add(1)];
    let _ = calls.decode_into(&input, &mut output[..capacity]);
    output[capacity] = 0;
    output
        .into_iter()
        .map(|unit| WideChar::from_bits(u32::from(unit)))
        .collect()
}

fn encode_with(calls: &impl WindowsLocaleCalls, wide: &[WideChar], output: &mut [u8]) -> usize {
    let Some(input) = nul_terminated_wide(wide) else {
        return 0;
    };
    let Ok(_) = i32::try_from(output.len()) else {
        return 0;
    };
    usize::try_from(calls.encode_into(&input, output)).unwrap_or(0)
}

fn nul_terminated_bytes(text: &[u8]) -> Vec<u8> {
    let content = text.split(|byte| *byte == 0).next().unwrap_or_default();
    let mut terminated = content.to_vec();
    terminated.push(0);
    terminated
}

fn nul_terminated_wide(wide: &[WideChar]) -> Option<Vec<u16>> {
    let content = wide.iter().take_while(|unit| **unit != 0);
    #[cfg(windows)]
    let mut terminated: Vec<u16> = content.copied().collect();
    #[cfg(not(windows))]
    let mut terminated = content
        .map(|unit| u16::try_from(*unit).ok())
        .collect::<Option<Vec<_>>>()?;
    terminated.push(0);
    Some(terminated)
}

#[cfg(test)]
mod tests {
    use super::{WindowsLocaleCalls, decode_with, encode_with};
    use crate::ncollection::{UtfUnit, WideChar};

    struct Latin1Calls;

    impl WindowsLocaleCalls for Latin1Calls {
        fn decode_size(&self, input: &[u8]) -> i32 {
            i32::try_from(input.len()).unwrap_or(0)
        }

        fn decode_into(&self, input: &[u8], output: &mut [u16]) -> i32 {
            for (unit, byte) in output.iter_mut().zip(input.iter()) {
                *unit = u16::from(*byte);
            }
            i32::try_from(input.len()).unwrap_or(0)
        }

        fn encode_into(&self, input: &[u16], output: &mut [u8]) -> i32 {
            if input.len() > output.len() {
                return 0;
            }
            for (byte, unit) in output.iter_mut().zip(input.iter()) {
                let Ok(value) = u8::try_from(*unit) else {
                    return 0;
                };
                *byte = value;
            }
            i32::try_from(input.len()).unwrap_or(0)
        }
    }

    #[test]
    fn active_codepage_decode_keeps_the_native_extra_terminator() {
        let decoded = decode_with(&Latin1Calls, b"\xe9tail\0ignored");
        let expected: Vec<WideChar> = [
            0xe9_u16,
            b't' as u16,
            b'a' as u16,
            b'i' as u16,
            b'l' as u16,
            0,
            0,
        ]
        .map(wide_from_u16)
        .to_vec();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn active_codepage_encode_reports_short_buffer_and_roundtrips_bytes() {
        let wide: Vec<WideChar> = [0xe9, 0].into();
        let mut bytes = [0xff; 3];
        assert_eq!(encode_with(&Latin1Calls, &wide, &mut bytes), 2);
        assert_eq!(bytes, [0xe9, 0, 0xff]);
        let mut short = [0xff; 1];
        assert_eq!(encode_with(&Latin1Calls, &wide, &mut short), 0);
        assert_eq!(short, [0xff]);
    }

    fn wide_from_u16(unit: u16) -> WideChar {
        WideChar::from_bits(u32::from(unit))
    }
}
