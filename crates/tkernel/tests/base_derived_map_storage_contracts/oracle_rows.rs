use super::*;
use std::fmt::Write;
macro_rules! row {
    ($rows:expr, $label:literal $(, $value:expr)* $(,)?) => {{
        write!($rows, $label).unwrap();
        $(write!($rows, " {}", $value).unwrap();)*
        writeln!($rows).unwrap();
    }};
}

pub(super) fn compare_all_rows() {
    let mut rows = String::new();
    double_rows(&mut rows);
    indexed_rows(&mut rows);
    assert_eq!(
        rows,
        include_str!(
            "../../../../tools/oracle/tkernel_base_derived_map_storage_contracts.expected"
        )
    );
}

fn double_rows(rows: &mut String) {
    let mut map = BaseDoubleMapStorage::new(1009, None);
    resize_double(&mut map, 0);
    // SAFETY: both tables occupy one contiguous allocation.
    let offset = unsafe { map.second_buckets().offset_from(map.buckets()) };
    row!(
        rows,
        "double_contiguous",
        map.nb_buckets(),
        offset,
        map.bucket_slots()[1009].is_null() as u8,
        map.second_slots()[1009].is_null() as u8
    );
    double_noop(rows, &map);
    double_clear(rows, &mut map);
    double_exchange(rows, &mut map);
}

fn double_noop(rows: &mut String, map: &BaseDoubleMapStorage) {
    let mut pending = Some(map.begin_resize(1010).unwrap());
    let first = pending.as_mut().unwrap().bucket_slots_mut().as_mut_ptr();
    let second = pending.as_mut().unwrap().second_slots_mut().as_mut_ptr();
    let mut count = 0;
    let resized = map.begin_resize_into(0, &mut count, &mut pending).unwrap();
    let pending = pending.as_mut().unwrap();
    row!(
        rows,
        "double_no_resize",
        resized as u8,
        count,
        (pending.bucket_slots_mut().as_mut_ptr() == first) as u8,
        (pending.second_slots_mut().as_mut_ptr() == second) as u8
    );
}

fn double_clear(rows: &mut String, map: &mut BaseDoubleMapStorage) {
    let mut node = MapLinkNode::new(std::ptr::null_mut());
    let pointer = std::ptr::from_mut(&mut node);
    map.second_slots_mut()[3] = pointer;
    // SAFETY: zero size skips node traversal.
    unsafe { map.destroy(|_, _| (), false) };
    row!(
        rows,
        "double_empty_clear",
        map.second_slots()[3].is_null() as u8,
        (!map.buckets().is_null()) as u8
    );
    map.bucket_slots_mut()[4] = pointer;
    map.second_slots_mut()[3] = pointer;
    map.increment();
    // SAFETY: the single live stack node has a null successor.
    unsafe { map.destroy(|_, _| (), false) };
    row!(
        rows,
        "double_live_clear",
        map.size(),
        map.bucket_slots()[4].is_null() as u8,
        map.second_slots()[3].is_null() as u8
    );
}

fn double_exchange(rows: &mut String, map: &mut BaseDoubleMapStorage) {
    let first = map.buckets();
    let second = map.second_buckets();
    let mut other = BaseDoubleMapStorage::new(5, None);
    map.exchange(&mut other);
    row!(
        rows,
        "double_exchange",
        (other.buckets() == first) as u8,
        (other.second_buckets() == second) as u8,
        map.nb_buckets(),
        map.second_buckets().is_null() as u8
    );
    // SAFETY: prior double_clear emptied the first table.
    unsafe { other.destroy(|_, _| (), true) };
    row!(
        rows,
        "double_release",
        other.nb_buckets(),
        other.buckets().is_null() as u8,
        other.second_buckets().is_null() as u8
    );
}

fn indexed_rows(rows: &mut String) {
    let mut node = MapLinkNode::new(std::ptr::null_mut());
    let pointer = std::ptr::from_mut(&mut node);
    let mut map = BaseIndexedMapStorage::new(101, None);
    map.resize_index_table(101).unwrap();
    map.index_slots_mut()[3] = pointer;
    map.resize_index_table(101).unwrap();
    row!(
        rows,
        "index_without_hash_reset",
        map.index_slots()[3].is_null() as u8
    );
    resize_indexed(&mut map, 0);
    map.index_slots_mut()[3] = pointer;
    resize_indexed(&mut map, 102);
    row!(
        rows,
        "index_growth",
        map.nb_buckets(),
        (map.index_slots()[3] == pointer) as u8,
        map.index_slots()[102].is_null() as u8,
        map.index_slots()[map.nb_buckets()].is_null() as u8
    );
    map.clear_index_table(false);
    row!(
        rows,
        "index_clear_retained",
        (!map.index_buckets().is_null()) as u8,
        map.index_slots()[3].is_null() as u8
    );
    index_exchange(rows, &mut map);
}

fn index_exchange(rows: &mut String, map: &mut BaseIndexedMapStorage) {
    let index = map.index_buckets();
    let hash = map.base().buckets();
    let mut other = BaseIndexedMapStorage::new(7, None);
    map.exchange(&mut other);
    row!(
        rows,
        "index_exchange",
        (other.index_buckets() == index) as u8,
        (other.base().buckets() == hash) as u8,
        map.index_buckets().is_null() as u8,
        map.nb_buckets()
    );
    other.clear_index_table(true);
    row!(
        rows,
        "index_release",
        other.index_buckets().is_null() as u8,
        (other.base().buckets() == hash) as u8
    );
}
