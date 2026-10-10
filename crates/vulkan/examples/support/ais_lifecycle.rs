//! Capture transitions use actual context methods so retained and destroyed presentations differ.
use super::scene::AisViewerScene;
use rustycad_tkv3d::prs_mgr::PrsMgrDisplayStatus;
pub fn apply_capture_transition(
    scene: &AisViewerScene,
    mode: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match mode {
        "ais-displayed" => {}
        "ais-erased" => scene.context.erase(&scene.object.cast(), false)?,
        "ais-redisplayed" => {
            scene.context.erase(&scene.object.cast(), false)?;
            scene.context.display(&scene.object.cast(), false)?;
        }
        "ais-mode1" => scene.context.display_with_modes(
            &scene.object.cast(),
            1,
            -1,
            false,
            PrsMgrDisplayStatus::None,
        )?,
        "ais-affinity-off" => {
            scene
                .context
                .set_view_affinity(&scene.object.cast(), &scene.view, false)?
        }
        _ => apply_bulk_transition(scene, mode)?,
    }
    Ok(())
}
fn apply_bulk_transition(
    scene: &AisViewerScene,
    mode: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match mode {
        "ais-cleared" => scene.context.clear_prs(&scene.object.cast(), 0, false)?,
        "ais-removed" => scene.context.remove(&scene.object.cast(), false)?,
        "ais-recreated" => {
            scene.context.remove(&scene.object.cast(), false)?;
            scene.context.display(&scene.object.cast(), false)?;
        }
        "ais-erased-all" => scene.context.erase_all(false)?,
        "ais-displayed-all" => {
            scene.context.erase_all(false)?;
            scene.context.display_all(false)?;
        }
        "ais-removed-all" => scene.context.remove_all(false)?,
        "ais-erased-selected" | "ais-displayed-selected" => apply_selected_transition(scene, mode)?,
        _ => {
            return Err(format!(
                "received AIS transition {mode:?}; expected a supported ais-* capture mode"
            )
            .into());
        }
    }
    Ok(())
}
fn apply_selected_transition(
    scene: &AisViewerScene,
    mode: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    scene.context.set_automatic_hilight(false);
    if mode == "ais-displayed-selected" {
        scene.context.erase(&scene.object.cast(), false)?;
    }
    scene.context.add_select_object(&scene.object.cast())?;
    if mode == "ais-displayed-selected" {
        scene.context.display_selected(false)?;
    } else {
        scene.context.erase_selected(false)?;
    }
    Ok(())
}
