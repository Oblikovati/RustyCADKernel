//! Rust side of the differential cases for Standard handles, transients, type descriptors, failures, error handler and TColStd handles.
use super::common::{Case, hex, next_bytes};
use rustycad_tkernel::ncollection::{
    AllocatorObject, HeapAllocatorObject, IncrementalAllocatorObject,
};
use rustycad_tkernel::standard::{
    Handle, RuntimeType, RuntimeTyped, StandardPersistent, StandardTransient, Transient, occ,
};
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;
use std::str::SplitWhitespace;
mod failures;
mod handler;
mod handles;

/// Operation name and handler of every case this area answers.
pub const CASES: &[(&str, Case)] = &[
    ("runtime_type", type_case),
    ("runtime_subtype_name", subtype_name_case),
    ("runtime_object", object_case),
    ("runtime_handle_script", handles::handle_script_case),
    ("runtime_hpacked", handles::hpacked_case),
    ("runtime_failure", failures::failure_case),
    ("runtime_failure_stack", failures::failure_stack_case),
    ("runtime_out_of_memory", failures::out_of_memory_case),
    ("runtime_stack_length", failures::stack_length_case),
    ("runtime_stack_trace", failures::stack_trace_case),
    ("runtime_handler_script", handler::handler_script_case),
    ("runtime_abort", handler::abort_case),
];

/// Column order of the native `types()` table.
fn types() -> [(&'static str, &'static RuntimeType); 7] {
    [
        ("Standard_Transient", StandardTransient::static_type()),
        ("Standard_Type", RuntimeType::static_type()),
        ("Standard_Persistent", StandardPersistent::static_type()),
        (
            "TColStd_HPackedMapOfInteger",
            HPackedMapOfInteger::static_type(),
        ),
        ("NCollection_BaseAllocator", AllocatorObject::static_type()),
        (
            "NCollection_IncAllocator",
            IncrementalAllocatorObject::static_type(),
        ),
        (
            "NCollection_HeapAllocator",
            HeapAllocatorObject::static_type(),
        ),
    ]
}
fn type_named(name: &str) -> &'static RuntimeType {
    types()
        .into_iter()
        .find(|(entry, _)| *entry == name)
        .unwrap_or_else(|| panic!("unknown probe type {name:?}"))
        .1
}
/// '1' or '0', as the native `bit`.
pub fn bit(value: bool) -> char {
    if value { '1' } else { '0' }
}
/// A request's hex field as the text a C string argument would hold.
pub fn next_text(input: &mut SplitWhitespace<'_>) -> String {
    String::from_utf8(next_bytes(input)).expect("probe texts are UTF-8")
}
/// "ADDR : <name>" when Display wrote the descriptor's own address first, else "BAD".
fn normalized_print(descriptor: &'static RuntimeType) -> String {
    let mut printed: Vec<u8> = Vec::new();
    descriptor.print(&mut printed).expect("in-memory print");
    let text = String::from_utf8(printed).expect("UTF-8 type name");
    let prefix = format!("{descriptor:p} : ");
    match text.strip_prefix(&prefix) {
        Some(name) if prefix.starts_with("0x") => format!("ADDR : {name}"),
        _ => "BAD".into(),
    }
}
/// "runtime_type <Name>": name, parent, Print, SubType by descriptor against every column, RTTI.
fn type_case(input: &mut SplitWhitespace<'_>) -> String {
    let descriptor = type_named(input.next().expect("type name"));
    let sub_types: String = types()
        .iter()
        .map(|(_, other)| bit(descriptor.is_subtype(other)))
        .collect();
    let parent = descriptor.parent().map_or("-", |parent| parent.name());
    // A Rust descriptor reference is never null: SubType(null handle) is always false.
    format!(
        "{} {parent} {} {sub_types}0 {} {}",
        descriptor.name(),
        hex(normalized_print(descriptor).as_bytes()),
        descriptor.dynamic_type().name(),
        bit(descriptor.is_kind(StandardTransient::static_type()))
    )
}
/// "runtime_subtype_name <Name> <hex name>": SubType(const char*).
fn subtype_name_case(input: &mut SplitWhitespace<'_>) -> String {
    let descriptor = type_named(input.next().expect("type name"));
    bit(descriptor.is_subtype_name(&next_text(input))).to_string()
}
fn make_object(class: &str) -> Handle<StandardTransient> {
    match class {
        "Standard_Transient" => Handle::new(StandardTransient::default()),
        "Standard_Persistent" => Handle::new(StandardPersistent::default()).cast(),
        "TColStd_HPackedMapOfInteger" => Handle::new(HPackedMapOfInteger::default()).cast(),
        "NCollection_IncAllocator" => Handle::new(IncrementalAllocatorObject::default()).cast(),
        _ => panic!("unknown probe class {class:?}"),
    }
}
/// IsInstance/IsKind by name, by every column type, and the occ:: helpers.
fn object_bits(object: &Handle<StandardTransient>, name: &str) -> String {
    let mut bits: String = [
        bit(object.is_instance_name(name)),
        bit(object.is_kind_name(name)),
        ' ',
    ]
    .into_iter()
    .collect();
    for (_, descriptor) in types() {
        bits.extend([
            bit(object.is_instance(descriptor)),
            bit(object.is_kind(descriptor)),
        ]);
    }
    let null = Handle::<StandardTransient>::default();
    bits.extend([
        ' ',
        bit(occ::is_kind_handle::<StandardPersistent, _>(object)),
        bit(occ::is_instance::<StandardPersistent>(
            object.view::<StandardTransient>().transient(),
        )),
        bit(occ::is_kind_handle::<StandardTransient, _>(&null)),
    ]);
    bits
}
/// "runtime_object <Class> <hex name>": RTTI queries, occ:: helpers and This().
fn object_case(input: &mut SplitWhitespace<'_>) -> String {
    let object = make_object(input.next().expect("class"));
    let bits = object_bits(&object, &next_text(input));
    let owned_this = object
        .this()
        .is_ok_and(|this| std::ptr::eq(this, object.as_ptr()));
    let unowned = StandardTransient::default();
    format!(
        "{bits} {} {} {} {}",
        object.dynamic_type().expect("live object").name(),
        object.ref_count(),
        bit(owned_this),
        super::common::guarded(unowned.this().map(|_| "1".to_string()))
    )
}
