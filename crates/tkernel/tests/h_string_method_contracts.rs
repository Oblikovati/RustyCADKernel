//! Native handle-string construction, buffer identity and content hash contracts.
use rustycad_tkernel::ncollection::{DataMap, HashPolicy};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString, HAsciiString, HExtendedString};

#[test]
fn ascii_handle_raw_pointer_names_the_wrapped_buffer_across_shared_handles() {
    let text = Handle::new(HAsciiString::new("abc"));
    let alias = text.clone();
    assert_eq!(text.to_c_ptr(), text.string().as_bytes_with_nul().as_ptr());
    assert_eq!(text.to_c_ptr(), alias.to_c_ptr());
    alias.clear();
    assert_eq!(text.to_c_ptr(), HAsciiString::default().to_c_ptr());
}

#[test]
fn extended_handle_raw_pointer_names_the_wrapped_buffer_across_shared_handles() {
    let text = Handle::new(HExtendedString::from_c_string(b"abc"));
    let alias = text.clone();
    assert_eq!(text.to_ext_ptr(), text.string().to_ext_ptr());
    assert_eq!(text.to_ext_ptr(), alias.to_ext_ptr());
    alias.clear();
    assert_eq!(text.to_ext_ptr(), HExtendedString::default().to_ext_ptr());
}

#[test]
fn move_constructors_transfer_underlying_buffers_without_deep_copies() {
    let mut ascii = AsciiString::new("abc");
    let ascii_pointer = ascii.as_bytes_with_nul().as_ptr();
    let ascii_handle = HAsciiString::from_moved_string(&mut ascii);
    assert_eq!(ascii_handle.to_c_ptr(), ascii_pointer);
    assert!(ascii.is_empty());
    let mut extended = ExtendedString::new("abc");
    let extended_pointer = extended.to_ext_ptr();
    let extended_handle = HExtendedString::from_moved_string(&mut extended);
    assert_eq!(extended_handle.to_ext_ptr(), extended_pointer);
    assert!(extended.is_empty());
}

#[test]
fn nullable_constructor_overloads_raise_native_null_object() {
    assert_eq!(
        HAsciiString::from_optional_c_string(None)
            .err()
            .unwrap()
            .exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(
        HExtendedString::from_optional_c_string(None)
            .err()
            .unwrap()
            .exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(
        HExtendedString::from_optional_utf16(None)
            .err()
            .unwrap()
            .exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(
        HAsciiString::from_optional_c_string(Some(b"a\0b"))
            .unwrap()
            .length(),
        1
    );
    assert_eq!(
        HExtendedString::from_optional_c_string(Some(b"a\0b"))
            .unwrap()
            .length(),
        1
    );
    assert_eq!(
        HExtendedString::from_optional_utf16(Some(&[65, 0, 66]))
            .unwrap()
            .length(),
        1
    );
}

#[test]
fn ascii_handle_content_policy_matches_native_null_identity_and_string_equality() {
    let policy = HAsciiString::hash_policy();
    let null = Handle::<HAsciiString>::default();
    let first = Handle::new(HAsciiString::new("abc"));
    let equal = Handle::new(HAsciiString::new("abc"));
    assert!(first != equal);
    assert!(policy.equal(&first, &equal));
    assert!(policy.equal(&first, &first));
    assert!(policy.equal(&null, &null));
    assert!(!policy.equal(&null, &first));
    assert!(!policy.equal(&first, &null));
    assert_eq!(policy.hash(&null), 0);
    assert_eq!(policy.hash(&first), first.string().hash_code());
    assert_eq!(policy.hash(&first), policy.hash(&equal));
}

#[test]
fn content_policy_compares_embedded_zeros_beyond_the_is_same_state_prefix() {
    let first = Handle::new(HAsciiString::from(AsciiString::from_raw(b"a\0b")));
    let second = Handle::new(HAsciiString::from(AsciiString::from_raw(b"a\0c")));
    assert!(first.is_same_state(&second));
    assert!(!HAsciiString::hash_policy().equal(&first, &second));
}

#[test]
fn content_policy_observes_mutations_through_shared_handles() {
    let first = Handle::new(HAsciiString::new("abc"));
    let alias = first.clone();
    let equal = Handle::new(HAsciiString::new("abc"));
    let policy = HAsciiString::hash_policy();
    assert!(policy.equal(&first, &equal));
    alias.clear();
    assert!(!policy.equal(&first, &equal));
    assert_eq!(policy.hash(&first), AsciiString::default().hash_code());
}

#[test]
fn native_content_policy_integrates_with_kernel_hash_maps() {
    let mut map = DataMap::with_hasher(HAsciiString::hash_policy(), 1);
    map.bind(Handle::new(HAsciiString::new("abc")), 42);
    map.bind(Handle::default(), 7);
    let lookup = Handle::new(HAsciiString::new("abc"));
    assert_eq!(map.find(&lookup), Ok(&42));
    assert_eq!(map.find(&Handle::default()), Ok(&7));
}

#[test]
fn handle_debug_reports_identity_without_formatting_object_contents() {
    let text = Handle::new(HAsciiString::new("secret contents"));
    let output = format!("{text:?}");
    assert!(output.contains("HAsciiString"));
    assert!(output.contains(&format!("{:p}", text.object_address())));
    assert!(output.contains("is_null: false"));
    assert!(!output.contains("secret contents"));
    let null = Handle::<HAsciiString>::default();
    assert!(format!("{null:?}").contains("is_null: true"));
}
