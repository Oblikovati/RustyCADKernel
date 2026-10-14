// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn source_winding_traversal_and_material_tags_match_literal_bytes() {
    let row = AnnularBoundaryDirection {
        source: RegionSourceId(101),
        traversed_winding: -1,
        reversed: true,
        native_winding: 1,
        material_side: -1,
    };
    let mut actual = Vec::new();
    rows::boundary(&mut actual, &row);
    assert_eq!(
        actual,
        [
            101, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 1, 1, 255
        ]
    );
}
#[test]
fn direction_and_role_codes_do_not_use_rust_enum_layout() {
    let mut actual = Vec::new();
    for value in [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ] {
        direction(&mut actual, value);
    }
    append_role_codes(&mut actual);
    assert_eq!(actual, [1, 2, 3, 4, 1, 2, 3, 1, 2, 3, 4]);
}
#[test]
fn original_parent_rosters_keep_exact_order_and_duplicate_multiplicity() {
    let row = FrozenOriginalKey {
        source: 1,
        member: AuthoredPrismId(2),
        directed_use: AuthoredPrismId(3),
    };
    let mut actual = Vec::new();
    parents(&mut actual, &[row.clone(), row]);
    let expected = [2_u64, 1, 2, 3, 1, 2, 3]
        .into_iter()
        .flat_map(u64::to_le_bytes)
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn append_role_codes(actual: &mut Vec<u8>) {
    for value in [
        PrismImageRole::First,
        PrismImageRole::Last,
        PrismImageRole::Generated,
    ] {
        image_role(actual, value);
    }
    for value in [
        PrismTrimRole::First,
        PrismTrimRole::Last,
        PrismTrimRole::Start,
        PrismTrimRole::End,
    ] {
        trim_role(actual, value);
    }
}
