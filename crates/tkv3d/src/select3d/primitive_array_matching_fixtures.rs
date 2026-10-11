//! Native stdout is chunked to keep every tracked text asset below 500 lines.

// Concatenation preserves native snapshot order and exact bytes without runtime I/O.
pub(super) const MATCHING_NATIVE_SNAPSHOTS: &str = concat!(
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-00.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-01.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-02.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-03.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-04.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-05.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-06.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-07.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-08.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-09.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-10.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-11.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-12.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-13.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-14.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-15.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-16.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-17.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-18.txt"),
    include_str!("../../tests/fixtures/sensitive-primitive-matching-native-19.txt"),
);
