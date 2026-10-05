//! Native-compatible UTF-16 C-string concatenation adapters.
use rustycad_tkernel::tcollection::ExtendedString;

#[test]
fn array_addition_uses_the_native_terminator_boundary() {
    let base = ExtendedString::from_utf16(&[88]);
    let literal = [65, 0, 66, 0];
    let mut appended = base.clone();
    appended.assign_cat_c_units(Some(&literal));
    let result = base.cat_c_units(Some(&literal));
    assert_eq!(appended.as_utf16(), &[88, 65]);
    assert_eq!(result.as_utf16(), &[88, 65]);
}

#[test]
fn bounded_c_unit_adapters_stop_at_nul_but_slice_addition_keeps_span() {
    let base = ExtendedString::from_utf16(&[88]);
    let input = [65, 0, 66];
    let mut c_string = base.clone();
    c_string.assign_cat_c_units(Some(&input));
    assert_eq!(base.cat_c_units(Some(&input)).as_utf16(), &[88, 65]);
    assert_eq!(c_string.as_utf16(), &[88, 65]);
    assert_eq!((&base + &input[..]).as_utf16(), &[88, 65, 0, 66]);
}

#[test]
fn bounded_c_unit_adapter_keeps_a_nonterminated_slice_prefix() {
    let base = ExtendedString::from_utf16(&[88]);
    assert_eq!(base.cat_c_units(Some(&[66, 67])).as_utf16(), &[88, 66, 67]);
}

#[test]
fn null_pointer_adapter_is_a_no_op() {
    let base = ExtendedString::from_utf16(&[88]);
    let mut appended = base.clone();
    appended.assign_cat_c_units(None);
    assert_eq!(appended, base);
    assert_eq!(base.cat_c_units(None), base);
}
