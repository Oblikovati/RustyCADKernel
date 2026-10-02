mod algorithm_cases;
mod context;
mod context_compute;
mod curves;
mod edge_cases;
mod face_face;
mod face_perform;
mod fclass;
mod fclass_faces;
mod intersections;
mod localize;
mod marked;
mod records;
mod shrunk_range;
mod surface_localize;
mod tools;
mod tools_curves;
mod tools_geometry;
mod tools_surface;
mod tools_topology;
mod topol;
mod topol_domain;
mod topol_surfaces;
mod wline;
mod wline_cases;
mod wline_decompose;
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
        "algorithms_roots" => algorithm_cases::roots(v),
        "algorithms_geometry" => algorithm_cases::geometry(v),
        "edge_full" => edge_cases::case(v),
        "shrunk" => shrunk_range::case(v),
        "bop_range" => records::index_case(v),
        "bop_pave" => records::pave_case(v),
        "bop_pair" => records::pair_case(v),
        "bop_point" => records::point_case(v),
        "bop_pave_block" => records::pave_block_case(v),
        "bop_common_block" => records::common_block_case(v),
        "bop_couple_paves" => records::couple_of_pave_blocks_case(v),
        "bop_shape_info" => records::shape_info_case(v),
        "bop_face_info" => records::face_info_case(v),
        "bop_data_structure" => records::data_structure_case(v),
        "bop_data_structure_common_block" => records::data_structure_common_block_case(v),
        "bop_data_structure_init" => records::data_structure_init_case(v),
        "bop_data_structure_edges" => records::data_structure_edges_case(v),
        "bop_data_structure_box" => records::data_structure_box_case(v),
        "bop_data_structure_infinite" => records::data_structure_infinite_case(v),
        "bop_data_structure_seam" => records::data_structure_seam_case(v),
        "points" => intersections::points_case(v),
        "common" => intersections::common_case(v),
        "intersection_curve" => curves::case(v),
        "localize" => localize::case(v),
        "surface_grid" => surface_localize::grid_case(v),
        "surface_cache" => surface_localize::cache_case(v),
        _ => dispatch_scalar(operation, v),
    }
}
fn dispatch_scalar(operation: &str, v: &[f64]) {
    match operation {
        "face_parameter" => face_parameter_case::case(v),
        "face_analytic" => face_parameter_case::analytic_case(v),
        "face_distance" => face_surface_case::distance_case(v),
        "face_check" => face_pcurve_case::case(v),
        "face_bounds" => face_boundaries_case::case(v),
        "face_policy" => face_surface_case::case(v),
        "face_pcurves" => face_surface_case::pcurves_case(v),
        "face_planes" => face_planes_case::case(v),
        "face_clip" => face_face::clip_case(v),
        "face_perform" => face_perform::case(v),
        "face_state" => face_face::case(v),
        "context_compute" => context_compute::case(v),
        "tools_curves" => tools_curves::case(v),
        "tools_topology" => tools_topology::case(v),
        "tools_surface" => tools_surface::case(v),
        "wline_decompose" => wline_decompose::case(v),
        "wline" => wline::case(v),
        "tools" => tools::case(v),
        "context" => context::case(v),
        "topol" => topol::case(v),
        "fclass" => fclass::case(v),
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

#[path = "../../src/int_tools/face_face/plane_clip.rs"]
mod plane_clip;

#[path = "../../src/int_tools/face_face/planes.rs"]
mod face_planes;
mod face_planes_case;

mod face_surface_case;
#[path = "../../src/int_tools/face_face/surface_policy.rs"]
mod face_surface_policy;

#[path = "../../src/int_tools/face_face/boundaries.rs"]
mod face_boundaries;
mod face_boundaries_case;

mod face_pcurve_case;
#[path = "../../src/int_tools/face_face/pcurve_checks.rs"]
mod face_pcurve_checks;

#[path = "../../src/int_tools/face_face/distance.rs"]
mod face_distance;

#[path = "../../src/int_tools/face_face/analytic.rs"]
mod face_analytic;
#[path = "../../src/int_tools/face_face/boundary_parameter.rs"]
mod face_boundary_parameter;
mod face_parameter_case;
