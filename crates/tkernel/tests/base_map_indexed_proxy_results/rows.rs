use super::{
    Read, Write,
    support::{Calls, ProxyMap, take_trace},
};
use rustycad_tkernel::standard::Failure;
use std::{cell::RefCell, rc::Rc};

/// Computes every native row from live Rust results; `all_rows()`.
pub fn all_rows() -> String {
    let calls = Rc::new(RefCell::new(String::new()));
    let mut rows = construction_row(&calls);
    rows.push_str(&value_rows(&calls));
    rows.push_str(&offset_rows(&calls));
    rows.push_str(&failure_rows(&calls));
    rows.push_str(&size_row(&calls));
    rows
}

fn assignment_state(copied: &Read, writable: &Write) -> String {
    let mut assigned = Read::default();
    let address = std::ptr::from_mut(&mut assigned);
    let same_copy = u8::from(std::ptr::eq(assigned.assign(copied), address));
    let same_convert = u8::from(std::ptr::eq(
        assigned.assign_from_mutable(writable),
        address,
    ));
    format!("{} {same_copy} {same_convert}", assigned.index())
}

fn construction_row(calls: &Calls) -> String {
    let owner = ProxyMap::new(calls);
    let detached = Read::default();
    let first = Read::new(&owner, 1);
    let copied = first;
    let writable = Write::new(&owner, 2);
    let converted = writable.as_const();
    let assignment = assignment_state(&copied, &writable);
    let more = u8::from(detached.more_with(&owner, ProxyMap::size).unwrap());
    format!(
        "construction {} {more} {} {} {assignment} {}\n",
        detached.index(),
        first.index(),
        converted.index(),
        take_trace(calls)
    )
}

fn value_rows(calls: &Calls) -> String {
    let mut owner = ProxyMap::new(calls);
    let readonly = Read::new(&owner, 1);
    let writable = Write::new(&owner, 1);
    let number = { readonly.value_with(&owner, ProxyMap::read).unwrap().number };
    let mut rows = format!("const_value {number} {}\n", take_trace(calls));
    writable
        .change_value_with(&mut owner, ProxyMap::write)
        .unwrap()
        .set(17);
    rows.push_str(&format!(
        "mutable_proxy {} {}\n",
        owner.values[0],
        take_trace(calls)
    ));
    rows.push_str(&owned_key_row(&mut owner, calls));
    rows
}

fn owned_key_row(owner: &mut ProxyMap, calls: &Calls) -> String {
    let readonly = Read::new(owner, 1);
    let writable = Write::new(owner, 1);
    let const_key = readonly.key_with(owner, ProxyMap::key).unwrap();
    let mutable_key = writable
        .change_key_with(owner, ProxyMap::mutable_key)
        .unwrap();
    format!(
        "owned_key {const_key} {mutable_key} {}\n",
        take_trace(calls)
    )
}

fn offset_rows(calls: &Calls) -> String {
    let mut owner = ProxyMap::new(calls);
    let readonly = Read::new(&owner, 1);
    let writable = Write::new(&owner, 2);
    let number = {
        readonly
            .value_at_with(&owner, 1, ProxyMap::read)
            .unwrap()
            .number
    };
    let mut rows = format!(
        "const_offset {number} {} {}\n",
        readonly.index(),
        take_trace(calls)
    );
    rows.push_str(&mutable_offset_row(&mut owner, &writable, calls));
    rows
}

fn mutable_offset_row(owner: &mut ProxyMap, writable: &Write, calls: &Calls) -> String {
    writable
        .change_value_at_with(owner, -1, ProxyMap::write)
        .unwrap()
        .set(19);
    format!(
        "mutable_offset {} {} {}\n",
        owner.values[0],
        writable.index(),
        take_trace(calls)
    )
}

fn failure_row(label: &str, error: Failure, calls: &Calls) -> String {
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    format!("{label} {} {}\n", take_trace(calls), error.message())
}

fn failure_rows(calls: &Calls) -> String {
    let mut owner = ProxyMap::new(calls);
    let mut rows = const_failures(&owner, calls);
    rows.push_str(&mutable_key_failures(&mut owner, calls));
    rows
}

fn const_failures(owner: &ProxyMap, calls: &Calls) -> String {
    let zero = Read::new(owner, 0);
    let end = Read::new(owner, 2);
    let error = zero.value_with(owner, ProxyMap::read).unwrap_err();
    let mut rows = failure_row("bad_const", error, calls);
    let error = end.value_at_with(owner, 1, ProxyMap::read).unwrap_err();
    rows.push_str(&failure_row("bad_const_offset", error, calls));
    rows
}

fn mutable_key_failures(owner: &mut ProxyMap, calls: &Calls) -> String {
    let writable = Write::new(owner, 1);
    let zero = Read::new(owner, 0);
    let error = writable
        .change_value_at_with(owner, -1, ProxyMap::write)
        .unwrap_err();
    let mut rows = failure_row("bad_mutable_offset", error, calls);
    let error = zero.key_with(owner, ProxyMap::key).unwrap_err();
    rows.push_str(&failure_row("bad_owned_key", error, calls));
    rows
}

fn size_row(calls: &Calls) -> String {
    let owner = ProxyMap::new(calls);
    let zero = Read::new(&owner, 0);
    let past = Read::new(&owner, 3);
    let zero_more = u8::from(zero.more_with(&owner, ProxyMap::size).unwrap());
    let past_more = u8::from(past.more_with(&owner, ProxyMap::size).unwrap());
    format!("live_size {zero_more} {past_more} {}\n", take_trace(calls))
}
