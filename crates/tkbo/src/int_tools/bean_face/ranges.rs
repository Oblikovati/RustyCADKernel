//! Private BeanFace range helpers from OCCT 3d097a0.
pub(crate) use super::empty_range::set_empty_result;
use crate::int_tools::{
    CurveRangeLocalizeData, CurveRangeSample, SurfaceRangeLocalizeData, SurfaceRangeSample,
};
use rustycad_tkernel::{ncollection::IndexedMap, standard::Failure};
pub(super) fn merge_samples(
    curves: &[CurveRangeSample],
    surfaces: &[SurfaceRangeSample],
    sorted_curves: &mut Vec<CurveRangeSample>,
    sorted_surfaces: &mut Vec<SurfaceRangeSample>,
) -> Result<(), Failure> {
    let mut keys = IndexedMap::<SurfaceRangeSample>::default();
    let mut groups: Vec<Vec<CurveRangeSample>> = Vec::new();
    for (curve, surface) in curves.iter().zip(surfaces) {
        let index = keys.add(*surface);
        if index > groups.len() {
            groups.push(Vec::new());
        }
        groups[index - 1].push(*curve);
    }
    for (index, curves) in groups.iter().enumerate() {
        let surface = *keys.find_key(index + 1)?;
        for curve in curves {
            sorted_surfaces.push(surface);
            sorted_curves.push(*curve);
        }
    }
    Ok(())
}
pub(super) fn check_sampling(
    curve: CurveRangeSample,
    surface: SurfaceRangeSample,
    curve_cache: &CurveRangeLocalizeData,
    surface_cache: &SurfaceRangeLocalizeData,
    differences: [f64; 3],
) -> [bool; 3] {
    [
        allow_sampling(
            curve.get_depth(),
            curve_cache.get_nb_sample(),
            curve_cache.get_min_range(),
            differences[0],
        ),
        allow_sampling(
            surface.get_depth_u(),
            surface_cache.get_nb_sample_u(),
            surface_cache.get_min_range_u(),
            differences[1],
        ),
        allow_sampling(
            surface.get_depth_v(),
            surface_cache.get_nb_sample_v(),
            surface_cache.get_min_range_v(),
            differences[2],
        ),
    ]
}
fn allow_sampling(depth: i32, samples: i32, minimum: f64, difference: f64) -> bool {
    let divisor = if depth == 0 { 1 } else { samples };
    !(f64::from(samples).powf(f64::from(depth.wrapping_add(1))) > 1000.
        || difference / f64::from(divisor) < minimum)
}
