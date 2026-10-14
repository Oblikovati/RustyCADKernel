// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn absent_nonunit_or_foreign_source_winding_is_typed_incomplete() {
    for (location, expected) in [
        (BezierWireLocation::Outside, None),
        (BezierWireLocation::Inside { winding: 0 }, Some(0)),
        (BezierWireLocation::Inside { winding: 2 }, Some(2)),
    ] {
        assert!(
            matches!(derive(RegionSourceId(101),RegionSourceId(101),false,&location,1),
            Err(AnnularDirectionCause::Winding{source:RegionSourceId(101),actual}) if actual==expected)
        );
    }
    assert!(matches!(
        derive(
            RegionSourceId(101),
            RegionSourceId(202),
            false,
            &BezierWireLocation::Inside { winding: 1 },
            1
        ),
        Err(AnnularDirectionCause::Count {
            actual: 0,
            expected: 1,
            ..
        })
    ));
}
#[test]
fn boundary_lookup_requires_unique_full_source_role() {
    let row = derive(
        RegionSourceId(101),
        RegionSourceId(101),
        false,
        &BezierWireLocation::Inside { winding: 1 },
        1,
    )
    .unwrap();
    assert!(matches!(
        boundary(&[row, row], RegionSourceId(101)),
        Err(AnnularDirectionCause::Count {
            actual: 2,
            expected: 1,
            ..
        })
    ));
    assert!(matches!(
        boundary(&[row, row], RegionSourceId(202)),
        Err(AnnularDirectionCause::Count {
            actual: 0,
            expected: 1,
            ..
        })
    ));
}
