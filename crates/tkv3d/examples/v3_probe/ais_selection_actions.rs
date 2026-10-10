//! Compare Martensite selection callbacks and state with the pinned OCCT fixture.
use super::{
    ais_detection::scene as detection_scene,
    common::{Reader, attempt, end, flag, int},
};
#[path = "../../tests/support/selection_actions_scene.rs"]
pub mod scene;
use rustycad_tkv3d::{ais::*, select_mgr::*};
use scene::{SelectionActionsScene, style_mode};
pub fn ais_selection_actions_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let scene = SelectionActionsScene::new(
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    selection_state(&scene);
    for _ in 0..r.integer() {
        let (op, a, b) = (r.integer(), r.integer(), r.integer());
        attempt(|| {
            for result in scene.operation(op, a, b)? {
                int(result);
            }
            Ok(())
        });
        selection_state(&scene);
    }
    end();
}
fn selection_state(scene: &SelectionActionsScene) {
    let ctx = &scene.detection.context;
    int(ctx.nb_selected());
    for owner in ctx.selection().objects().iter() {
        int(scene.detection.owner_id(owner));
    }
    for owner in &scene.detection.owners {
        flag(ctx.is_selected_owner(owner));
    }
    flag(ctx.is_selected_object(&scene.detection.object.cast()));
    flag(ctx.more_selected());
    int(scene.detection.owner_id(&ctx.selected_owner().unwrap()));
    flag(ctx.selected_interactive().unwrap() == scene.detection.object);
    flag(ctx.first_selected_object() == scene.detection.object);
    flag(ctx.is_hilighted_object(&scene.detection.object.cast()));
    int(style_mode(&scene.status.hilight_style()));
    flag(ctx.main_prs_mgr().is_immediate_mode_on());
    flag(!ctx.last_active_view().is_null());
    flag(scene.detection.view.is_invalidated());
    int(scene.detection.selector().nb_picked());
    let mut trace = scene.detection.trace.write();
    int(trace.events.len() as i32);
    for event in trace.events.drain(..) {
        int(event);
    }
}
