use super::{flag, integer, real};
use rustycad_tkbo::int_tools::{CurveRangeLocalizeData, CurveRangeSample};
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
fn sample(index: i32, depth: i32) -> CurveRangeSample {
    let mut result = CurveRangeSample::new(index);
    result.set_depth(depth);
    result
}
fn bounds_state(bounds: &BoundingBox3) {
    flag(bounds.is_void());
    flag(bounds.is_whole());
    real(bounds.gap());
    if !bounds.is_void() && !bounds.is_whole() {
        for p in [bounds.corner_min().unwrap(), bounds.corner_max().unwrap()] {
            real(p.x());
            real(p.y());
            real(p.z());
        }
    }
}
fn sample_state(cache: &CurveRangeLocalizeData, key: &CurveRangeSample) {
    let mut bounds = BoundingBox3::default();
    bounds.set_whole();
    flag(cache.is_range_out(key));
    flag(cache.find_box(key, &mut bounds));
    bounds_state(&bounds);
}
fn apply(cache: &mut CurveRangeLocalizeData, step: i32, depth: i32) {
    let key = sample(step % 137 - 30, depth + step % 3);
    let mut bounds = BoundingBox3::default();
    bounds.set_point(Point3::new(f64::from(step), -0.0, -f64::from(step)));
    bounds.set_gap(0.25);
    cache.add_box(&key, &bounds);
    sample_state(cache, &key);
    cache.add_out_range(&key);
    sample_state(cache, &key);
    if step % 2 == 0 {
        cache.add_box(&key, &BoundingBox3::default());
        cache.add_box(&key, &bounds);
    }
    sample_state(cache, &key);
}
pub fn case(v: &[f64]) {
    let mut cache = CurveRangeLocalizeData::new(v[0] as i32, v[1]);
    integer(cache.get_nb_sample());
    real(cache.get_min_range());
    sample_state(&cache, &CurveRangeSample::default());
    for step in 0..v[2] as i32 {
        apply(&mut cache, step, v[3] as i32);
    }
    let mut output = vec![sample(999, -1)];
    cache.list_range_out(&mut output);
    cache.list_range_out(&mut output);
    integer(output.len() as i32);
    for key in output {
        integer(key.get_range_index());
        integer(key.get_depth());
    }
}
