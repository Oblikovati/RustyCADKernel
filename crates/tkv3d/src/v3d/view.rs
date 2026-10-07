//! `V3d_View`: a view of a viewer, over a driver view (`Graphic3d_CView`) and its camera.
//!
//! The driver view is created once and never changes; the camera is the driver view's
//! (`SetCamera` refuses a null camera, so it is never null). The mutable fields sit in one
//! `SharedState`; a member reads what it needs, releases the lock, then calls the camera, the
//! driver view or the viewer. The members are split by topic: view_camera.rs (rotations and
//! orientation), view_mapping.rs (size, zoom, fit and panning), view_convert.rs (pixel and
//! view coordinates, queries), view_grid.rs (grids and trihedron), view_render.rs (backgrounds,
//! lights, clip planes, rendering parameters) and view_dump.rs (image dump and `DumpJson`).
use super::{V3dTrihedron, V3dTypeOfView, V3dViewer, v3d_bad_value};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::exceptions::{null_object, program_error};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Vector3};
use rustycad_tkservice::aspect::{
    AspectGrid, AspectNeutralWindow, AspectRenderingContext, AspectTypeOfTriedronPosition,
    AspectWindow, AspectWindowMembers as _,
};
use rustycad_tkservice::graphic3d::{
    Graphic3dCLight, Graphic3dCView, Graphic3dCViewMembers as _, Graphic3dCViewSubviews as _,
    Graphic3dCamera, Graphic3dCameraFocusType, Graphic3dCameraIodType, Graphic3dCameraProjection,
    Graphic3dGroup, Graphic3dStructureSource, Graphic3dVertex,
};

/// The mutable fields of a view.
#[derive(Clone)]
pub(crate) struct V3dViewState {
    pub(crate) old_mouse_x: f64,
    pub(crate) old_mouse_y: f64,
    pub(crate) cam_start_op_up: Direction3,
    pub(crate) cam_start_op_dir: Direction3,
    pub(crate) cam_start_op_eye: Point3,
    pub(crate) cam_start_op_center: Point3,
    pub(crate) immediate_update: bool,
    pub(crate) is_invalidated_immediate: bool,
    pub(crate) viewer: WeakHandle<V3dViewer>,
    pub(crate) subviews: Vec<Handle<V3dView>>,
    pub(crate) parent_view: WeakHandle<V3dView>,
    pub(crate) active_lights: Vec<Handle<Graphic3dCLight>>,
    pub(crate) active_lights_cursor: usize,
    pub(crate) default_view_axis: Direction3,
    pub(crate) default_view_point: Point3,
    pub(crate) window: Handle<dyn AspectWindow>,
    pub(crate) sx: i32,
    pub(crate) sy: i32,
    pub(crate) rx: f64,
    pub(crate) ry: f64,
    pub(crate) rotate_gravity: Point3,
    pub(crate) computed_mode: bool,
    pub(crate) switch_set_front: bool,
    pub(crate) z_rotation: bool,
    pub(crate) zoom_at_point_x: i32,
    pub(crate) zoom_at_point_y: i32,
    pub(crate) trihedron: Handle<V3dTrihedron>,
    pub(crate) grid: Handle<dyn AspectGrid>,
    pub(crate) plane: Axis3,
    pub(crate) trsf: [[f64; 4]; 4],
    pub(crate) shader_grid_active: bool,
    pub(crate) grid_echo_structure: Handle<dyn Graphic3dStructureSource>,
    pub(crate) grid_echo_group: Handle<dyn Graphic3dGroup>,
    pub(crate) x_screen_axis: Vector3,
    pub(crate) y_screen_axis: Vector3,
    pub(crate) z_screen_axis: Vector3,
    pub(crate) view_axis: Direction3,
    pub(crate) gravity_reference_point: Graphic3dVertex,
    pub(crate) auto_z_fit_is_on: bool,
    pub(crate) auto_z_fit_scale_factor: f64,
}

impl V3dViewState {
    /// The fields before the constructor body. The members that the native constructors
    /// leave uninitialized (`myOldMouseX/Y`, `sx`, `sy`, `rx`, `ry`, `MyZoomAtPointX/Y`,
    /// `MyTrsf`) start at zero.
    fn initial(viewer: &Handle<V3dViewer>) -> Self {
        Self {
            old_mouse_x: 0.0,
            old_mouse_y: 0.0,
            cam_start_op_up: Direction3::default(),
            cam_start_op_dir: Direction3::default(),
            cam_start_op_eye: Point3::default(),
            cam_start_op_center: Point3::default(),
            immediate_update: false,
            is_invalidated_immediate: true,
            viewer: viewer.downgrade(),
            subviews: Vec::new(),
            parent_view: WeakHandle::default(),
            active_lights: Vec::new(),
            active_lights_cursor: 0,
            default_view_axis: Direction3::default(),
            default_view_point: Point3::default(),
            window: Handle::default(),
            sx: 0,
            sy: 0,
            rx: 0.0,
            ry: 0.0,
            rotate_gravity: Point3::default(),
            computed_mode: false,
            switch_set_front: false,
            z_rotation: false,
            zoom_at_point_x: 0,
            zoom_at_point_y: 0,
            trihedron: Handle::default(),
            grid: Handle::default(),
            plane: Axis3::default(),
            trsf: [[0.0; 4]; 4],
            shader_grid_active: false,
            grid_echo_structure: Handle::default(),
            grid_echo_group: Handle::default(),
            x_screen_axis: Vector3::default(),
            y_screen_axis: Vector3::default(),
            z_screen_axis: Vector3::default(),
            view_axis: Direction3::default(),
            gravity_reference_point: Graphic3dVertex::default(),
            auto_z_fit_is_on: true,
            auto_z_fit_scale_factor: 1.0,
        }
    }
}

/// A view of a viewer.
///
/// Example: `let view = viewer.create_view()?; view.set_window(&window, 0)?; view.fit_all(0.01, true)?;`.
pub struct V3dView {
    transient: StandardTransient,
    pub(crate) view: Handle<dyn Graphic3dCView>,
    default_camera: Handle<Graphic3dCamera>,
    this: SharedState<WeakHandle<V3dView>>,
    pub(crate) state: SharedState<V3dViewState>,
}
rustycad_tkernel::impl_transient!(V3dView, "V3d_View", StandardTransient, transient);

/// The driver view of a viewer's driver; a missing viewer or driver is refused.
fn create_driver_view(viewer: &Handle<V3dViewer>) -> Result<Handle<dyn Graphic3dCView>, Failure> {
    let Some(viewer_ref) = viewer.get() else {
        return Err(null_object("V3d_View: null viewer; expected a viewer"));
    };
    let driver = viewer_ref.driver();
    let Some(driver) = driver.get() else {
        return Err(null_object(
            "V3d_View: the viewer has no graphic driver; expected a driver",
        ));
    };
    let view = driver.create_view(&viewer_ref.structure_manager())?;
    if view.is_null() {
        return Err(null_object(
            "V3d_View: the driver created no view; expected a driver view",
        ));
    }
    Ok(view)
}

impl V3dView {
    fn allocate(
        viewer: &Handle<V3dViewer>,
        view: Handle<dyn Graphic3dCView>,
        default_camera: Graphic3dCamera,
    ) -> Handle<Self> {
        let handle = Handle::new(Self {
            transient: StandardTransient::default(),
            view,
            default_camera: Handle::new(default_camera),
            this: SharedState::new(WeakHandle::default()),
            state: SharedState::new(V3dViewState::initial(viewer)),
        });
        *handle.this.write() = handle.downgrade();
        handle
    }
    /// `V3d_View(viewer, type)`: a view with the viewer's background, rendering parameters,
    /// projection, size and visualization, registered in the viewer.
    /// Example: `V3dView::new(&viewer, V3dTypeOfView::Orthographic)?`.
    pub fn new(viewer: &Handle<V3dViewer>, kind: V3dTypeOfView) -> Result<Handle<Self>, Failure> {
        let cview = create_driver_view(viewer)?;
        let viewer_ref = viewer
            .get()
            .ok_or_else(|| null_object("V3d_View: null viewer"))?;
        cview.set_background(&viewer_ref.get_background_color());
        cview.set_gradient_background(&viewer_ref.get_gradient_background());
        let params = viewer_ref.default_rendering_params();
        cview.change_rendering_params(&mut |p| *p = params.clone());
        let camera = Graphic3dCamera::default();
        camera.set_fov_y(45.0);
        camera.set_iod(Graphic3dCameraIodType::Relative, 0.05);
        camera.set_z_focus(Graphic3dCameraFocusType::Relative, 1.0);
        let projection = if kind == V3dTypeOfView::Orthographic {
            Graphic3dCameraProjection::Orthographic
        } else {
            Graphic3dCameraProjection::Perspective
        };
        camera.set_projection_type(projection);
        let view = Self::allocate(viewer, cview, Graphic3dCamera::default());
        view.set_auto_z_fit_mode(true, 1.0)?;
        view.set_back_facing_model(
            rustycad_tkservice::graphic3d::Graphic3dTypeOfBackfacingModel::Auto,
        );
        view.set_camera(&Handle::new(camera))?;
        view.set_axis(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)?;
        view.set_visualization(viewer_ref.default_visualization());
        view.set_twist(0.0)?;
        view.set_at(0.0, 0.0, 0.0)?;
        view.set_proj_orientation(viewer_ref.default_view_proj(), false)?;
        view.set_size(viewer_ref.default_view_size())?;
        let z_size = viewer_ref.default_view_size();
        view.set_z_size(2.0 * z_size)?;
        view.set_depth(viewer_ref.default_view_size() / 2.0)?;
        view.set_view_mapping_default();
        view.set_view_orientation_default();
        viewer_ref.add_view(&view);
        view.init();
        view.state.write().immediate_update = true;
        Ok(view)
    }
    /// `V3d_View(viewer, view)`: a copy of another view's settings and default camera.
    /// Example: `V3dView::new_copy(&viewer, &other)?`.
    pub fn new_copy(
        viewer: &Handle<V3dViewer>,
        other: &Handle<V3dView>,
    ) -> Result<Handle<Self>, Failure> {
        let Some(other_ref) = other.get() else {
            return Err(null_object("V3d_View: null view to copy; expected a view"));
        };
        let cview = create_driver_view(viewer)?;
        cview.copy_settings(&other_ref.view())?;
        let default_camera = Graphic3dCamera::from_camera(&other_ref.default_camera());
        let view = Self::allocate(viewer, cview, default_camera);
        {
            let source = other_ref.state.read().clone();
            let mut state = view.state.write();
            (state.default_view_point, state.default_view_axis) =
                (source.default_view_point, source.default_view_axis);
        }
        view.set_auto_z_fit_mode(
            other_ref.auto_z_fit_mode(),
            other_ref.auto_z_fit_scale_factor(),
        )?;
        if let Some(viewer_ref) = viewer.get() {
            viewer_ref.add_view(&view);
        }
        view.init();
        view.state.write().immediate_update = true;
        Ok(view)
    }
    /// The view itself as a handle (`this`).
    pub fn this(&self) -> Handle<V3dView> {
        self.this.read().upgrade()
    }
    /// `View`: the driver view; `view.view()`.
    pub fn view(&self) -> Handle<dyn Graphic3dCView> {
        self.view.clone()
    }
    /// `Viewer`: the viewer, null after `Remove`; `view.viewer()`.
    pub fn viewer(&self) -> Handle<V3dViewer> {
        self.state.read().viewer.upgrade()
    }
    /// `Window`; `view.window()`.
    pub fn window(&self) -> Handle<dyn AspectWindow> {
        self.state.read().window.clone()
    }
    /// `DefaultCamera`; `view.default_camera()`.
    pub fn default_camera(&self) -> Handle<Graphic3dCamera> {
        self.default_camera.clone()
    }
    /// `Camera`: the camera of the driver view; `view.camera()`.
    pub fn camera(&self) -> Handle<Graphic3dCamera> {
        self.view.camera()
    }
    /// `SetCamera`; a null camera is refused with `Standard_NullObject` (natively it breaks
    /// every later camera member). Example: `view.set_camera(&camera)?`.
    pub fn set_camera(&self, camera: &Handle<Graphic3dCamera>) -> Result<(), Failure> {
        if camera.is_null() {
            return Err(null_object(
                "V3d_View::SetCamera: null camera; expected a camera",
            ));
        }
        self.view.set_camera(camera);
        self.immediate_update()
    }
    /// `Init` (private): the computed mode of the viewer.
    fn init(&self) {
        let Some(viewer) = self
            .viewer()
            .get()
            .map(|v| (v.computed_mode(), v.default_computed_mode()))
        else {
            return;
        };
        self.state.write().computed_mode = viewer.0;
        if !viewer.0 || !viewer.1 {
            self.set_computed_mode(false);
        }
    }
    /// `SetWindow(window, context)`: attaches a window, activates the view and redraws.
    /// A subview raises `Standard_ProgramError`. Example: `view.set_window(&window, 0)?`.
    pub fn set_window(
        &self,
        window: &Handle<dyn AspectWindow>,
        context: AspectRenderingContext,
    ) -> Result<(), Failure> {
        if self.view.is_removed() {
            return Ok(());
        }
        if self.is_subview() {
            return Err(program_error(
                "V3d_View::SetWindow() called twice; the view is a subview",
            ));
        }
        self.state.write().window = window.clone();
        self.view.set_window(&Handle::default(), window, context);
        if let Some(viewer) = self.viewer().get() {
            viewer.set_view_on_one(&self.this())?;
        }
        self.set_ratio();
        if self.state.read().immediate_update {
            self.redraw();
        }
        Ok(())
    }
    /// `SetWindow(parentView, size, corner, offset, margins)`: makes this view a subview of
    /// the parent (or of the parent's parent) in a virtual 4x4 window.
    /// Example: `view.set_subview_window(&parent, Vec2::new(0.5, 0.5), corner, Vec2::new(0.0, 0.0), Vec2::new(0, 0))?`.
    pub fn set_subview_window(
        &self,
        parent_view: &Handle<V3dView>,
        size: Vec2<f64>,
        corner: AspectTypeOfTriedronPosition,
        offset: Vec2<f64>,
        margins: Vec2<i32>,
    ) -> Result<(), Failure> {
        if self.view.is_removed() {
            return Ok(());
        }
        let Some(parent_ref) = parent_view.get() else {
            return Err(null_object(
                "V3d_View::SetWindow: null parent view; expected a view",
            ));
        };
        let parent = if parent_ref.is_subview() {
            parent_ref.parent_view()
        } else {
            parent_view.clone()
        };
        let current = self.state.read().parent_view.upgrade();
        if parent != current {
            if !current.is_null() {
                return Err(program_error(
                    "V3d_View::SetWindow() called twice; the subview has another parent",
                ));
            }
            self.state.write().parent_view = parent.downgrade();
            if let Some(parent_ref) = parent.get() {
                parent_ref.add_subview(&self.this());
            }
        }
        let window = AspectNeutralWindow::default();
        window.set_virtual(true);
        window.set_size(4, 4);
        self.view.set_subview_corner(corner);
        self.view.set_subview_size(size);
        self.view.set_subview_offset(offset);
        self.view.set_subview_margins(margins);
        let window: Handle<dyn AspectWindow> = Handle::new(window).cast();
        self.state.write().window = window.clone();
        let parent_cview = parent.get().map(V3dView::view).unwrap_or_default();
        self.view.set_window(&parent_cview, &window, 0);
        if let Some(viewer) = self.viewer().get() {
            viewer.set_view_on_one(&self.this())?;
        }
        self.set_ratio();
        Ok(())
    }
    /// `SetMagnify`: an undefined view takes a window and fits the rectangle of another view.
    /// Example: `view.set_magnify(&window, &previous, 0, 0, 100, 100)?`.
    pub fn set_magnify(
        &self,
        window: &Handle<dyn AspectWindow>,
        previous_view: &Handle<V3dView>,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> Result<(), Failure> {
        if self.view.is_removed() || self.view.is_defined() {
            return Ok(());
        }
        let Some(previous) = previous_view.get() else {
            return Ok(());
        };
        let (u1, v1) = previous.convert_to_view(x1, y1)?;
        let (u2, v2) = previous.convert_to_view(x2, y2)?;
        self.view.set_window(&Handle::default(), window, 0);
        self.fit_all_rect(u1, v1, u2, v2)?;
        if let Some(viewer) = self.viewer().get() {
            viewer.set_view_on_one(&self.this())?;
        }
        self.state.write().window = window.clone();
        self.set_ratio();
        self.redraw();
        self.set_view_mapping_default();
        Ok(())
    }
    /// `Remove`: erases the grid, the shader grid and the trihedron, removes the subviews,
    /// leaves the viewer and removes the driver view. Example: `view.remove()`.
    pub fn remove(&self) {
        let state = self.state.read().clone();
        if let Some(grid) = state.grid.get() {
            grid.erase();
        }
        if state.shader_grid_active {
            self.view.grid_erase();
            self.state.write().shader_grid_active = false;
        }
        if let Some(trihedron) = state.trihedron.get() {
            trihedron.erase();
        }
        if let Some(parent) = state.parent_view.upgrade().get() {
            parent.remove_subview(self);
            self.state.write().parent_view = WeakHandle::default();
        }
        let subviews = std::mem::take(&mut self.state.write().subviews);
        subviews
            .iter()
            .filter_map(Handle::get)
            .for_each(V3dView::remove);
        if let Some(viewer) = state.viewer.upgrade().get() {
            viewer.del_view(self);
            self.state.write().viewer = WeakHandle::default();
        }
        self.view.remove();
        self.state.write().window = Handle::default();
    }
    /// `SetImmediateUpdate`: returns the previous mode; `view.set_immediate_update(false)`.
    pub fn set_immediate_update(&self, immediate_update: bool) -> bool {
        std::mem::replace(&mut self.state.write().immediate_update, immediate_update)
    }
    /// `ImmediateUpdate` (protected): `Update` when the immediate mode is on.
    pub(crate) fn immediate_update(&self) -> Result<(), Failure> {
        if self.state.read().immediate_update {
            self.update()?;
        }
        Ok(())
    }
}

impl Drop for V3dView {
    /// The destructor detaches the view from its parent, detaches the subviews and removes
    /// the driver view.
    fn drop(&mut self) {
        let state = self.state.read().clone();
        if let Some(parent) = state.parent_view.upgrade().get() {
            parent.remove_subview(self);
        }
        for subview in state.subviews.iter().filter_map(Handle::get) {
            {
                let mut sub_state = subview.state.write();
                (sub_state.parent_view, sub_state.window) =
                    (WeakHandle::default(), Handle::default());
            }
            subview.view.remove();
            if let Some(viewer) = subview.viewer().get() {
                viewer.set_view_off_one(&subview.this());
            }
        }
        if !self.view.is_removed() {
            self.view.remove();
        }
    }
}

/// `V3d_BadValue_Raise_if` with the offending value.
pub(crate) fn bad_value_if(condition: bool, message: &str) -> Result<(), Failure> {
    if condition {
        return Err(v3d_bad_value(message));
    }
    Ok(())
}
