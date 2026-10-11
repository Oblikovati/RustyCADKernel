//! Ordered chunks reproduce pinned native stdout without external I/O.

pub(super) const TRIANGULATION_NATIVE_SNAPSHOTS: &str = concat!(
    include_str!("../../tests/fixtures/sensitive-triangulation-native-00.txt"),
    include_str!("../../tests/fixtures/sensitive-triangulation-native-01.txt"),
    include_str!("../../tests/fixtures/sensitive-triangulation-native-02.txt"),
    include_str!("../../tests/fixtures/sensitive-triangulation-native-03.txt"),
    include_str!("../../tests/fixtures/sensitive-triangulation-native-04.txt"),
    include_str!("../../tests/fixtures/sensitive-triangulation-native-05.txt"),
    include_str!("../../tests/fixtures/sensitive-triangulation-native-06.txt"),
    include_str!("../../tests/fixtures/sensitive-triangulation-native-07.txt"),
    include_str!("../../tests/fixtures/sensitive-triangulation-native-08.txt"),
    include_str!("../../tests/fixtures/sensitive-triangulation-native-09.txt"),
    include_str!("../../tests/fixtures/sensitive-triangulation-native-10.txt"),
);
