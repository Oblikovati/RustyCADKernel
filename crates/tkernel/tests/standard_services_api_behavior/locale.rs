//! Standard_CLocaleSentry.
use rustycad_tkernel::standard::c_locale_sentry::CLocaleSentry;
use rustycad_tkernel::tcollection::AsciiString;

#[test]
fn the_c_locale_is_one_shared_instance_with_c_conventions() {
    let locale = CLocaleSentry::get_c_locale();
    assert!(std::ptr::eq(locale, CLocaleSentry::get_c_locale()));
    assert_eq!(locale.name(), "C");
    assert_eq!(locale.decimal_point(), '.');
    assert_eq!(locale.thousands_separator(), "");
}

#[test]
fn numbers_keep_c_text_inside_and_outside_a_sentry() {
    let outside = AsciiString::from_real(-2.25);
    let inside = {
        let _sentry = CLocaleSentry::default();
        AsciiString::from_real(-2.25)
    };
    assert_eq!(inside.as_bytes(), b"-2.25");
    assert_eq!(outside, inside);
}
