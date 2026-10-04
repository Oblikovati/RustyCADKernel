//! Native BVH pair ordering, positional filtering and accumulation.
use rustycad_tkbo::bop_tools::{PairIds, PairSelector};
use rustycad_tkmath::bvh::{BvhBox, BvhBoxSet, BvhPairTraverse, BvhVec3d};
fn boxes(offset: i32, second: bool) -> BvhBoxSet<BvhVec3d> {
    let mut boxes = BvhBoxSet::default();
    for i in 0..12 {
        let shift = if second { f64::from(offset) * 0.5 } else { 0. };
        let length = if second { 0.75 } else { 1.25 };
        boxes.add(
            if second { i % 3 } else { 100 - i },
            BvhBox::from_corners(
                BvhVec3d::new(f64::from(i) + shift, 0., 0.),
                BvhVec3d::new(f64::from(i) + shift + length, 1., 1.),
            ),
        );
    }
    boxes
}
#[test]
fn native_bvh_pair_selection_matches() {
    let fixture = include_str!("../../../tools/oracle/tkbo_pair_selector.expected");
    assert_eq!(fixture.lines().count(), 36);
    for line in fixture.lines() {
        let row: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(observe(&row), row[4..], "{line}");
    }
}
fn observe(row: &[i32]) -> Vec<i32> {
    let mut first = boxes(0, false);
    let mut second = boxes(row[3], true);
    let mut selector = PairSelector::default();
    if row[1] == 0 {
        selector.set_bvh_sets(&mut first, &mut second);
    } else {
        selector.set_shared_bvh_set(&mut first);
    }
    selector.set_same(row[0] != 0);
    let first_count = selector.select().unwrap();
    if row[2] == 2 {
        selector.clear();
    }
    let second_count = if row[2] == 0 {
        -1
    } else {
        selector.select().unwrap()
    };
    let mut metric = 42.;
    let rejected = selector.reject_node(
        &BvhVec3d::new(0., 0., 0.),
        &BvhVec3d::new(1., 1., 1.),
        &BvhVec3d::new(2., 0., 0.),
        &BvhVec3d::new(3., 1., 1.),
        &mut metric,
    );
    let mut actual = vec![
        first_count,
        second_count,
        i32::from(rejected),
        metric as i32,
        selector.pairs().len() as i32,
    ];
    append_pairs(&selector, &mut actual);
    selector.sort();
    append_pairs(&selector, &mut actual);
    actual
}
fn append_pairs(selector: &PairSelector<'_>, actual: &mut Vec<i32>) {
    for pair in selector.pairs().iter() {
        actual.extend([pair.id1, pair.id2]);
    }
}
#[test]
fn pair_ids_and_absent_sets_retain_source_defaults() {
    assert_eq!(PairIds::default(), PairIds::new(-1, -1));
    assert!(PairIds::new(1, 9) < PairIds::new(2, 0));
    assert!(PairIds::new(1, 2) < PairIds::new(1, 3));
    let mut selector = PairSelector::default();
    assert_eq!(selector.select().unwrap(), 0);
    selector.set_same(true);
    assert!(selector.reject_element(1, 0).unwrap());
    selector.clear_bvh_sets();
    assert_eq!(selector.select().unwrap(), 0);
    assert_eq!(
        selector.reject_element(0, 1).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
}
