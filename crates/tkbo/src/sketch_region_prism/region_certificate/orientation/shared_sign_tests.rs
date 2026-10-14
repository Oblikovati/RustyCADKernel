// SPDX-License-Identifier: GPL-2.0-only
//! Literal induced-edge expectations pin the shared primitive before its annular consumer.
use super::*;
use Orientation::{Forward as F, Reversed as R};

#[test]
fn shared_trim_rule_preserves_all_stored_rectangle_and_cap_signs() {
    let roles = [
        PrismTrimRole::First,
        PrismTrimRole::Last,
        PrismTrimRole::Start,
        PrismTrimRole::End,
    ];
    for (native, facing, side, cap) in [
        (1, 1, [R, F, F, R], [F; 4]),
        (1, -1, [F, R, R, F], [R; 4]),
        (-1, 1, [R, F, F, R], [R; 4]),
        (-1, -1, [F, R, R, F], [F; 4]),
    ] {
        for (face, expected) in [
            (PrismImageRole::Generated, side),
            (PrismImageRole::First, cap),
            (PrismImageRole::Last, cap),
        ] {
            assert_eq!(
                roles.map(|trim| expected_trim_direction(face, trim, native, facing)),
                expected
            );
        }
    }
}

#[test]
fn shared_outward_rule_preserves_the_v1_literal_sign_table() {
    let roles = [
        PrismImageRole::First,
        PrismImageRole::Last,
        PrismImageRole::Generated,
    ];
    for (native, jacobian, expected) in [
        (1, 1, [R, F, R]),
        (1, -1, [F, R, F]),
        (-1, 1, [R, F, F]),
        (-1, -1, [F, R, R]),
    ] {
        assert_eq!(
            roles.map(|role| signed_prism_orientation(prism_outward_sign(role, native, jacobian))),
            expected
        );
    }
}
