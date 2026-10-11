//! Ordered native chunks keep each committed source under 500 lines.
pub(super) const QUADRIC_NATIVE: &str = concat!(
    include_str!("../../tests/fixtures/sensitive-quadrics-native-00.txt"),
    include_str!("../../tests/fixtures/sensitive-quadrics-native-01.txt"),
    include_str!("../../tests/fixtures/sensitive-quadrics-native-02.txt"),
    include_str!("../../tests/fixtures/sensitive-quadrics-native-03.txt"),
    include_str!("../../tests/fixtures/sensitive-quadrics-native-04.txt"),
    include_str!("../../tests/fixtures/sensitive-quadrics-native-05.txt"),
    include_str!("../../tests/fixtures/sensitive-quadrics-native-06.txt"),
    include_str!("../../tests/fixtures/sensitive-quadrics-native-07.txt"),
    include_str!("../../tests/fixtures/sensitive-quadrics-native-08.txt"),
);
