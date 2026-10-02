//! Intermediate grouping and range flags are compared to unchanged OCCT helpers.
mod boxes;
mod grid;
mod line_plane;
use super::bean_ranges;
use rustycad_tkbo::int_tools::{
    CurveRangeLocalizeData, CurveRangeSample, MarkedRangeSet, SurfaceRangeLocalizeData,
    SurfaceRangeSample,
};
use rustycad_tkernel::standard::Failure;
fn integer(value: i32) {
    print!("{value} ");
}
fn real(value: f64) {
    print!("{} ", value.to_bits());
}
fn curve_sample(index: i32, depth: i32) -> CurveRangeSample {
    let mut sample = CurveRangeSample::new(index);
    sample.set_depth(depth);
    sample
}
fn surface_sample(index: i32, depth: i32) -> SurfaceRangeSample {
    SurfaceRangeSample::new(index, depth, index % 3, depth - 1)
}
fn snapshot(ranges: &MarkedRangeSet) -> Result<(), Failure> {
    integer(ranges.length());
    for index in 1..=ranges.length() {
        integer(ranges.flag(index)?);
        let range = ranges.range(index)?;
        real(range.first());
        real(range.last());
    }
    Ok(())
}
fn empty(values: &[f64]) -> Result<(), Failure> {
    let mut ranges =
        MarkedRangeSet::from_ranges(&[values[0], values[1], values[2]], values[3] as i32);
    for parameter in [values[4], values[5], values[4]] {
        integer(i32::from(bean_ranges::set_empty_result(
            parameter,
            &mut ranges,
        )?));
        snapshot(&ranges)?;
    }
    Ok(())
}
fn grouping(values: &[f64]) -> Result<(), Failure> {
    let curves: Vec<_> = (0..values[0] as i32)
        .map(|index| curve_sample(index, index % 4))
        .collect();
    let surfaces: Vec<_> = (0..values[1] as i32)
        .map(|index| surface_sample((index * 7) % values[2] as i32, index % 2))
        .collect();
    let mut sorted_curves = vec![curve_sample(17, 19)];
    let mut sorted_surfaces = vec![surface_sample(23, 29)];
    bean_ranges::merge_samples(&curves, &surfaces, &mut sorted_curves, &mut sorted_surfaces)?;
    integer(sorted_curves.len() as i32);
    integer(sorted_surfaces.len() as i32);
    for (curve, surface) in sorted_curves.iter().zip(&sorted_surfaces) {
        integer(curve.get_range_index());
        integer(curve.get_depth());
        integer(surface.get_index_u());
        integer(surface.get_depth_u());
        integer(surface.get_index_v());
        integer(surface.get_depth_v());
    }
    Ok(())
}
fn sampling(values: &[f64]) {
    let curve = curve_sample(0, values[0] as i32);
    let surface = SurfaceRangeSample::new(0, values[1] as i32, 0, values[2] as i32);
    let curve_cache = CurveRangeLocalizeData::new(values[3] as i32, values[6]);
    let surface_cache =
        SurfaceRangeLocalizeData::new(values[4] as i32, values[5] as i32, values[7], values[8]);
    for allowed in bean_ranges::check_sampling(
        curve,
        surface,
        &curve_cache,
        &surface_cache,
        [values[9], values[10], values[11]],
    ) {
        integer(i32::from(allowed));
    }
}
pub fn run(operation: &str, values: &[f64]) {
    let result = match operation {
        "bean_box" => boxes::run(values),
        "bean_grid" => grid::run(values),
        "bean_line_plane" => line_plane::run(values),
        "bean_empty" => empty(values),
        "bean_group" => grouping(values),
        "bean_sampling" => {
            sampling(values);
            Ok(())
        }
        _ => panic!(
            "Unknown bean operation {operation}; expected bean_box, bean_empty, bean_grid, bean_group, bean_line_plane or bean_sampling"
        ),
    };
    if let Err(error) = result {
        print!("{} ", error.exception_type());
    }
}
