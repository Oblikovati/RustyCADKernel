//! Native ItemsView forward cursors preserve reference identity and end comparisons.
use rustycad_tkernel::ncollection::{
    DataMap, FlatDataMap, IndexedDataMap, IndexedMap, ItemsIterator, ItemsSource, ItemsView,
    ItemsViewMut, KeyIndexItems, KeyValueIndexItems, KeyValueItems, OrderedDataMap,
};
use std::cell::Cell;

#[test]
fn readonly_cursor_copy_prefix_postfix_and_assignment_keep_positions() {
    let mut map = IndexedDataMap::<i32, i32>::default();
    map.add(10, 100);
    map.add(20, 200);
    let view = ItemsView::<_, KeyValueIndexItems>::new(&map);
    let mut cursor = view.begin();
    let previous = cursor.advance_post();
    assert_eq!(previous.value(), Some((&10, &100, 1)));
    assert_eq!(cursor.value(), Some((&20, &200, 2)));
    assert!(cursor != previous);
    let address = std::ptr::from_mut(&mut cursor);
    assert_eq!(std::ptr::from_mut(cursor.assign_from(&previous)), address);
    assert_eq!(std::ptr::from_mut(cursor.assign_self()), address);
    assert_eq!(std::ptr::from_mut(cursor.advance_returning()), address);
    assert_eq!(cursor.next(), Some((&20, &200, 2)));
    assert!(cursor == view.end());
}

struct NoncloneExtractor;
struct NoncloneSource;
impl ItemsSource<NoncloneExtractor> for NoncloneSource {
    type Item<'a> = i32;
    fn items_begin(&self) -> impl Iterator<Item = i32> + Clone {
        [10, 20].into_iter()
    }
    fn items_size(&self) -> usize {
        2
    }
    fn items_is_empty(&self) -> bool {
        false
    }
}

#[test]
fn copied_view_needs_no_source_or_extractor_clone() {
    let source = NoncloneSource;
    let view = ItemsView::<_, NoncloneExtractor>::new(&source);
    let copied = view;
    assert!(view.begin() == copied.cbegin());
    assert!(view.end() == copied.cend());
    assert_eq!(copied.begin().collect::<Vec<_>>(), [10, 20]);
}

#[test]
fn readonly_views_compare_live_owners_and_all_exhausted_positions() {
    let mut first = IndexedMap::<i32>::default();
    let mut second = IndexedMap::<i32>::default();
    first.add(10);
    second.add(10);
    let left = ItemsView::<_, KeyIndexItems>::new(&first);
    let right = ItemsView::<_, KeyIndexItems>::new(&second);
    assert!(left.begin() != right.begin());
    assert!(left.cend() == right.end());
    let mut cursor = left.cbegin();
    assert_eq!(cursor.next(), Some((&10, 1)));
    assert!(cursor == right.cend());
    let detached = ItemsIterator::<std::iter::Empty<i32>>::default();
    assert!(cursor == detached);
    assert!(!detached.more());
}

macro_rules! keyed_view_case {
    ($name:ident, $map:ident) => {
        #[test]
        fn $name() {
            let mut map = $map::<i32, i32>::default();
            map.bind(10, 100);
            let view = ItemsView::<_, KeyValueItems>::new(&map);
            assert!(view.begin() == view.cbegin());
            let mut cursor = view.cbegin();
            assert_eq!(cursor.value(), Some((&10, &100)));
            assert_eq!(cursor.advance_post().value(), Some((&10, &100)));
            assert!(cursor == view.cend());
        }
    };
}
keyed_view_case!(data_map_items_cursor_contract, DataMap);
keyed_view_case!(flat_data_map_items_cursor_contract, FlatDataMap);
keyed_view_case!(ordered_data_map_items_cursor_contract, OrderedDataMap);

#[test]
fn mutable_const_qualified_view_still_yields_writable_values() {
    let mut map = IndexedDataMap::<i32, i32>::default();
    map.add(10, 100);
    let mut view = ItemsViewMut::<_, KeyValueIndexItems>::new(&mut map);
    for (key, value, index) in view.cbegin() {
        assert_eq!((*key, index), (10, 1));
        *value += 7;
    }
    assert!(view.end().next().is_none());
    assert!(view.cend().next().is_none());
    assert_eq!(map.find_from_index(1).unwrap(), &107);
}

struct SourceCalls(Cell<usize>);
impl ItemsSource<KeyValueItems> for SourceCalls {
    type Item<'a> = usize;
    fn items_begin(&self) -> impl Iterator<Item = usize> + Clone {
        self.0.set(self.0.get() + 1);
        std::iter::once(7)
    }
    fn items_size(&self) -> usize {
        1
    }
    fn items_is_empty(&self) -> bool {
        false
    }
}

#[test]
fn end_construction_does_not_initialize_the_maps_begin_iterator() {
    let source = SourceCalls(Cell::new(0));
    let view = ItemsView::<_, KeyValueItems>::new(&source);
    assert!(!view.end().more());
    assert!(!view.cend().more());
    assert_eq!(source.0.get(), 0);
    assert_eq!(view.cbegin().value(), Some(7));
    assert_eq!(source.0.get(), 1);
}
