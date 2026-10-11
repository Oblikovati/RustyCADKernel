//! Numeric native assets have no OCCT runtime dependency in Rust tests.
use rustycad_tkv3d::ais::*;
/// Parse a committed native record; `native_values("flags-0")`.
pub fn native_values(case: &str) -> Vec<f64> {
    let line = include_str!("../fixtures/view-navigation-native.txt")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(case))
        .unwrap_or_else(|| panic!("received fixture {case:?}; expected a native navigation row"));
    line.split_whitespace()
        .skip(1)
        .map(|number| {
            number
                .parse()
                .unwrap_or_else(|_| panic!("received fixture value {number:?}; expected f64"))
        })
        .collect()
}
/// Collect all public component state; `part_values(&AisWalkPart::new())`.
pub fn part_values(part: &AisWalkPart) -> [f64; 4] {
    [
        part.value,
        part.pressure,
        part.duration,
        f64::from(u8::from(part.is_empty())),
    ]
}
/// Read native query outcomes and every axis; `delta_values(&AisWalkDelta::new())`.
pub fn delta_values(delta: &AisWalkDelta) -> Vec<f64> {
    let flags = [
        delta.is_defined(),
        delta.to_move(),
        delta.to_rotate(),
        delta.is_empty(),
        delta.is_jumping(),
        delta.is_crouching(),
        delta.is_running(),
    ];
    let mut values: Vec<f64> = flags.map(|flag| f64::from(u8::from(flag))).to_vec();
    for axis in TRANSLATION_AXES {
        values.extend(part_values(&delta[axis]));
    }
    for axis in ROTATION_AXES {
        values.extend(part_values(&delta[axis]));
    }
    values
}
/// Compare IEEE values including native NaN observations; `assert_native(&values,"part-default")`.
pub fn assert_native(actual: &[f64], case: &str) {
    let expected = native_values(case);
    assert_eq!(actual.len(), expected.len(), "native record {case}");
    for (index, (left, right)) in actual.iter().zip(expected).enumerate() {
        assert!(
            *left == right || left.is_nan() && right.is_nan(),
            "received {case}[{index}]={left}; expected native {right}"
        );
        if *left == 0. && right == 0. {
            assert_eq!(left.is_sign_negative(), right.is_sign_negative());
        }
    }
}
/// Translation components in native order.
pub const TRANSLATION_AXES: [AisWalkTranslation; 3] = [
    AisWalkTranslation::Forward,
    AisWalkTranslation::Side,
    AisWalkTranslation::Up,
];
/// Rotation components in native order.
pub const ROTATION_AXES: [AisWalkRotation; 3] = [
    AisWalkRotation::Yaw,
    AisWalkRotation::Pitch,
    AisWalkRotation::Roll,
];
