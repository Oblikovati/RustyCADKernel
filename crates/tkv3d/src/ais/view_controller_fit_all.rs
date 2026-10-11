//! Native FitAllAuto alternates selected geometry and scene using a size-relative tolerance.
use super::{AisInteractiveContext, AisViewController};
use crate::v3d::V3dView;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkservice::graphic3d::Graphic3dCamera;
impl AisViewController {
    /// Fit selection, switching to the scene when already fitted; `controller.fit_all_auto(&context, &view)?`.
    pub fn fit_all_auto(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
    ) -> Result<(), Failure> {
        let selected = context.bounding_box_of_selection(Some(view))?;
        if selected.is_void() {
            return view.fit_all(0.01, false);
        }
        let tolerance =
            (selected.corner_max()?.xyz() - selected.corner_min()?.xyz()).modulus() * 0.000001;
        fit_selection_or_scene(view, &selected, tolerance)?;
        Ok(())
    }
}
fn fit_selection_or_scene(
    view: &V3dView,
    selected: &BoundingBox3,
    tolerance: f64,
) -> Result<(), Failure> {
    let scene = view.view().min_max_values(false)?;
    let camera = view.camera();
    let selection_camera = fit_candidate_camera(view, selected)?;
    let scene_camera = fit_candidate_camera(view, &scene)?;
    if camera_matches_selected_fit(&selection_camera, &camera, tolerance) {
        camera.copy(&scene_camera);
    } else {
        camera.copy(&selection_camera);
    }
    Ok(())
}
fn fit_candidate_camera(view: &V3dView, bounds: &BoundingBox3) -> Result<Graphic3dCamera, Failure> {
    let camera = view.camera().copy_members();
    // OCCT 3d097a0 deliberately ignores FitMinMax's false result for a void scene.
    view.fit_min_max(&camera, bounds, 0.01, 0.0, true)?;
    Ok(camera)
}

fn camera_matches_selected_fit(
    selected: &Graphic3dCamera,
    current: &Graphic3dCamera,
    tolerance: f64,
) -> bool {
    selected.center().is_equal(&current.center(), tolerance)
        && (selected.scale() - current.scale()).abs() < tolerance
        && (selected.distance() - current.distance()).abs() < tolerance
}
