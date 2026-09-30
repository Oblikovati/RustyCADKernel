//! Planar bounding boxes retain native gap and overlap behavior in tree bounds.
use rustycad_tkernel::ncollection::BoundingVolume;
use rustycad_tkmath::{bnd::BoundingBox2, gp::Point2};
#[test]
fn planar_tree_bounds_delegate_union_overlap_and_extent() {
    let mut first = BoundingBox2::default();
    first.add_point(Point2::new(0.0, 0.0));
    first.add_point(Point2::new(1.0, 1.0));
    let mut second = BoundingBox2::default();
    second.add_point(Point2::new(3.0, 4.0));
    assert!(BoundingVolume::is_out(&first, &second));
    BoundingVolume::add(&mut first, &second);
    assert!(!BoundingVolume::is_out(&first, &second));
    assert_eq!(BoundingVolume::square_extent(&first), 25.0);
    first.enlarge(0.5);
    assert_eq!(BoundingVolume::square_extent(&first), 41.0);
}
