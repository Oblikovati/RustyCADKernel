use rustycad_tkernel::ncollection::{
    ArrayError, CollectionCursor, CollectionCursorMut, DataMap, FlatDataMap, IndexedDataMap,
    IndexedMap, ItemsView, ItemsViewMut, KeyIndexItems, KeyValueIndexItems, KeyValueItems,
    OrderedDataMap,
};

#[test]
fn default_cursors_have_no_more_items() {
    assert!(!CollectionCursor::<i32>::default().more());
    assert!(!CollectionCursorMut::<i32>::default().more());
}

#[test]
fn cursor_walks_and_reports_positions() {
    let items = [1, 2, 3];
    let mut cursor = CollectionCursor::with_position(&items, 1);
    assert_eq!((cursor.value_iter(), cursor.end_iter()), (1, 3));
    assert_eq!(cursor.value(), Ok(&2));
    cursor.next();
    cursor.next();
    assert!(!cursor.more());
    let error = cursor.value().unwrap_err();
    assert_eq!(
        error,
        ArrayError::Index {
            index: 3,
            lower: 0,
            upper: 2
        }
    );
    *cursor.change_value_iter() = 0;
    *cursor.change_end_iter() = 1;
    assert!(cursor.more() && cursor.value() == Ok(&1));
    cursor.init(&items[..2]);
    assert_eq!((cursor.value_iter(), cursor.end_iter()), (0, 2));
}

#[test]
fn mutable_cursor_changes_values_and_compares_with_const() {
    let mut items = [1, 2];
    let mut cursor = CollectionCursorMut::new(&mut items);
    *cursor.change_value().unwrap() = 10;
    cursor.next();
    assert_eq!(cursor.value(), Ok(&2));
    let snapshot = cursor.as_const();
    assert!(snapshot == cursor.as_const());
    let copy = snapshot;
    assert!(copy == snapshot);
    cursor.next();
    assert!(cursor.change_value().is_err());
    assert_eq!(items, [10, 2]);
}

#[test]
fn cursors_on_different_storage_differ() {
    let (first, second) = ([1], [1]);
    assert!(CollectionCursor::new(&first) != CollectionCursor::new(&second));
    let mut mutable = [1];
    let other = [1];
    let cursor = CollectionCursorMut::new(&mut mutable);
    assert!(cursor != CollectionCursor::new(&other));
    assert!(CollectionCursor::new(&other) != cursor);
}

#[test]
fn items_views_forward_to_the_map() {
    let mut map = DataMap::<i32, i32>::default();
    assert!(ItemsView::<_, KeyValueItems>::new(&map).is_empty());
    map.bind(1, 10);
    let mut view = ItemsViewMut::<_, KeyValueItems>::new(&mut map);
    for (_, value) in view.begin() {
        *value += 1;
    }
    assert_eq!((view.size(), view.is_empty()), (1, false));
    let view = ItemsView::<_, KeyValueItems>::new(&map);
    assert_eq!(view.begin().collect::<Vec<_>>(), [(&1, &11)]);
}

#[test]
fn indexed_views_report_one_based_indices() {
    let mut keys = IndexedMap::<i32>::default();
    keys.add(7);
    keys.add(9);
    let view = ItemsView::<_, KeyIndexItems>::new(&keys);
    assert_eq!(view.begin().collect::<Vec<_>>(), [(&7, 1), (&9, 2)]);
    assert_eq!(view.size(), 2);
    let mut data = IndexedDataMap::<i32, i32>::default();
    data.add(4, 40);
    let view = ItemsView::<_, KeyValueIndexItems>::new(&data);
    assert_eq!(view.begin().collect::<Vec<_>>(), [(&4, &40, 1)]);
}

#[test]
fn flat_and_ordered_maps_have_views() {
    let mut flat = FlatDataMap::<i32, i32>::default();
    flat.bind(1, 2);
    assert_eq!(ItemsView::<_, KeyValueItems>::new(&flat).size(), 1);
    let mut ordered = OrderedDataMap::<i32, i32>::default();
    ordered.bind(3, 4);
    ordered.bind(1, 2);
    let view = ItemsView::<_, KeyValueItems>::new(&ordered);
    assert_eq!(view.begin().collect::<Vec<_>>(), [(&3, &4), (&1, &2)]);
}
