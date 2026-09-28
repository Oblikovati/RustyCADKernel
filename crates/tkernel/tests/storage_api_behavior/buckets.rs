//! Storage_Bucket, Storage_BucketOfPersistent and Storage_BucketIterator.
use rustycad_tkernel::standard::{Handle, StandardPersistent};
use rustycad_tkernel::storage::{Bucket, BucketIterator, BucketOfPersistent};

fn objects(count: usize) -> Vec<Handle<StandardPersistent>> {
    (0..count)
        .map(|_| Handle::new(StandardPersistent::default()))
        .collect()
}

fn same(left: &Handle<StandardPersistent>, right: &Handle<StandardPersistent>) -> bool {
    std::ptr::eq(left.as_ptr(), right.as_ptr())
}

#[test]
fn appends_spill_into_new_buckets_in_order() {
    let items = objects(5);
    let mut bucket = BucketOfPersistent::new(2, 1);
    for item in &items {
        bucket.append(item);
    }
    assert_eq!(bucket.length(), 5);
    for (index, item) in items.iter().enumerate() {
        assert!(same(&bucket.value(index as i32 + 1), item));
    }
    assert!(bucket.value(6).is_null() && bucket.value(0).is_null());
    let mut walk = BucketIterator::new(Some(&bucket));
    let mut seen = 0;
    while walk.more() {
        assert!(same(&walk.value(), &items[seen]));
        seen += 1;
        walk.next();
    }
    assert_eq!(seen, 5);
}

#[test]
fn an_empty_collection_yields_one_null_slot_and_clear_keeps_one_bucket() {
    let mut bucket = BucketOfPersistent::default();
    let mut walk = BucketIterator::new(Some(&bucket));
    assert!(walk.more() && walk.value().is_null());
    walk.next();
    assert!(!walk.more());
    walk.next();
    assert!(!walk.more());
    bucket.append(&objects(1)[0]);
    bucket.clear();
    assert_eq!(bucket.length(), 0);
}

#[test]
fn iterator_init_reset_and_detached_state() {
    let items = objects(3);
    let mut bucket = BucketOfPersistent::new(2, 4);
    for item in &items {
        bucket.append(item);
    }
    let mut walk = BucketIterator::new(None);
    assert!(!walk.more() && walk.value().is_null());
    walk.reset();
    assert!(!walk.more());
    walk.init(Some(&bucket));
    walk.next();
    walk.next();
    assert!(same(&walk.value(), &items[2]));
    walk.reset();
    assert!(same(&walk.value(), &items[0]));
    walk.init(None);
    assert!(!walk.more());
    walk.reset();
    assert!(walk.more(), "reset returns to the collection given before");
}

#[test]
fn a_bucket_keeps_its_capacity() {
    let mut bucket = Bucket::default();
    assert_eq!(bucket.space_size(), 200_000);
    bucket.clear();
    assert_eq!(Bucket::with_size(3).space_size(), 3);
}
