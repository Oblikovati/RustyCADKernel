mod curves;
mod intersections;
mod marked;
mod records;
use rustycad_tkbo::int_tools::*;
use rustycad_tkg3d::top_abs::State;
fn real(value: f64) {
    print!("{} ", value.to_bits());
}
fn range(value: Range) {
    real(value.first());
    real(value.last());
}
fn integer(value: i32) {
    print!("{value} ");
}
fn flag(value: bool) {
    integer(i32::from(value));
}
pub fn dispatch(operation: &str, v: &[f64]) {
    dispatch_case(operation, v);
    println!();
}
fn dispatch_case(operation: &str, v: &[f64]) {
    match operation {
        "bop_range" => records::index_case(v),
        "bop_pave" => records::pave_case(v),
        "bop_pair" => records::pair_case(v),
        "bop_point" => records::point_case(v),
        "points" => intersections::points_case(v),
        "common" => intersections::common_case(v),
        "intersection_curve" => curves::case(v),
        "range" => range_case(v),
        "marked" => marked::case(v),
        "root" => root_case(v),
        "curve" => curve_case(v),
        "surface" => surface_case(v),
        _ => {
            eprintln!("Unknown probe operation: {operation}");
            std::process::exit(2);
        }
    }
}
fn range_case(v: &[f64]) {
    range(Range::default());
    let mut r = Range::new(v[0], v[1]);
    range(r);
    r.set_first(v[2]);
    r.set_last(v[3]);
    let (first, last) = r.range();
    real(first);
    real(last);
}
fn root_state(r: &Root) {
    real(r.root());
    integer(r.root_type());
    real(r.layer_height());
    integer(r.state_before() as i32);
    integer(r.state_after() as i32);
    flag(r.is_valid());
    let (t1, t2, f1, f2) = r.interval();
    for value in [t1, t2, f1, f2] {
        real(value);
    }
}
fn root_case(v: &[f64]) {
    root_state(&Root::default());
    let mut r = Root::new(v[0], v[1] as i32);
    root_state(&r);
    r.set_root(v[2]);
    r.set_type(v[3] as i32);
    r.set_layer_height(v[4]);
    r.set_state_before(State::ALL[v[5] as usize]);
    r.set_state_after(State::ALL[v[6] as usize]);
    r.set_interval(v[7], v[8], v[9], v[10]);
    root_state(&r);
}
fn curve_state(sample: &CurveRangeSample, v: &[f64]) {
    integer(sample.get_depth());
    integer(sample.get_range_index());
    integer(sample.get_range_index_deeper(v[2] as i32));
    range(sample.get_range(v[0], v[1], v[2] as i32));
    print!("{} ", sample.hash_code());
}
fn curve_case(v: &[f64]) {
    let mut base = BaseRangeSample::default();
    integer(base.get_depth());
    base = BaseRangeSample::new(v[0] as i32);
    integer(base.get_depth());
    base.set_depth(v[1] as i32);
    integer(base.get_depth());
    let mut sample = CurveRangeSample::default();
    curve_state(&sample, &v[2..]);
    sample = CurveRangeSample::new(v[5] as i32);
    sample.set_depth(v[0] as i32);
    curve_state(&sample, &v[2..]);
    flag(sample.is_equal(&sample));
    sample.set_range_index(v[6] as i32);
    sample.set_depth(v[1] as i32);
    curve_state(&sample, &v[2..]);
    flag(sample == CurveRangeSample::default());
}
fn surface_state(sample: &SurfaceRangeSample) {
    let (u, v) = sample.get_ranges();
    flag(u == *sample.get_sample_range_u());
    flag(v == *sample.get_sample_range_v());
    let (ui, vi) = sample.get_indexes();
    integer(ui);
    integer(vi);
    let (ud, vd) = sample.get_depths();
    integer(ud);
    integer(vd);
    integer(sample.get_index_u());
    integer(sample.get_index_v());
    integer(sample.get_depth_u());
    integer(sample.get_depth_v());
    integer(sample.get_range_index_u_deeper(3));
    integer(sample.get_range_index_v_deeper(2));
    range(sample.get_range_u(-3.0, 7.0, 3));
    range(sample.get_range_v(9.0, -1.0, 2));
    print!("{} ", sample.hash_code());
}
fn surface_case(v: &[f64]) {
    surface_state(&SurfaceRangeSample::default());
    let mut sample = SurfaceRangeSample::new(v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32);
    surface_state(&sample);
    let copy = sample;
    flag(copy.is_equal(&sample));
    sample.set_indexes(v[4] as i32, v[5] as i32);
    sample.set_depth_u(v[6] as i32);
    sample.set_depth_v(v[7] as i32);
    surface_state(&sample);
    sample.set_index_u(v[8] as i32);
    sample.set_index_v(v[9] as i32);
    surface_copy_cases(&sample);
}
fn surface_copy_cases(sample: &SurfaceRangeSample) {
    let (u, w) = sample.get_ranges();
    let mut other = SurfaceRangeSample::from_ranges(w, u);
    surface_state(&other);
    other.assign(sample);
    flag(other == *sample);
    other.set_ranges(w, u);
    surface_state(&other);
    other.set_sample_range_u(u);
    other.set_sample_range_v(w);
    surface_state(&other);
}
