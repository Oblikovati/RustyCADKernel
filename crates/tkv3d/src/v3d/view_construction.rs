//! Native base construction is separated from binding the final concrete or derived object.
use super::{V3dTypeOfView, V3dView, V3dViewSource, V3dViewer};
use rustycad_tkernel::standard::exceptions::null_object;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkservice::graphic3d::{
    Graphic3dCView, Graphic3dCViewMembers as _, Graphic3dCamera, Graphic3dCameraFocusType,
    Graphic3dCameraIodType, Graphic3dCameraProjection, Graphic3dTypeOfBackfacingModel,
};

impl V3dView {
    /// Construct and register the actual concrete view; `V3dView::new(&viewer,V3dTypeOfView::Orthographic)?`.
    // Native construction returns a handle transport capable of retaining valid subclasses.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        viewer: &Handle<V3dViewer>,
        kind: V3dTypeOfView,
    ) -> Result<Handle<dyn V3dViewSource>, Failure> {
        let base = Self::new_embedded(viewer, kind)?;
        let view = Handle::new(base).cast();
        Self::bind_source(&view)?;
        Ok(view)
    }
    /// Initialize fields owned by a genuine subclass before registration; `let base=V3dView::new_embedded(&viewer,kind)?`.
    pub fn new_embedded(viewer: &Handle<V3dViewer>, kind: V3dTypeOfView) -> Result<Self, Failure> {
        let cview = super::view::create_driver_view(viewer)?;
        let owner = viewer
            .get()
            .ok_or_else(|| null_object("V3d_View: null viewer; expected a viewer"))?;
        inherited_driver_defaults(&cview, owner);
        let base = allocate_owned_view(viewer, cview, Graphic3dCamera::default());
        inherited_camera_defaults(&base, kind)?;
        inherited_orientation_defaults(&base, owner)?;
        inherited_mapping_defaults(&base, owner)?;
        Ok(base)
    }
    /// Bind and register the final object's identity without a copied core view; `V3dView::bind_source(&derived.cast())?`.
    pub fn bind_source(view: &Handle<dyn V3dViewSource>) -> Result<(), Failure> {
        let source = view
            .get()
            .ok_or_else(|| null_object("V3d_View: null source; expected initialized owned base"))?;
        let base = source.view_base();
        *base.this.write() = view.downgrade();
        if let Some(viewer) = base.viewer().get() {
            viewer.add_view(view);
        }
        base.init();
        base.state.write().immediate_update = true;
        Ok(())
    }
    /// Copy settings into the actual newly owned view; `V3dView::new_copy(&viewer,&other)?`.
    pub fn new_copy(
        viewer: &Handle<V3dViewer>,
        other: &Handle<dyn V3dViewSource>,
    ) -> Result<Handle<dyn V3dViewSource>, Failure> {
        let source = other
            .get()
            .ok_or_else(|| null_object("V3d_View: null view to copy; expected a view"))?;
        let cview = super::view::create_driver_view(viewer)?;
        cview.copy_settings(&source.view())?;
        let base = allocate_owned_view(
            viewer,
            cview,
            Graphic3dCamera::from_camera(&source.default_camera()),
        );
        copy_inherited_view_axis(&base, source.view_base());
        base.set_auto_z_fit_mode(source.auto_z_fit_mode(), source.auto_z_fit_scale_factor())?;
        let view = Handle::new(base).cast();
        Self::bind_source(&view)?;
        Ok(view)
    }
}
fn allocate_owned_view(
    viewer: &Handle<V3dViewer>,
    view: Handle<dyn Graphic3dCView>,
    default_camera: Graphic3dCamera,
) -> V3dView {
    V3dView {
        transient: StandardTransient::default(),
        driver_view_slot: rustycad_tkernel::standard::IntrusiveReferenceSlot::from_owned(view),
        default_camera: Handle::new(default_camera),
        this: SharedState::new(WeakHandle::default()),
        state: SharedState::new(super::view::V3dViewState::initial(viewer)),
    }
}
fn inherited_driver_defaults(cview: &Handle<dyn Graphic3dCView>, viewer: &V3dViewer) {
    cview.set_background(&viewer.get_background_color());
    cview.set_gradient_background(&viewer.get_gradient_background());
    let params = viewer.default_rendering_params();
    cview.change_rendering_params(&mut |p| *p = params.clone());
}
fn inherited_camera_defaults(view: &V3dView, kind: V3dTypeOfView) -> Result<(), Failure> {
    let camera = Graphic3dCamera::default();
    camera.set_fov_y(45.0);
    camera.set_iod(Graphic3dCameraIodType::Relative, 0.05);
    camera.set_z_focus(Graphic3dCameraFocusType::Relative, 1.0);
    let projection = match kind {
        V3dTypeOfView::Orthographic => Graphic3dCameraProjection::Orthographic,
        _ => Graphic3dCameraProjection::Perspective,
    };
    camera.set_projection_type(projection);
    view.set_auto_z_fit_mode(true, 1.0)?;
    view.set_back_facing_model(Graphic3dTypeOfBackfacingModel::Auto);
    view.set_camera(&Handle::new(camera))
}
fn inherited_orientation_defaults(view: &V3dView, viewer: &V3dViewer) -> Result<(), Failure> {
    view.set_axis(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)?;
    view.set_visualization(viewer.default_visualization());
    view.set_twist(0.0)?;
    view.set_at(0.0, 0.0, 0.0)?;
    view.set_proj_orientation(viewer.default_view_proj(), false)
}
fn inherited_mapping_defaults(view: &V3dView, viewer: &V3dViewer) -> Result<(), Failure> {
    let size = viewer.default_view_size();
    view.set_size(size)?;
    view.set_z_size(2.0 * size)?;
    view.set_depth(size / 2.0)?;
    view.set_view_mapping_default();
    view.set_view_orientation_default();
    Ok(())
}
fn copy_inherited_view_axis(view: &V3dView, source: &V3dView) {
    let axis = source.state.read();
    let mut target = view.state.write();
    (target.default_view_point, target.default_view_axis) =
        (axis.default_view_point, axis.default_view_axis);
}
