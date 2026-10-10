//! Shared AIS mutations keep native captures and timed display/erase paths observable.
use super::scene::AisViewerScene;
use rustycad_tkv3d::prs_mgr::PrsMgrDisplayStatus;
pub fn apply_capture_transition(
    scene: &AisViewerScene,
    mode: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match mode {
        "ais-displayed" => {},
        "ais-erased" => scene.context.erase(&scene.object.cast(),false)?,
        "ais-redisplayed" => {scene.context.erase(&scene.object.cast(),false)?;scene.context.display(&scene.object.cast(),false)?;},
        "ais-mode1" => scene.context.display_with_modes(&scene.object.cast(),1,-1,false,PrsMgrDisplayStatus::None)?,
        "ais-affinity-off" => scene.context.set_view_affinity(&scene.object.cast(),&scene.view,false)?,
        _ => return Err(format!("received AIS transition {mode:?}; expected ais-displayed, ais-erased, ais-redisplayed, ais-mode1 or ais-affinity-off").into()),
    }
    Ok(())
}
