//! Native default-key specialization follows the static handle type, not dynamic RTTI.
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy, HashKey, HashPolicy};
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString, HAsciiString, HExtendedString};

fn assert_content_map<T: rustycad_tkernel::standard::RuntimeTyped>(
    first: Handle<T>,
    equal: Handle<T>,
) {
    let null = Handle::<T>::default();
    assert_ne!(first, equal);
    assert!(DefaultHashPolicy.equal(&first, &equal));
    assert_eq!(first.occt_hash(), equal.occt_hash());
    assert_eq!(null.occt_hash(), 0);
    assert!(DefaultHashPolicy.equal(&null, &null));
    assert!(!DefaultHashPolicy.equal(&first, &null));
    assert!(!DefaultHashPolicy.equal(&null, &first));
    let mut map = DataMap::<Handle<T>, i32>::default();
    assert!(map.bind(first.clone(), 1));
    assert!(!map.bind(equal, 2));
    assert!(map.bind(null.clone(), 3));
    assert_eq!(map.len(), 2);
    assert_eq!(map.find(&first), Ok(&2));
    assert_eq!(map.find(&null), Ok(&3));
}

#[test]
fn default_ascii_handle_keys_compare_contents_and_support_null() {
    assert_content_map(
        Handle::new(HAsciiString::new("abc")),
        Handle::new(HAsciiString::new("abc")),
    );
}

#[test]
fn default_extended_handle_keys_compare_contents_and_support_null() {
    assert_content_map(
        Handle::new(HExtendedString::from_c_string(b"abc")),
        Handle::new(HExtendedString::from_c_string(b"abc")),
    );
}

#[test]
fn default_string_handle_keys_compare_beyond_embedded_nuls() {
    let first = Handle::new(HAsciiString::from(AsciiString::from_raw(b"a\0b")));
    let second = Handle::new(HAsciiString::from(AsciiString::from_raw(b"a\0c")));
    assert!(first.is_same_state(&second));
    assert!(!DefaultHashPolicy.equal(&first, &second));
    let first = Handle::new(HExtendedString::from(ExtendedString::from_raw(&[
        65, 0, 66,
    ])));
    let second = Handle::new(HExtendedString::from(ExtendedString::from_raw(&[
        65, 0, 67,
    ])));
    assert!(!DefaultHashPolicy.equal(&first, &second));
}

#[test]
fn statically_upcast_string_handles_keep_native_identity_keys() {
    let first = Handle::new(HAsciiString::new("abc"));
    let second = Handle::new(HAsciiString::new("abc"));
    let base_first = first.cast::<StandardTransient>();
    let base_second = second.cast::<StandardTransient>();
    assert!(!DefaultHashPolicy.equal(&base_first, &base_second));
    assert_eq!(base_first.occt_hash(), base_first.object_address() as usize);
    let mut map = DataMap::<Handle<StandardTransient>, i32>::default();
    assert!(map.bind(base_first, 1));
    assert!(map.bind(base_second, 2));
    assert_eq!(map.len(), 2);
}

#[test]
fn ordinary_handles_and_rust_hash_keep_identity_semantics() {
    use std::hash::{Hash, Hasher};
    let first = Handle::new(StandardTransient::default());
    let second = Handle::new(StandardTransient::default());
    assert!(!DefaultHashPolicy.equal(&first, &second));
    assert!(DefaultHashPolicy.equal(&first, &first.clone()));
    let ascii = Handle::new(HAsciiString::new("abc"));
    let mut actual = std::collections::hash_map::DefaultHasher::new();
    ascii.hash(&mut actual);
    let mut identity = std::collections::hash_map::DefaultHasher::new();
    (ascii.object_address() as usize).hash(&mut identity);
    assert_eq!(actual.finish(), identity.finish());
}

#[derive(Default)]
struct PolicySelectedTransient {
    parent: StandardTransient,
    value: u32,
}
#[derive(Default)]
struct SelectedHandlePolicy;
impl HashPolicy<Handle<PolicySelectedTransient>> for SelectedHandlePolicy {
    fn hash(&self, key: &Handle<PolicySelectedTransient>) -> usize {
        key.get().map_or(0, |key| key.value as usize)
    }
    fn equal(
        &self,
        first: &Handle<PolicySelectedTransient>,
        second: &Handle<PolicySelectedTransient>,
    ) -> bool {
        first.get().map(|key| key.value) == second.get().map(|key| key.value)
    }
}
rustycad_tkernel::impl_transient!(
    PolicySelectedTransient,
    "PolicySelectedTransient",
    StandardTransient,
    parent,
    interfaces: [],
    handle_key_policy: SelectedHandlePolicy
);

#[test]
fn downstream_macro_can_select_native_policy_without_dynamic_downcasts() {
    assert_content_map(
        Handle::new(PolicySelectedTransient {
            value: 7,
            ..Default::default()
        }),
        Handle::new(PolicySelectedTransient {
            value: 7,
            ..Default::default()
        }),
    );
}
