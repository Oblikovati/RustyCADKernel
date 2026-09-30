//! Cache invalidation and native map hashing are observable parts of localization.
use rustycad_tkbo::int_tools::{CurveRangeLocalizeData, CurveRangeSample};
use rustycad_tkernel::ncollection::HashKey;
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
#[test]
fn localization_preserves_settings_and_miss_output() {
    let cache = CurveRangeLocalizeData::new(-2, -0.0);
    assert_eq!(cache.get_nb_sample(), -2);
    assert_eq!(cache.get_min_range().to_bits(), (-0.0f64).to_bits());
    let sample = CurveRangeSample::new(7);
    let mut bounds = BoundingBox3::default();
    bounds.set_whole();
    assert!(!cache.find_box(&sample, &mut bounds));
    assert!(bounds.is_whole());
    assert!(!cache.is_range_out(&sample));
    assert_eq!(sample.occt_hash(), sample.hash_code());
}
#[test]
fn exclusion_invalidates_box_but_new_box_keeps_exclusion() {
    let mut cache = CurveRangeLocalizeData::new(4, 0.1);
    let sample = CurveRangeSample::new(3);
    let mut bounds = BoundingBox3::default();
    bounds.set_point(Point3::new(1.0, 2.0, 3.0));
    cache.add_box(&sample, &bounds);
    assert!(cache.find_box(&sample, &mut bounds));
    cache.add_out_range(&sample);
    assert!(!cache.find_box(&sample, &mut bounds));
    cache.add_box(&sample, &BoundingBox3::default());
    assert!(cache.is_range_out(&sample));
    assert!(cache.find_box(&sample, &mut bounds));
    assert!(bounds.is_void());
}
#[test]
fn localization_listing_appends_unique_exclusions() {
    let mut cache = CurveRangeLocalizeData::new(4, 0.1);
    let sample = CurveRangeSample::new(9);
    cache.add_out_range(&sample);
    cache.add_out_range(&sample);
    let mut listed = vec![CurveRangeSample::default()];
    cache.list_range_out(&mut listed);
    cache.list_range_out(&mut listed);
    assert_eq!(listed, vec![CurveRangeSample::default(), sample, sample]);
}
