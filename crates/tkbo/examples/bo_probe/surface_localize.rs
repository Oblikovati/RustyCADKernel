use super::{flag, integer, real};
use rustycad_tkbo::int_tools::{SurfaceRangeLocalizeData as Cache, SurfaceRangeSample};
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
fn point(p: Point3) {
    real(p.x());
    real(p.y());
    real(p.z());
}
fn state(cache: &Cache) {
    integer(cache.get_nb_sample_u());
    integer(cache.get_nb_sample_v());
    real(cache.get_min_range_u());
    real(cache.get_min_range_v());
    real(cache.get_grid_deflection());
    integer(cache.get_range_u_grid());
    integer(cache.get_range_v_grid());
    integer(cache.get_nb_u_points_in_frame());
    integer(cache.get_nb_v_points_in_frame());
}
fn fill(cache: &mut Cache, v: &[f64]) {
    cache.set_range_v_grid(v[1] as i32).unwrap();
    cache.set_range_u_grid(v[0] as i32).unwrap();
    for u in 1..=v[0] as i32 {
        cache.set_u_param(u, f64::from(u) * v[2]).unwrap();
    }
    for w in 1..=v[1] as i32 {
        cache.set_v_param(w, f64::from(w) * v[3]).unwrap();
    }
    for u in 1..=v[0] as i32 {
        for w in 1..=v[1] as i32 {
            cache
                .set_grid_point(u, w, Point3::new(f64::from(u), f64::from(w), -0.0))
                .unwrap();
        }
    }
}
fn frame_state(cache: &mut Cache, v: &[f64]) {
    cache.set_frame(v[4], v[5], v[6], v[7]);
    state(cache);
    for index in 1..=cache.get_nb_u_points_in_frame().max(0) + 1 {
        real(cache.get_u_param_in_frame(index).unwrap());
    }
    for index in 1..=cache.get_nb_v_points_in_frame().max(0) + 1 {
        real(cache.get_v_param_in_frame(index).unwrap());
    }
    for u in 1..=cache.get_nb_u_points_in_frame().max(0) + 1 {
        for w in 1..=cache.get_nb_v_points_in_frame().max(0) + 1 {
            point(cache.get_point_in_frame(u, w).unwrap());
        }
    }
}
fn sharing(cache: &mut Cache) {
    let mut copy = cache.clone();
    copy.set_u_param(1, 77.0).unwrap();
    copy.set_v_param(1, 78.0).unwrap();
    copy.set_grid_point(1, 1, Point3::new(79.0, 80.0, 81.0))
        .unwrap();
    real(cache.get_u_param(1).unwrap());
    real(cache.get_v_param(1).unwrap());
    point(cache.get_grid_point(1, 1).unwrap());
    copy.assign(&Cache::default());
    state(&copy);
    real(copy.get_u_param(1).unwrap());
    real(copy.get_v_param(1).unwrap());
    point(copy.get_grid_point(1, 1).unwrap());
    copy.clear_grid();
    state(&copy);
    state(cache);
}
pub fn grid_case(v: &[f64]) {
    let mut cache = Cache::default();
    state(&cache);
    frame_state(&mut cache, v);
    cache = Cache::new(3, -2, -0.0, 0.125);
    cache.set_grid_deflection(-0.25);
    fill(&mut cache, v);
    frame_state(&mut cache, v);
    cache.set_range_u_grid(v[0] as i32).unwrap();
    cache.set_range_v_grid(v[1] as i32).unwrap();
    state(&cache);
    sharing(&mut cache);
    cache.set_range_u_grid(v[0] as i32 + 1).unwrap();
    point(cache.get_grid_point(1, 1).unwrap());
    cache.set_range_v_grid(v[1] as i32 + 1).unwrap();
    point(cache.get_grid_point(1, 1).unwrap());
    cache.clear_grid();
    frame_state(&mut cache, v);
}
fn cache_state(cache: &Cache, key: &SurfaceRangeSample) {
    let mut bounds = BoundingBox3::default();
    bounds.set_whole();
    flag(cache.is_range_out(key));
    flag(cache.find_box(key, &mut bounds));
    flag(bounds.is_whole());
    flag(bounds.is_void());
}
fn listing(cache: &Cache) {
    let mut samples = vec![SurfaceRangeSample::default()];
    cache.list_range_out(&mut samples);
    cache.list_range_out(&mut samples);
    integer(samples.len() as i32);
    for key in samples {
        integer(key.get_index_u());
        integer(key.get_depth_u());
        integer(key.get_index_v());
        integer(key.get_depth_v());
    }
}
pub fn cache_case(v: &[f64]) {
    let mut cache = Cache::default();
    cache_state(&cache, &SurfaceRangeSample::default());
    for i in 0..v[0] as i32 {
        let key = SurfaceRangeSample::new(i % 137, i % 3, i % 7, i % 5);
        cache.add_box(&key, &BoundingBox3::default());
        cache.add_out_range(&key);
        cache_state(&cache, &key);
    }
    listing(&cache);
    listing(&cache.clone());
    let mut assigned = Cache::default();
    assigned.assign(&cache);
    listing(&assigned);
    cache.remove_range_out_all();
    listing(&cache);
    cache_state(&cache, &SurfaceRangeSample::default());
}
