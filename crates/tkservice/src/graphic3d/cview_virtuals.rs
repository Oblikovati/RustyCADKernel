//! The bodies of the virtual members of `Graphic3d_CView` (the trait defaults), and of the
//! pure members with a base body (`Resized`, `DiagnosticInformation`) that drivers call.
use super::cview_xr::Graphic3dCViewXr as _;
use super::{
    Graphic3dCView, Graphic3dCViewMembers as _, Graphic3dCViewStructures as _,
    Graphic3dCViewSubviews as _, Graphic3dCamera, Graphic3dCameraFocusType, Graphic3dCameraIodType,
    Graphic3dCameraProjection, Graphic3dDiagnosticInfo, Graphic3dFrameStatsDict,
    Graphic3dStereoMode, Graphic3dTypeOfAnswer as Answer, Graphic3dZLayerId,
};
use crate::aspect::{
    AspectEye, AspectNeutralWindow, AspectOpenVRSession, AspectXRSession,
    AspectXRSessionInfoString, AspectXRSessionMembers as _,
};
use crate::cast::int_from_f64;
use crate::graphic3d::frame_stats_text::add_info_text;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;

/// The structures the manager displays and whether this view shows them.
fn manager_structures<V: Graphic3dCView + ?Sized>(
    view: &V,
) -> Vec<Handle<dyn super::Graphic3dStructureSource>> {
    view.structure_manager()
        .get()
        .map(|m| m.displayed_structures())
        .unwrap_or_default()
}

/// Whether the view accepts a structure (as is or computed).
fn accepted<V: Graphic3dCView + ?Sized>(
    view: &V,
    structure: &Handle<dyn super::Graphic3dStructureSource>,
) -> bool {
    let visual = structure
        .get()
        .map(|s| s.structure_base().fields.read().visual);
    visual.is_some_and(|v| matches!(view.accept_display(v), Answer::Yes | Answer::Compute))
}

/// The native `Activate`.
pub(crate) fn activate<V: Graphic3dCView + ?Sized>(view: &V) {
    if !view.is_active() {
        view.cview_base().fields.write().is_active = true;
        for structure in manager_structures(view) {
            if !view.is_displayed(&structure) && accepted(view, &structure) {
                view.display(&structure);
            }
        }
    }
    view.update(super::GRAPHIC3D_ZLAYER_ID_UNKNOWN);
}

/// The native `Deactivate`.
pub(crate) fn deactivate<V: Graphic3dCView + ?Sized>(view: &V) {
    if !view.is_active() {
        return;
    }
    for structure in manager_structures(view) {
        if view.is_displayed(&structure) && accepted(view, &structure) {
            view.erase(&structure);
        }
    }
    view.update(super::GRAPHIC3D_ZLAYER_ID_UNKNOWN);
    view.cview_base().fields.write().is_active = false;
}

/// The native `Remove`, retaining the outer driver view identity; `graphic3d_cview_remove(view)`.
pub fn remove<V: Graphic3dCView + ?Sized>(view: &V) {
    if view.is_removed() {
        return;
    }
    let address = std::ptr::from_ref(view).cast::<()>();
    let parent = std::mem::take(&mut view.cview_base().fields.write().parent_view).upgrade();
    if let Some(parent) = parent.get() {
        parent.remove_subview(address);
    }
    let subviews = std::mem::take(&mut view.cview_base().fields.write().subviews);
    for subview in subviews.iter().filter_map(Handle::get) {
        subview.remove();
    }
    for structure in view.displayed_structures() {
        view.erase(&structure);
    }
    let (manager, id) = {
        let mut fields = view.cview_base().fields.write();
        fields.structs_to_compute.clear();
        fields.structs_computed.clear();
        fields.structs_displayed.clear(true);
        (fields.structure_manager.clone(), fields.id)
    };
    if let Some(manager) = manager.get() {
        manager.un_identification(address, id);
    }
    let mut fields = view.cview_base().fields.write();
    fields.is_active = false;
    fields.is_removed = true;
}

/// The native `MinMaxValues(bool)`.
pub(crate) fn min_max_values<V: Graphic3dCView + ?Sized>(
    view: &V,
    include_auxiliary: bool,
) -> Result<BoundingBox3, Failure> {
    let mut result = BoundingBox3::default();
    if !view.is_defined() {
        return Ok(result);
    }
    let camera = view.camera();
    let size = view.window().get().map_or([0, 0], |w| {
        let (width, height) = w.size();
        [width, height]
    });
    for layer in view.layers().iter().filter_map(Handle::get) {
        result.add_box(&layer.bounding_box(
            view.identification(),
            &camera,
            size,
            include_auxiliary,
        )?);
    }
    Ok(result)
}

/// The native `InvalidateZLayerBoundingBox`.
pub(crate) fn invalidate_z_layer_bounding_box<V: Graphic3dCView + ?Sized>(
    view: &V,
    layer_id: Graphic3dZLayerId,
) {
    if let Some(layer) = view.layer(layer_id).get() {
        layer.invalidate_bounding_box();
        return;
    }
    for layer in view.layers().iter().filter_map(Handle::get) {
        if layer.nb_of_transform_persistence_objects() > 0 {
            layer.invalidate_bounding_box();
        }
    }
}

/// The native `CopySettings`.
pub(crate) fn copy_settings<V: Graphic3dCView + ?Sized>(
    view: &V,
    other: &Handle<dyn Graphic3dCView>,
) -> Result<(), Failure> {
    let Some(other) = other.get() else {
        return Ok(());
    };
    let params = other.rendering_params();
    view.change_rendering_params(&mut |p| *p = params.clone());
    view.set_background(&other.background());
    view.set_gradient_background(&other.gradient_background());
    view.set_background_image(&other.background_image(), true);
    view.set_background_image_style(other.background_image_style());
    view.set_texture_env(&other.texture_env());
    view.set_shading_model(other.shading_model())?;
    view.set_backfacing_model(other.backfacing_model());
    let camera = other
        .camera()
        .get()
        .map(Graphic3dCamera::from_camera)
        .unwrap_or_default();
    view.set_camera(&Handle::new(camera));
    view.set_lights(&other.lights());
    view.set_clip_planes(&other.clip_planes());
    Ok(())
}

/// The base body of `Resized`: a subview resizes its neutral window; `resized(view)`.
pub fn resized<V: Graphic3dCView + ?Sized>(view: &V) {
    if view.is_subview() {
        let window = view.window().cast::<AspectNeutralWindow>();
        view.subview_resized(&window);
    }
}

/// The native `InitXR`.
pub(crate) fn init_xr<V: Graphic3dCView + ?Sized>(view: &V) -> bool {
    let session = {
        let mut fields = view.cview_base().fields.write();
        if fields.xr_session.is_null() {
            let session: Handle<dyn AspectXRSession> =
                Handle::new(AspectOpenVRSession::default()).cast();
            if let Some(session) = session.get() {
                session.set_unit_factor(fields.unit_factor);
            }
            fields.xr_session = session;
        }
        fields.xr_session.clone()
    };
    let Some(session) = session.get() else {
        return false;
    };
    if !session.is_open() {
        session.open();
        let mut fields = view.cview_base().fields.write();
        if fields.back_xr_camera.is_null() {
            let copy = fields
                .camera
                .get()
                .map(Graphic3dCamera::from_camera)
                .unwrap_or_default();
            fields.back_xr_camera = Handle::new(copy);
        }
    }
    session.is_open()
}

/// The native `ReleaseXR`: restores the projection of the camera kept by `InitXR`.
pub(crate) fn release_xr<V: Graphic3dCView + ?Sized>(view: &V) {
    let (session, camera, back) = {
        let fields = view.cview_base().fields.read();
        (
            fields.xr_session.clone(),
            fields.camera.clone(),
            fields.back_xr_camera.clone(),
        )
    };
    let Some(session) = session.get() else { return };
    if let (true, Some(camera), Some(back)) = (session.is_open(), camera.get(), back.get()) {
        camera.set_fov_2d(back.fov_2d());
        camera.set_fov_y(back.fov_y());
        camera.set_aspect(back.aspect());
        camera.set_iod(back.iod_type(), back.iod());
        camera.set_z_focus(back.z_focus_type(), back.z_focus());
        camera.reset_custom_projection();
        view.cview_base().fields.write().back_xr_camera = Handle::default();
    }
    session.close();
}

/// The native `ProcessXRInput`.
pub(crate) fn process_xr_input<V: Graphic3dCView + ?Sized>(view: &V) -> Result<(), Failure> {
    let (stereo_mode, camera) = {
        let fields = view.cview_base().fields.read();
        (
            fields.render_params.stereo.stereo_mode,
            fields.camera.clone(),
        )
    };
    let stereo = camera
        .get()
        .is_some_and(|c| c.projection_type() == Graphic3dCameraProjection::Stereo);
    if stereo_mode == Graphic3dStereoMode::OpenVR && stereo {
        view.init_xr();
    } else {
        view.release_xr();
    }
    if !view.is_active_xr() {
        let mut fields = view.cview_base().fields.write();
        fields.base_xr_camera = Handle::default();
        fields.posed_xr_camera = Handle::default();
        return Ok(());
    }
    let session = view.xr_session();
    let (Some(session), Some(camera_ref)) = (session.get(), camera.get()) else {
        return Ok(());
    };
    session.process_events();
    view.invalidate();
    apply_head_mounted_display(view, session, camera_ref);
    let mut fields = view.cview_base().fields.write();
    fields.base_xr_camera = camera.clone();
    if fields.posed_xr_camera.is_null() {
        fields.posed_xr_camera = Handle::new(Graphic3dCamera::default());
    }
    drop(fields);
    view.synchronize_xr_base_to_posed_camera()
}

/// The camera field of view, aspect, IOD, focus and stereo projection of the session.
fn apply_head_mounted_display<V: Graphic3dCView + ?Sized>(
    view: &V,
    session: &dyn AspectXRSession,
    camera: &Graphic3dCamera,
) {
    let (hmd_fov_2d, unit_factor) = {
        let fields = view.cview_base().fields.read();
        (fields.render_params.stereo.hmd_fov_2d, fields.unit_factor)
    };
    camera.set_fov_2d(f64::from(hmd_fov_2d));
    camera.set_aspect(session.aspect());
    camera.set_fov_y(session.field_of_view());
    camera.set_iod(Graphic3dCameraIodType::Absolute, session.iod());
    camera.set_z_focus(Graphic3dCameraFocusType::Absolute, 1.0 * unit_factor);
    if session.has_projection_frustums() {
        camera.set_custom_stereo_frustums(
            &session.projection_frustum(AspectEye::Left),
            &session.projection_frustum(AspectEye::Right),
        );
        return;
    }
    let pose_l = session.head_to_eye_transform(AspectEye::Left);
    let pose_r = session.head_to_eye_transform(AspectEye::Right);
    let proj_l = session.projection_matrix(AspectEye::Left, camera.z_near(), camera.z_far());
    let proj_r = session.projection_matrix(AspectEye::Right, camera.z_near(), camera.z_far());
    camera.set_custom_stereo_projection(&proj_l, &pose_l, &proj_r, &pose_r);
}

/// The base body of `DiagnosticInformation`: the vendor, device, tracker, display and serial
/// strings of the XR session for the device flag; `diagnostic_information(view, &mut dict, flags)`.
pub fn diagnostic_information<V: Graphic3dCView + ?Sized>(
    view: &V,
    dict: &mut Graphic3dFrameStatsDict,
    flags: Graphic3dDiagnosticInfo,
) {
    let session = view.cview_base().fields.read().xr_session.clone();
    let Some(session) = session
        .get()
        .filter(|_| (flags.0 & Graphic3dDiagnosticInfo::DEVICE.0) != 0)
    else {
        return;
    };
    let text = |info| String::from_utf8_lossy(session.get_string(info).as_bytes()).into_owned();
    let viewport = session.recommended_viewport();
    let display = format!(
        "{}x{}@{} [FOVy: {}]",
        viewport.x,
        viewport.y,
        int_from_f64(f64::from(session.display_frequency()).round()),
        int_from_f64(session.field_of_view().round())
    );
    add_info_text(dict, "VRvendor", &text(AspectXRSessionInfoString::Vendor));
    add_info_text(dict, "VRdevice", &text(AspectXRSessionInfoString::Device));
    add_info_text(dict, "VRtracker", &text(AspectXRSessionInfoString::Tracker));
    add_info_text(dict, "VRdisplay", &display);
    add_info_text(
        dict,
        "VRserial",
        &text(AspectXRSessionInfoString::SerialNumber),
    );
}
