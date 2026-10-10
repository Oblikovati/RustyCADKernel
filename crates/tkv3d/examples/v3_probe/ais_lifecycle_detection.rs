//! Inject real selector result storage to observe protected detected-rank removal.
use super::{
    ais_load::LoadScene,
    common::{flag, int},
};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkv3d::select_mgr::*;
pub(super) fn seed_detection(scene: &LoadScene, current: i32, highlighted: i32) {
    let selector = Handle::new(SelectMgrViewerSelector::new());
    {
        let mut fields = selector.viewer_selector_base().fields.write();
        for object in &scene.objects[..3] {
            fields.results.stored.add(
                SelectMgrEntityOwner::with_selectable(&object.cast(), 0).cast(),
                SelectMgrSortCriterion::default(),
            );
        }
        fields.results.indexes = Array1::with_bounds(1, 3).unwrap();
        fields
            .results
            .indexes
            .as_mut_slice()
            .copy_from_slice(&[1, 2, 3]);
        fields.results.is_sorted = true;
    }
    let manager = Handle::new(SelectMgrSelectionManager::new(&selector.cast()));
    let mut fields = scene.ctx.fields.write();
    fields.selection_manager = manager;
    fields.detected.clear(None);
    for rank in [1, 2, 1, 3, 99, 2] {
        fields.detected.append(rank);
    }
    fields.current_detected = current;
    fields.current_highlighted = highlighted;
}
pub(super) fn detection_state(scene: &LoadScene) {
    let fields = scene.ctx.fields.read();
    int(fields.current_detected);
    int(fields.current_highlighted);
    int(fields.detected.len() as i32);
    for rank in &fields.detected {
        int(*rank);
    }
    flag(fields.last_picked.is_null());
}
