use rustycad_tkernel::ncollection::{
    HashPolicy, IndexedDataMap, IndexedDataValues, IndexedDataValuesMut,
};
fn send_sync<T: Send + Sync>() {}
fn send_value<T: Send>(_: T) {}
fn unpin<T: Unpin>() {}
#[derive(Debug)]
struct Pinned(std::marker::PhantomPinned);
struct PinnedPolicy;
impl HashPolicy<Pinned> for PinnedPolicy {
    fn hash(&self, _: &Pinned) -> usize {
        0
    }
    fn equal(&self, _: &Pinned, _: &Pinned) -> bool {
        true
    }
}
#[test]
fn owner_and_views_preserve_positive_thread_traits_and_unpin() {
    send_sync::<IndexedDataMap<i32, i32>>();
    send_sync::<IndexedDataValues<'_, i32>>();
    send_sync::<IndexedDataValuesMut<'_, i32>>();
    unpin::<IndexedDataMap<Pinned, Pinned, PinnedPolicy>>();
    let mut map = IndexedDataMap::<i32, i32>::default();
    map.add(1, 7);
    send_value(map.items_mut());
    send_value(map.indexed_items_mut());
}
fn shorten_map<'a>(
    _: &'a (),
    map: IndexedDataMap<&'static str, &'static str, BorrowedPolicy>,
) -> IndexedDataMap<&'a str, &'a str, BorrowedPolicy> {
    map
}
fn shorten_view<'a>(
    map: &'a IndexedDataMap<&'static str, &'static str, BorrowedPolicy>,
) -> IndexedDataValues<'a, &'a str> {
    map.iter()
}
#[test]
fn owner_and_shared_views_remain_covariant_in_borrowed_payloads() {
    let anchor = ();
    let mut map = IndexedDataMap::<&'static str, &'static str, _>::with_hasher(BorrowedPolicy, 1);
    map.add("key", "item");
    assert_eq!(shorten_view(&map).next(), Some(&"item"));
    let shortened = shorten_map(&anchor, map);
    assert_eq!(shortened.find_key(1), Ok(&"key"));
}
#[test]
fn retained_disjoint_mutable_items_and_keys_survive_next_projection() {
    let mut map = IndexedDataMap::<i32, i32>::default();
    map.add(1, 10);
    map.add(2, 20);
    let mut pairs = map.items_mut();
    let (key1, value1) = pairs.next().unwrap();
    let (key2, value2) = pairs.next().unwrap();
    *value1 += 1;
    *value2 += 2;
    assert_eq!((*key1, *value1, *key2, *value2), (1, 11, 2, 22));
    assert!(pairs.next().is_none());
    drop(pairs);
    let mut values = map.iter_mut();
    let first = values.next().unwrap();
    let second = values.next().unwrap();
    *first += 10;
    *second += 10;
    assert_eq!((*first, *second), (21, 32));
}
#[test]
fn value_only_view_can_cross_threads_without_moving_non_send_keys() {
    #[derive(Debug)]
    struct LocalKey(std::rc::Rc<()>);
    struct LocalPolicy;
    impl HashPolicy<LocalKey> for LocalPolicy {
        fn hash(&self, key: &LocalKey) -> usize {
            std::rc::Rc::strong_count(&key.0)
        }
        fn equal(&self, a: &LocalKey, b: &LocalKey) -> bool {
            std::rc::Rc::ptr_eq(&a.0, &b.0)
        }
    }
    let mut map = IndexedDataMap::with_hasher(LocalPolicy, 1);
    map.add(LocalKey(std::rc::Rc::new(())), 7);
    send_value(map.iter());
    send_value(map.iter_mut());
}

struct BorrowedPolicy;
impl HashPolicy<&str> for BorrowedPolicy {
    fn hash(&self, key: &&str) -> usize {
        key.len()
    }
    fn equal(&self, a: &&str, b: &&str) -> bool {
        a == b
    }
}
