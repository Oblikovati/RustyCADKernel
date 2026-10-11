//! Shared analytic selection policy preserves callback order and untouched nonpoint outputs.
use super::Select3DSensitiveEntity;
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrSelectionType;
use rustycad_tkernel::standard::Failure;

pub(super) trait AnalyticQuadricSelection: Select3DSensitiveEntity {
    fn quadric_point_overlap(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    fn quadric_inside_overlap(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure>;
    fn retain_quadric_point(&self, _pick: &SelectBasicsPickResult) {
        // Only spheres retain detected points; circles and cylinders remain stateless.
        // This callback must run before distance calculation can fail.
    }
}
pub(super) fn match_analytic_quadric<S: AnalyticQuadricSelection>(
    entity: &S,
    volume: &dyn SelectBasicsSelectingVolumeManager,
    pick: &mut SelectBasicsPickResult,
) -> Result<bool, Failure> {
    if volume.get_active_selection_type() != SelectMgrSelectionType::Point {
        return match_quadric_nonpoint(entity, volume);
    }
    if !entity.quadric_point_overlap(volume, pick)? {
        return Ok(false);
    }
    entity.retain_quadric_point(pick);
    pick.set_dist_to_geom_center(volume.dist_to_geometry_center(&entity.center_of_geometry())?);
    Ok(true)
}
fn match_quadric_nonpoint<S: AnalyticQuadricSelection>(
    entity: &S,
    volume: &dyn SelectBasicsSelectingVolumeManager,
) -> Result<bool, Failure> {
    if volume.is_overlap_allowed() {
        return entity.quadric_inside_overlap(volume, None);
    }
    let mut inside = true;
    Ok(entity.quadric_inside_overlap(volume, Some(&mut inside))? && inside)
}
