//! Standard_GUID, Standard_Time, Standard_Atomic and Standard_VersionInfo cases
//! (twin of tkernel_standard_values_probe.cxx).
use super::super::common::{guarded, hex, next, next_bytes};
use super::scalar::hex64;
use rustycad_tkernel::standard::{Guid, atomic, time, version_info};
use std::str::SplitWhitespace;
use std::sync::atomic::{AtomicI32, Ordering};

/// Text, UUID fields, std::hash, ToExtString and ShallowDump of one GUID.
fn guid_image(guid: Guid) -> String {
    let uuid = guid.to_uuid();
    let ext: Vec<u8> = guid
        .to_ext_string()
        .iter()
        .flat_map(|unit| unit.to_ne_bytes())
        .collect();
    let mut dump = String::new();
    guid.shallow_dump(&mut dump).expect("String write");
    format!(
        "{guid} {} {} {} {} {} {} {dump}",
        uuid.data1,
        uuid.data2,
        uuid.data3,
        hex(&uuid.data4),
        hex64(guid.hash_code() as u64),
        hex(&ext)
    )
}

/// "guid_text <hex>": CheckGUIDFormat and the `const char*` constructor.
pub fn guid_text_case(input: &mut SplitWhitespace<'_>) -> String {
    let text = next_bytes(input);
    let check = u8::from(Guid::check_format(Some(&text)));
    format!(
        "{check} {}",
        guarded(Guid::from_ascii(&text).map(guid_image))
    )
}

/// "guid_utf16 <hex units>": the `const char16_t*` constructor.
pub fn guid_utf16_case(input: &mut SplitWhitespace<'_>) -> String {
    let text = input.next().expect("UTF-16 units");
    let units: Vec<u16> = (0..text.len() / 4)
        .map(|i| u16::from_str_radix(&text[4 * i..4 * i + 4], 16).expect("hex unit"))
        .collect();
    guarded(Guid::from_utf16(&units).map(guid_image))
}

/// "guid_components a b c d e f g h i j": the component and UUID constructors.
pub fn guid_components_case(input: &mut SplitWhitespace<'_>) -> String {
    let parts: [i64; 10] = std::array::from_fn(|_| next::<i64>(input));
    let bytes: [u8; 6] = std::array::from_fn(|i| parts[4 + i] as u8);
    let guid = Guid::from_components(
        parts[0] as i32,
        parts[1] as u16,
        parts[2] as u16,
        parts[3] as u16,
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
    );
    let copy = Guid::from_uuid(guid.to_uuid());
    format!("{} {}", guid_image(guid), u8::from(copy == guid))
}

/// "guid_compare <hex> <hex>": comparisons and both Assign/operator= overloads.
pub fn guid_compare_case(input: &mut SplitWhitespace<'_>) -> String {
    let first = Guid::from_ascii(&next_bytes(input)).expect("valid GUID request");
    let second = Guid::from_ascii(&next_bytes(input)).expect("valid GUID request");
    let mut assigned = Guid::new();
    assigned.assign(second);
    let mut from_uuid = Guid::new();
    from_uuid.assign_uuid(first.to_uuid());
    let operator = second;
    let uuid_operator = Guid::from(first.to_uuid());
    [
        first == second,
        first != second,
        first.is_same(second),
        first.is_not_same(second),
        assigned.is_same(second),
        from_uuid.is_same(first),
        operator == second,
        uuid_operator == first,
    ]
    .iter()
    .map(|&flag| if flag { '1' } else { '0' })
    .collect()
}

/// "time_equal <a> <b>".
pub fn time_equal_case(input: &mut SplitWhitespace<'_>) -> String {
    let first = next::<i64>(input);
    u8::from(time::is_equal(first, next::<i64>(input))).to_string()
}

/// "atomic <start> <old> <new>": increment, decrement twice, increment, then two CAS calls.
pub fn atomic_case(input: &mut SplitWhitespace<'_>) -> String {
    let start = next::<i32>(input);
    let old = next::<i32>(input);
    let new = next::<i32>(input);
    let value = AtomicI32::new(start);
    let steps = [
        atomic::atomic_increment(&value),
        atomic::atomic_decrement(&value),
        atomic::atomic_decrement(&value),
        atomic::atomic_increment(&value),
    ];
    let swapped = u8::from(atomic::atomic_compare_and_swap(&value, old, new));
    let after_swap = value.load(Ordering::SeqCst);
    let again = u8::from(atomic::atomic_compare_and_swap(&value, old, start));
    format!(
        "{} {} {} {} {swapped} {after_swap} {again} {}",
        steps[0],
        steps[1],
        steps[2],
        steps[3],
        value.load(Ordering::SeqCst)
    )
}

/// "version": the Standard_VersionInfo functions.
pub fn version_case(_: &mut SplitWhitespace<'_>) -> String {
    format!(
        "{} {} {} {} {}",
        hex(version_info::occt_development_version().as_bytes()),
        hex64(version_info::occt_version_double().to_bits()),
        version_info::occt_version_string(),
        version_info::occt_version_string_complete(),
        version_info::occt_version_string_extended()
    )
}
