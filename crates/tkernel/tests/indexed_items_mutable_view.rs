//! Generic mutable indexed views retain OCCT index order and reference writes.
use rustycad_tkernel::ncollection::{IndexedDataMap, ItemsViewMut, KeyValueIndexItems};

#[test]
fn indexed_view_writes_follow_swapped_one_based_indices() {
    let mut map = IndexedDataMap::<i32, i32>::default();
    map.add(10, 100);
    map.add(20, 200);
    map.add(30, 300);
    map.swap(1, 3).unwrap();
    let mut view = ItemsViewMut::<_, KeyValueIndexItems>::new(&mut map);
    assert_eq!((view.size(), view.is_empty()), (3, false));
    let actual: Vec<_> = view.begin().map(update_indexed_value).collect();
    assert_eq!(actual, [(30, 301, 1), (20, 202, 2), (10, 103, 3)]);
    assert_eq!(map.find_from_key(&10), Ok(&103));
    assert_eq!(map.find_from_key(&20), Ok(&202));
    assert_eq!(map.find_from_key(&30), Ok(&301));
}

fn update_indexed_value((key, value, index): (&i32, &mut i32, usize)) -> (i32, i32, usize) {
    *value += index as i32;
    (*key, *value, index)
}

#[test]
fn empty_indexed_view_has_no_mutable_entries() {
    let mut map = IndexedDataMap::<i32, i32>::default();
    let mut view = ItemsViewMut::<_, KeyValueIndexItems>::new(&mut map);
    assert_eq!((view.size(), view.is_empty()), (0, true));
    assert_eq!(view.begin().count(), 0);
}
