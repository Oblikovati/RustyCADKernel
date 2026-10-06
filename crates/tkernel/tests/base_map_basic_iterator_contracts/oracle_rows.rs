//! Each row is computed from the typed Rust cursor and joined to native subclass stdout.
use super::{
    support::{Calls, value_address},
    *,
};
use std::fmt::Write as _;
macro_rules! row {
    ($rows:expr, $label:literal $(, $value:expr)* $(,)?) => {{
        write!($rows, $label).unwrap();
        $(write!($rows, " {}", $value).unwrap();)*
        writeln!($rows).unwrap();
    }};
}

pub(super) fn compare_all_rows() {
    with_nodes(|nodes, calls| {
        let mut rows = String::new();
        empty_row(&mut rows);
        mutable_rows(&mut rows, nodes, &calls);
        node_chain_row(&mut rows, nodes);
        keyonly_rows(&mut rows, nodes, &calls);
        ordered_rows(&mut rows, nodes, &calls);
        copy_rows(&mut rows, nodes);
        assert_eq!(
            rows,
            include_str!(
                "../../../../tools/oracle/tkernel_base_map_basic_iterator_contracts.expected"
            )
        );
    });
}

fn empty_row(rows: &mut String) {
    let mut empty = Write::default();
    let map = ProbeMap::new([std::ptr::null_mut(); 3]);
    let other = Read::new(&map);
    // SAFETY: empty cursor never dereferences a node.
    unsafe { empty.advance() };
    row!(
        rows,
        "default_end",
        empty.more() as u8,
        empty.node().is_null() as u8,
        (empty == other) as u8
    );
}

fn mutable_rows(rows: &mut String, nodes: [*mut Node; 3], calls: &Calls) {
    let mut map = ProbeMap::new(nodes);
    let mut cursor = Write::new(&mut map);
    calls.reset();
    // SAFETY: with_nodes keeps every node live; each exclusive value access is sequential.
    unsafe {
        let value = *cursor.value();
        let arrow = cursor.value_ptr();
        let key = *cursor.key();
        row!(
            rows,
            "mutable_access",
            value,
            key,
            (arrow == value_address(nodes[0])) as u8,
            calls.reads.get(),
            calls.writes.get(),
            calls.keys.get()
        );
        *cursor.value() = 111;
    }
    const_rows(rows, nodes, &cursor, calls);
    advance_rows(rows, &mut cursor, nodes);
}

fn const_rows(rows: &mut String, nodes: [*mut Node; 3], writable: &Write<'_>, calls: &Calls) {
    let view = ProbeMap::new(nodes);
    let converted = Read::new(&view);
    // SAFETY: the live node's value has no concurrent mutable access.
    let value = unsafe { *converted.value() };
    row!(
        rows,
        "const_container",
        (converted == *writable) as u8,
        (converted.node() == nodes[0]) as u8,
        value
    );
    let mut assigned = Read::default();
    let identity = std::ptr::from_mut(&mut assigned);
    row!(
        rows,
        "const_assignment",
        (std::ptr::eq(assigned.assign(&converted), identity)) as u8,
        (assigned == *writable) as u8
    );
    calls.reset();
    // SAFETY: shared reads end before any later mutation through the writable cursor.
    let (value, arrow) = unsafe { (*converted.value(), converted.value_ptr()) };
    // SAFETY: nodes[0] remains live; only the address is produced here.
    let expected = unsafe { value_address(nodes[0]) };
    row!(
        rows,
        "const_access",
        value,
        (arrow == expected) as u8,
        calls.reads.get(),
        calls.writes.get()
    );
}

fn advance_rows(rows: &mut String, cursor: &mut Write<'_>, nodes: [*mut Node; 3]) {
    // SAFETY: both bucket chains remain live throughout this callback.
    unsafe {
        let old = cursor.advance_post();
        row!(
            rows,
            "postfix",
            *old.key(),
            *cursor.key(),
            (old.node() == nodes[0]) as u8
        );
        let identity = std::ptr::from_mut(cursor);
        let same = std::ptr::eq(cursor.advance_returning(), identity);
        row!(
            rows,
            "prefix",
            same as u8,
            *cursor.key(),
            (cursor.node() == nodes[2]) as u8
        );
        cursor.advance();
        cursor.advance();
    }
    let other_map = ProbeMap::new([std::ptr::null_mut(); 3]);
    let other = Read::new(&other_map);
    row!(
        rows,
        "all_ends",
        cursor.more() as u8,
        (*cursor == Write::default()) as u8,
        (*cursor == other) as u8
    );
}

fn node_chain_row(rows: &mut String, nodes: [*mut Node; 3]) {
    let mut cursor = Read::from_node(nodes[0]);
    write!(rows, "node_chain ").unwrap();
    while cursor.more() {
        // SAFETY: a->b raw chain remains live throughout with_nodes.
        unsafe {
            write!(rows, "{},", cursor.key()).unwrap();
            cursor.advance();
        }
    }
    writeln!(rows, "{}", cursor.more() as u8).unwrap();
}

fn keyonly_rows(rows: &mut String, nodes: [*mut Node; 3], calls: &Calls) {
    let mut map = ProbeMap::new(nodes);
    let cursor = BaseMapBasicIterator::<Node, true, true>::new(&mut map);
    calls.reset();
    // SAFETY: key-only cursor performs shared accesses to the live node.
    let (value, key, arrow) = unsafe { (*cursor.value(), *cursor.key(), cursor.value_ptr()) };
    // SAFETY: only the address of the live value is produced.
    let expected = unsafe { value_address(nodes[0]) };
    row!(
        rows,
        "keyonly",
        value,
        key,
        (arrow == expected) as u8,
        calls.reads.get(),
        calls.writes.get(),
        calls.keys.get()
    );
}

fn ordered_rows(rows: &mut String, nodes: [*mut Node; 3], calls: &Calls) {
    let map = ProbeMap::new(nodes);
    let mut ordered = BaseMapBasicIterator::<Node, false, false, true>::new(&map);
    calls.reset();
    write!(rows, "ordered ").unwrap();
    while ordered.more() {
        // SAFETY: c->a->b ordered chain remains live through this callback.
        unsafe {
            write!(rows, "{},", ordered.key()).unwrap();
            ordered.advance();
        }
    }
    writeln!(rows, "{} {}", calls.buckets.get(), calls.orders.get()).unwrap();
    let keyonly = BaseMapBasicIterator::<Node, false, true, true>::new(&map);
    calls.reset();
    // SAFETY: only sequential shared reads of the live first ordered node occur.
    let (key, value) = unsafe { (*keyonly.key(), *keyonly.value()) };
    row!(
        rows,
        "ordered_keyonly",
        key,
        value,
        calls.reads.get(),
        calls.keys.get()
    );
}

fn copy_rows(rows: &mut String, nodes: [*mut Node; 3]) {
    let old = Write::from_node(nodes[0]);
    let copied = old;
    let mut target = Write::default();
    let identity = std::ptr::from_mut(&mut target);
    let assigned = std::ptr::eq(target.assign(&copied), identity);
    let equal = target == old;
    row!(
        rows,
        "copy_assignment",
        assigned as u8,
        equal as u8,
        (std::ptr::eq(target.assign_self(), identity)) as u8
    );
    let mut other = ProbeMap::new([std::ptr::null_mut(); 3]);
    other.storage.bucket_slots_mut()[40] = nodes[0].cast();
    let same = Read::new(&other);
    row!(
        rows,
        "shared_node_equality",
        (same == old) as u8,
        (same != Write::default()) as u8
    );
}
