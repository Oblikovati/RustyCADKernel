//! Native ordered BVH box selection, accumulation and clear semantics.
use rustycad_tkbo::bop_tools::{BoxSelector, BoxSet};
use rustycad_tkmath::bvh::BvhBinnedBuilder;
use rustycad_tkmath::bvh::{
    BvhBox, BvhBoxSet, BvhSetTraverse, BvhTraverse, BvhVec2d, BvhVec3d, BvhVector,
};
use std::sync::Arc;
fn point<V: BvhVector<Scalar = f64>>(x: f64) -> V {
    let mut point = V::default();
    point[0] = x;
    point
}
#[test]
fn native_bvh_box_selection_matches() {
    let fixture = include_str!("../../../tools/oracle/tkbo_box_selector.expected");
    assert_eq!(fixture.lines().count(), 30);
    for line in fixture.lines() {
        let row: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let actual = if row[0] == 2 {
            observe::<BvhVec2d>(&row, 0)
        } else {
            observe::<BvhVec3d>(&row, 0)
        };
        assert_eq!(actual, row[3..], "{line}");
    }
}
fn observe<V: BvhVector<Scalar = f64>>(row: &[i32], builder: usize) -> Vec<i32> {
    let mut standard = BvhBoxSet::<V>::default();
    let mut wrapped = BoxSet::<V>::new(if builder == 2 {
        Some(Arc::new(BvhBinnedBuilder::<V, 48>::default()))
    } else {
        None
    });
    let boxes = if builder == 0 {
        &mut standard
    } else {
        &mut *wrapped
    };
    for index in 0..12 {
        boxes.add(
            100 - index,
            BvhBox::from_corners(point(f64::from(index)), point(f64::from(index) + 0.75)),
        );
    }
    let mut selector = BoxSelector::default();
    selector.set_bvh_set(Some(boxes));
    let bounds = [[-1., 20.], [2.25, 5.25], [99., 100.], [4., 4.]];
    selector.set_box(if row[1] == 4 {
        BvhBox::default()
    } else {
        let range = bounds[row[1] as usize];
        BvhBox::from_corners(point(range[0]), point(range[1]))
    });
    let first = selector.select().unwrap();
    let mut second = -1;
    if row[2] == 2 {
        selector.clear();
    }
    if row[2] != 0 {
        second = selector.select().unwrap();
    }
    let mut inside = true;
    let rejected = selector.reject_node(&point(3.), &point(4.), &mut inside);
    let mut actual = vec![
        first,
        second,
        i32::from(rejected),
        i32::from(inside),
        i32::from(selector.accept_metric(&inside)),
        selector.indices().extent(),
    ];
    actual.extend(selector.indices().iter().copied());
    actual
}
#[test]
fn absent_set_selects_nothing_and_reports_direct_access() {
    let mut selector = BoxSelector::<BvhVec3d>::default();
    assert_eq!(selector.select().unwrap(), 0);
    assert_eq!(
        selector.reject_element(7).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn native_linear_default_and_supplied_builder_match() {
    for mode in [1, 2] {
        let fixture = if mode == 1 {
            include_str!("../../../tools/oracle/tkbo_linear_box_selector.expected")
        } else {
            include_str!("../../../tools/oracle/tkbo_box_selector.expected")
        };
        assert_eq!(fixture.lines().count(), 30);
        for line in fixture.lines() {
            let row: Vec<i32> = line
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            let actual = if row[0] == 2 {
                observe::<BvhVec2d>(&row, mode)
            } else {
                observe::<BvhVec3d>(&row, mode)
            };
            assert_eq!(actual, row[3..], "builder {mode}: {line}");
        }
    }
}
