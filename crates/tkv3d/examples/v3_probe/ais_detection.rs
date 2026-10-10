//! Native AIS detection trace counterpart.
use super::common::{Reader, attempt, end, flag, int};
#[path = "../../tests/support/detection_scene.rs"]
pub mod scene;
use rustycad_tkv3d::select_mgr::SelectMgrViewerSelectorResultMembers;
use scene::DetectionScene;
pub fn ais_detection_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let scene = DetectionScene::new(
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    detection_state(&scene);
    for _ in 0..r.integer() {
        let (op, value) = (r.integer(), r.integer());
        attempt(|| {
            if let Some(result) = scene.operation(op, value)? {
                int(result);
            }
            Ok(())
        });
        detection_state(&scene);
    }
    end();
}
fn detection_state(scene: &DetectionScene) {
    let fields = scene.context.fields.read();
    int(fields.current_detected);
    int(fields.current_highlighted);
    int(fields.detected.len() as i32);
    for rank in fields.detected.iter() {
        int(*rank);
    }
    drop(fields);
    flag(scene.context.has_detected());
    flag(scene.context.has_next_detected());
    flag(scene.context.more_detected());
    int(scene.owner_id(&scene.context.detected_owner()));
    int(scene.owner_id(&scene.context.detected_current_owner().unwrap()));
    flag(scene.context.has_detected() && scene.context.detected_interactive() == scene.object);
    flag(scene.context.detected_current_object().unwrap() == scene.object);
    int(scene.selector().nb_picked());
    flag(scene.context.main_prs_mgr().is_immediate_mode_on());
    flag(!scene.context.last_active_view().is_null());
    flag(scene.view.computed_mode());
    flag(scene.view.is_invalidated());
    let mut trace = scene.trace.write();
    int(trace.events.len() as i32);
    for event in trace.events.drain(..) {
        int(event);
    }
}
