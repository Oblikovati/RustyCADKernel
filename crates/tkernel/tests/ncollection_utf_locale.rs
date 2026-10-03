#![cfg(all(unix, not(target_os = "android")))]
#![allow(unsafe_code)]
//! Thread locales isolate native conversion tests from other test threads.
use rustycad_tkernel::ncollection::UtfString;
use rustycad_tkernel::standard::c_locale_sentry::CLocaleSentry;
use std::ffi::CStr;

struct TestThreadLocale {
    owned: libc::locale_t,
    previous: libc::locale_t,
}

impl TestThreadLocale {
    fn enter(name: &CStr) -> Self {
        let owned =
            unsafe { libc::newlocale(libc::LC_CTYPE_MASK, name.as_ptr(), std::ptr::null_mut()) };
        assert!(
            !owned.is_null(),
            "locale {name:?}: expected an installed test locale"
        );
        let previous = unsafe { libc::uselocale(owned) };
        assert!(
            !previous.is_null(),
            "locale {name:?}: expected a valid previous thread locale"
        );
        Self { owned, previous }
    }
}

impl Drop for TestThreadLocale {
    fn drop(&mut self) {
        unsafe {
            libc::uselocale(self.previous);
            libc::freelocale(self.owned);
        }
    }
}

fn converted(input: &[u8], expected: &[u8], symbols: i32) {
    let mut text = UtfString::<u8>::new();
    text.from_locale(input, -1);
    assert_eq!(text.length(), symbols);
    let mut output = [0xff; 12];
    assert_eq!(text.to_locale(&mut output), !expected.is_empty());
    assert_eq!(&output[..expected.len()], expected);
    assert_eq!(output[expected.len()], 0);
    assert!(
        output[expected.len() + 1..]
            .iter()
            .all(|byte| *byte == 0xff)
    );
}

#[test]
fn native_utf8_first_character_byte_count_limits_wide_character_count() {
    let _locale = TestThreadLocale::enter(c"C.UTF-8");
    converted(b"abc", b"a", 1);
    converted("äbc".as_bytes(), "äb".as_bytes(), 2);
    converted("€bcd".as_bytes(), "€bc".as_bytes(), 3);
    converted("😀abcd".as_bytes(), "😀abc".as_bytes(), 4);
    converted(b"\xe4bc", b"", 0);
    converted(b"", b"", 0);
    converted(b"\0abc", b"", 0);
}

#[test]
fn native_locale_output_preserves_truncation_and_error_result() {
    let _locale = TestThreadLocale::enter(c"C.UTF-8");
    let text = UtfString::<u8>::from_unicode_units("äb".as_bytes(), -1);
    let mut one = [0xff];
    assert!(!text.to_locale(&mut one));
    assert_eq!(one, [0]);
    let mut two = [0xff; 2];
    assert!(text.to_locale(&mut two));
    assert_eq!(two, [0xc3, 0xa4]);
    assert!(!text.to_locale(&mut []));
    let _c_locale = CLocaleSentry::new();
    let mut unsupported = [0xff; 4];
    assert!(text.to_locale(&mut unsupported));
    assert_eq!(unsupported, [0xff; 4]);
}

#[test]
fn sentry_restores_multibyte_conversion_and_explicit_symbol_limit() {
    let _locale = TestThreadLocale::enter(c"C.UTF-8");
    {
        let _sentry = CLocaleSentry::new();
        converted("äbc".as_bytes(), b"", 0);
    }
    converted("äbc".as_bytes(), "äb".as_bytes(), 2);
    let mut text = UtfString::<u16>::new();
    text.from_locale("€bcd".as_bytes(), 1);
    assert_eq!(text.to_c_string(), &[0x20ac, 0]);
}
