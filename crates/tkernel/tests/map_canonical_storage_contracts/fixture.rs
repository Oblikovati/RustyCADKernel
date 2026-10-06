use rustycad_tkernel::ncollection::Map;
#[path = "../support/traced_collision_hash.rs"]
mod hashing;
pub use hashing::CollisionHash;
pub type TestMap = Map<i32, CollisionHash>;
pub fn populated(buckets: usize) -> TestMap {
    let mut map = Map::with_hasher(CollisionHash::default(), buckets);
    for key in 1..=3 {
        map.add(key);
    }
    map
}
pub fn keys(map: &TestMap) -> String {
    map.iter().map(|key| format!("{key},")).collect()
}
pub fn native_row(prefix: &str, actual: String) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_map_canonical_storage_contracts.expected");
    assert_eq!(
        actual,
        expected
            .lines()
            .find(|row| row.starts_with(prefix))
            .unwrap()
    );
}
pub fn caught(operation: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err()
}
