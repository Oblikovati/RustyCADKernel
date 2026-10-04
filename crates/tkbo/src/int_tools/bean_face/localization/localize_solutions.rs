//! Recursive curve/surface box subdivision from `LocalizeSolutions`.
use super::{BeanFaceIntersector, Failure};
use crate::int_tools::{
    CurveRangeLocalizeData, CurveRangeSample, SurfaceRangeLocalizeData, SurfaceRangeSample,
};
use rustycad_tkmath::bnd::BoundingBox3;
mod localizer;
mod subdivision;
pub(super) type SamplePair = (CurveRangeSample, SurfaceRangeSample);
pub(super) fn compute(
    bean: &BeanFaceIntersector,
    curve_cache: &mut CurveRangeLocalizeData,
    surface_cache: &mut SurfaceRangeLocalizeData,
    curve_box: &BoundingBox3,
    surface_box: &BoundingBox3,
    output: &mut Vec<SamplePair>,
) -> Result<bool, Failure> {
    localizer::compute(
        bean,
        curve_cache,
        surface_cache,
        curve_box,
        surface_box,
        output,
    )
}
