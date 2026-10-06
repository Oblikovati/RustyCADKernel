//! Full native subclass stdout join; each value below comes from the Rust storage API.
use super::*;
macro_rules! row {
    ($rows:expr, $label:literal $(, $value:expr)* $(,)?) => {{
        write!($rows, $label).unwrap();
        $(write!($rows, " {}", $value).unwrap();)*
        writeln!($rows).unwrap();
    }};
}

pub(super) fn compare_all_rows() {
    let mut rows = String::new();
    let mut map = BaseMapStorage::new(5, None);
    let mut legacy = BaseMapStorage::new_legacy(7, false, None);
    constructor_rows(&mut rows, &mut map, &legacy);
    iterator_rows(&mut rows, &mut map);
    no_resize_row(&mut rows, &map);
    chain_rows(&mut rows, &mut map);
    move_rows(&mut rows, &mut legacy);
    let mut wrapping = BaseMapStorage::new(0, None);
    row!(
        rows,
        "wrapping",
        (wrapping.decrement() == usize::MAX) as u8,
        wrapping.increment()
    );
    row!(
        rows,
        "signed_buckets",
        buckets_from_int(3).unwrap(),
        buckets_from_int(-1).is_err() as u8
    );
    row!(
        rows,
        "prime_aliases",
        next_map_prime(0),
        next_map_prime(100)
    );
    assert_eq!(
        rows,
        include_str!("../../../../tools/oracle/tkernel_base_map_storage_contracts.expected")
    );
}

fn constructor_rows(rows: &mut String, map: &mut BaseMapStorage, legacy: &BaseMapStorage) {
    row!(
        rows,
        "constructor",
        map.nb_buckets(),
        map.size(),
        map.extent(),
        map.length(),
        map.is_empty() as u8,
        map.buckets_mut().is_null() as u8,
        map.buckets().is_null() as u8,
        map.resizable() as u8
    );
    row!(rows, "legacy", legacy.nb_buckets(), legacy.is_empty() as u8);
}

fn iterator_rows(rows: &mut String, map: &mut BaseMapStorage) {
    let default = BaseMapBucketIterator::default();
    let mut empty = BaseMapBucketIterator::new(map);
    // SAFETY: an unallocated cursor contains no node to dereference.
    unsafe { empty.advance() };
    row!(
        rows,
        "unallocated_iterator",
        default.is_equal(&empty) as u8,
        empty.bucket_index(),
        empty.more() as u8
    );
    resize(map, 0);
    let mut retained = BaseMapBucketIterator::new(map);
    let copy = retained.clone();
    // SAFETY: an allocated but empty cursor contains no node to dereference.
    unsafe { retained.advance() };
    row!(
        rows,
        "allocated_iterator",
        map.nb_buckets(),
        copy.bucket_index(),
        retained.bucket_index(),
        retained.is_equal(&copy) as u8,
        copy.is_equal(&default) as u8
    );
    retained.reset();
    row!(
        rows,
        "reset_iterator",
        retained.is_equal(&copy) as u8,
        retained.bucket_index()
    );
}

fn no_resize_row(rows: &mut String, map: &BaseMapStorage) {
    let mut pending = Some(map.begin_resize(102).unwrap());
    let pointer = pending.as_ref().unwrap().buckets();
    let mut second = std::ptr::null_mut();
    let mut count = 999;
    let resized = map
        .begin_resize_legacy_into(0, &mut count, &mut pending, &mut second)
        .unwrap();
    row!(
        rows,
        "no_resize_outputs",
        resized as u8,
        count,
        (pending.as_ref().unwrap().buckets() == pointer) as u8,
        second.is_null() as u8
    );
}

fn chain_rows(rows: &mut String, map: &mut BaseMapStorage) {
    let mut first = MapLinkNode::new(std::ptr::null_mut());
    let first_ptr = std::ptr::from_mut(&mut first);
    let mut second = MapLinkNode::new(first_ptr);
    let second_ptr = std::ptr::from_mut(&mut second);
    let mut last = MapLinkNode::new(std::ptr::null_mut());
    let last_ptr = std::ptr::from_mut(&mut last);
    map.bucket_slots_mut()[0] = second_ptr;
    map.bucket_slots_mut()[4] = last_ptr;
    for _ in 0..3 {
        map.increment();
    }
    traversal_row(rows, map, [first_ptr, second_ptr, last_ptr]);
    row!(
        rows,
        "size_aliases",
        map.size(),
        map.extent(),
        map.length(),
        map.resizable() as u8,
        map.decrement(),
        map.increment()
    );
    destroy_rows(rows, map, [first_ptr, second_ptr, last_ptr]);
}

fn traversal_row(rows: &mut String, map: &BaseMapStorage, nodes: [*mut MapLinkNode; 3]) {
    let mut cursor = BaseMapBucketIterator::new(map);
    write!(rows, "iterator_chain ").unwrap();
    while cursor.more() {
        let tag = nodes
            .iter()
            .position(|pointer| *pointer == cursor.node())
            .unwrap()
            + 1;
        write!(rows, "{tag}:{},", cursor.bucket_index()).unwrap();
        // SAFETY: chain_rows owns all three nodes during this call, with acyclic links.
        unsafe { cursor.advance() };
    }
    writeln!(rows, "{}", cursor.bucket_index()).unwrap();
}

fn destroy_rows(rows: &mut String, map: &mut BaseMapStorage, nodes: [*mut MapLinkNode; 3]) {
    let mut tags = String::new();
    // SAFETY: chain_rows owns all three live nodes throughout this call.
    unsafe {
        map.destroy(
            |node, _| {
                tags.push_str(
                    &(nodes.iter().position(|pointer| *pointer == node).unwrap() + 1).to_string(),
                );
                *(*node).next_mut() = std::ptr::null_mut();
            },
            false,
        )
    };
    row!(
        rows,
        "destroy_retained",
        tags,
        map.size(),
        map.nb_buckets(),
        (!map.buckets().is_null()) as u8,
        map.bucket_slots()[0].is_null() as u8,
        map.bucket_slots()[4].is_null() as u8
    );
    // SAFETY: zero-size map has no remaining traversal.
    unsafe { map.destroy(|_, _| panic!("empty callback"), true) };
    row!(
        rows,
        "destroy_released",
        tags,
        map.nb_buckets(),
        map.buckets().is_null() as u8
    );
}

fn move_rows(rows: &mut String, legacy: &mut BaseMapStorage) {
    let allocator = BaseAllocator::incremental(12 * 1024);
    let mut source = BaseMapStorage::new(7, Some(allocator.clone()));
    resize(&mut source, 0);
    source.increment();
    let pointer = source.buckets();
    let mut moved = BaseMapStorage::move_from(&mut source);
    row!(
        rows,
        "move",
        (moved.buckets() == pointer) as u8,
        moved.size(),
        moved.nb_buckets(),
        source.size(),
        source.nb_buckets(),
        source.buckets().is_null() as u8,
        (source.allocator() == moved.allocator()) as u8
    );
    moved.exchange(legacy);
    row!(
        rows,
        "exchange",
        legacy.size(),
        (legacy.buckets() == pointer) as u8,
        moved.size(),
        moved.buckets().is_null() as u8,
        (legacy.allocator() == &allocator) as u8
    );
}
