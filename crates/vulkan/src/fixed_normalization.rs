//! Native normalization comes from structure/view state, not arbitrary group matrices.
use rustycad_tkernel::ncollection::Mat4;
use rustycad_tkservice::graphic3d::*;
pub(crate) fn native_normalize_state(
    camera: &Graphic3dCamera,
    fields: &Graphic3dCStructureFields,
    view: [f32; 16],
    local_persistence: bool,
) -> bool {
    let axial = camera.axial_scale();
    if [axial.x(), axial.y(), axial.z()]
        .into_iter()
        .any(|v| v != 1.)
    {
        return true;
    }
    if fields
        .trsf
        .get()
        .is_some_and(|datum| scale_enables_normalization(datum.transformation().scale_factor()))
    {
        return true;
    }
    (!fields.trsf_pers.is_null() || local_persistence)
        && scale_enables_normalization(transform_utils::scale_factor(&Mat4::from_data(view)))
}
fn scale_enables_normalization(scale: f64) -> bool {
    (scale - 1.).abs() > rustycad_tkernel::precision::confusion()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_scale_predicate_matches_native_confusion_and_mirroring() {
        assert!(!scale_enables_normalization(1.));
        assert!(!scale_enables_normalization(1. + 1e-8));
        assert!(scale_enables_normalization(0.75));
        assert!(scale_enables_normalization(-1.));
        assert!(!scale_enables_normalization(f64::NAN));
    }
}
