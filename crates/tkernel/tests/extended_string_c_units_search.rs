//! Null-terminated UTF-16 search adapters against OCCT's pointer overloads.
use rustycad_tkernel::tcollection::ExtendedString;

#[test]
fn pointer_search_stops_at_nul_while_explicit_spans_keep_all_units() {
    let value = ExtendedString::from_raw(&[66, 65, 66, 65, 0]);
    let mixed = [65, 0, 66, 0];
    assert_eq!(value.search_c_units(Some(&mixed)), 2);
    assert_eq!(value.search_from_end_c_units(Some(&mixed)), 4);
    assert_eq!(value.search(&mixed[..3]), -1);
    assert_eq!(value.search_c_units(Some(&[66, 65, 0])), 1);
    assert_eq!(value.search_from_end_c_units(Some(&[66, 65, 0])), 3);
}

#[test]
fn null_and_empty_pointer_searches_return_not_found() {
    let value = ExtendedString::from_raw(&[66, 65]);
    assert_eq!(value.search_c_units(None), -1);
    assert_eq!(value.search_from_end_c_units(None), -1);
    assert_eq!(value.search_c_units(Some(&[0, 65])), -1);
    assert_eq!(value.search_from_end_c_units(Some(&[0])), -1);
}
