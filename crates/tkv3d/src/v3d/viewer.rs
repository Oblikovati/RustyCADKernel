//! `V3d_Viewer`: the viewer owns the structure manager of a graphic driver, its views, its
//! lights, its Z layers, the privileged plane and the grids.
//!
//! The views hold the viewer through a weak handle (a raw pointer natively) and the viewer
//! holds them through handles, as natively. The members are split by topic: viewer_lights.rs,
//! viewer_grid.rs (privileged plane, grids and grid echo), viewer_iter.rs (the deprecated
//! iteration members) and viewer_dump.rs.
use super::{
    V3dCircularGrid, V3dRectangularGrid, V3dTypeOfOrientation, V3dTypeOfView,
    V3dTypeOfVisualization, V3dView, v3d_bad_value,
};
use crate::v3d::V3dViewSource;
use rustycad_tkernel::ncollection::AllocatorList;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::HandleReference as _;
use rustycad_tkernel::standard::short_real::short_real_last;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, StandardDirection3};
use rustycad_tkservice::aspect::{
    AspectBackground, AspectGenId, AspectGradientBackground, AspectGradientFillMethod,
    AspectGridType,
};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_TOP, Graphic3dAspectMarker3d, Graphic3dCLight, Graphic3dGraphicDriver,
    Graphic3dGroup, Graphic3dRenderingParams, Graphic3dStructureManager, Graphic3dStructureSource,
    Graphic3dTypeOfShadingModel, Graphic3dVertex, Graphic3dZLayerId, Graphic3dZLayerSettings,
};
use std::collections::BTreeSet;

/// The mutable fields of a viewer.
#[derive(Clone)]
pub(crate) struct V3dViewerState {
    pub(crate) layer_ids: BTreeSet<i32>,
    pub(crate) z_layer_gen_id: AspectGenId,
    pub(crate) defined_views: AllocatorList<Handle<dyn V3dViewSource>>,
    pub(crate) active_views: AllocatorList<Handle<dyn V3dViewSource>>,
    pub(crate) defined_lights: AllocatorList<Handle<Graphic3dCLight>>,
    pub(crate) active_lights: AllocatorList<Handle<Graphic3dCLight>>,
    pub(crate) background: AspectBackground,
    pub(crate) gradient_background: AspectGradientBackground,
    pub(crate) view_size: f64,
    pub(crate) view_proj: V3dTypeOfOrientation,
    pub(crate) visualization: V3dTypeOfVisualization,
    pub(crate) default_type_of_view: V3dTypeOfView,
    pub(crate) default_rendering_params: Graphic3dRenderingParams,
    pub(crate) light_cursors: [rustycad_tkernel::ncollection::ListCursor; 2],
    pub(crate) view_cursors: [rustycad_tkernel::ncollection::ListCursor; 2],
    pub(crate) computed_mode: bool,
    pub(crate) default_computed_mode: bool,
    pub(crate) privileged_plane: Axis3,
    pub(crate) plane_structure: Handle<dyn Graphic3dStructureSource>,
    pub(crate) display_plane: bool,
    pub(crate) display_plane_length: f64,
    pub(crate) r_grid: Handle<V3dRectangularGrid>,
    pub(crate) c_grid: Handle<V3dCircularGrid>,
    pub(crate) grid_type: AspectGridType,
    pub(crate) grid_echo: bool,
    pub(crate) grid_echo_structure: Handle<dyn Graphic3dStructureSource>,
    pub(crate) grid_echo_group: Handle<dyn Graphic3dGroup>,
    pub(crate) grid_echo_aspect: Handle<Graphic3dAspectMarker3d>,
    pub(crate) grid_echo_last_vert: Graphic3dVertex,
}

/// The viewer of a graphic driver.
///
/// Example: `let viewer = V3dViewer::new(&driver)?; let view = viewer.create_view()?;`.
pub struct V3dViewer {
    transient: StandardTransient,
    driver: Handle<dyn Graphic3dGraphicDriver>,
    structure_manager: Handle<Graphic3dStructureManager>,
    this: SharedState<WeakHandle<V3dViewer>>,
    pub(crate) state: SharedState<V3dViewerState>,
}
rustycad_tkernel::impl_transient!(V3dViewer, "V3d_Viewer", StandardTransient, transient);

impl V3dViewer {
    /// A viewer of a driver: gray 30 background, view size 1000, `XposYnegZpos` projection,
    /// orthographic Z-buffer views, rectangular grid with echo, privileged plane XOY.
    /// Example: `V3dViewer::new(&NullGraphicDriver::new_handle())?`.
    pub fn new(driver: &Handle<dyn Graphic3dGraphicDriver>) -> Result<Handle<Self>, Failure> {
        let manager = Handle::new(Graphic3dStructureManager::new(driver)?);
        // gp_Ax3(P, gp_Dir(D::Z), gp_Dir(D::X)) takes the general constructor: its cross
        // products give the Y direction (-0, 1, 0).
        let plane = Axis3::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::from_standard(StandardDirection3::Z),
            Direction3::from_standard(StandardDirection3::X),
        )?;
        let last = short_real_last();
        let state = V3dViewerState {
            layer_ids: BTreeSet::new(),
            z_layer_gen_id: AspectGenId::new(1, i32::MAX)?,
            defined_views: AllocatorList::default(),
            active_views: AllocatorList::default(),
            defined_lights: AllocatorList::default(),
            active_lights: AllocatorList::default(),
            background: AspectBackground::new(Color::named(NamedColor::GRAY30)),
            gradient_background: AspectGradientBackground::default(),
            view_size: 1000.0,
            view_proj: V3dTypeOfOrientation::XposYnegZpos,
            visualization: V3dTypeOfVisualization::Zbuffer,
            default_type_of_view: V3dTypeOfView::Orthographic,
            default_rendering_params: Graphic3dRenderingParams::new()?,
            light_cursors: Default::default(),
            view_cursors: [rustycad_tkernel::ncollection::ListCursor::default(); 2],
            computed_mode: true,
            default_computed_mode: false,
            privileged_plane: plane,
            plane_structure: Handle::default(),
            display_plane: false,
            display_plane_length: 1000.0,
            r_grid: Handle::default(),
            c_grid: Handle::default(),
            grid_type: AspectGridType::Rectangular,
            grid_echo: true,
            grid_echo_structure: Handle::default(),
            grid_echo_group: Handle::default(),
            grid_echo_aspect: Handle::default(),
            grid_echo_last_vert: Graphic3dVertex::new(last, last, last),
        };
        let viewer = Handle::new(Self {
            transient: StandardTransient::default(),
            driver: driver.clone(),
            structure_manager: manager,
            this: SharedState::new(WeakHandle::default()),
            state: SharedState::new(state),
        });
        *viewer.this.write() = viewer.downgrade();
        Ok(viewer)
    }
    /// The viewer itself as a handle (`this`); null only while the viewer is dropped.
    pub fn this(&self) -> Handle<V3dViewer> {
        self.this.read().upgrade()
    }
    /// `IfMoreViews`: whether another view can be defined; `viewer.if_more_views()`.
    pub fn if_more_views(&self) -> bool {
        let count = self.state.read().defined_views.len() as i32;
        self.structure_manager
            .get()
            .is_some_and(|m| count < m.max_num_of_views())
    }
    /// `CreateView`: a new view of the default type; `viewer.create_view()?`.
    pub fn create_view(&self) -> Result<Handle<dyn V3dViewSource>, Failure> {
        let kind = self.state.read().default_type_of_view;
        V3dView::new(&self.this(), kind)
    }
    /// `SetViewOn()`: activates every defined view; `viewer.set_view_on()?`.
    pub fn set_view_on(&self) -> Result<(), Failure> {
        let mut cursor = self.defined_view_iterator();
        while cursor.more() {
            self.set_view_on_one(&cursor.value_ref())?;
            cursor.next();
        }
        Ok(())
    }
    /// `SetViewOff()`: deactivates defined views through their native live nodes; `viewer.set_view_off()`.
    pub fn set_view_off(&self) {
        let mut cursor = self.defined_view_iterator();
        while cursor.more() {
            self.set_view_off_one(&cursor.value_ref());
            cursor.next();
        }
    }
    /// `Update`: the same as `Redraw`; `viewer.update()`.
    pub fn update(&self) {
        self.redraw();
    }
    /// `Redraw`: redraws the subviews first, then the other views; `viewer.redraw()`.
    pub fn redraw(&self) {
        self.visit_defined_view_nodes(|view| view.redraw());
    }
    /// `RedrawImmediate`: the immediate layers, subviews first; `viewer.redraw_immediate()`.
    pub fn redraw_immediate(&self) {
        self.visit_defined_view_nodes(|view| view.redraw_immediate());
    }
    /// `Invalidate`: invalidates every defined view; `viewer.invalidate()`.
    pub fn invalidate(&self) {
        let mut cursor = self.defined_view_iterator();
        while cursor.more() {
            cursor.value_ref().observe().invalidate();
            cursor.next();
        }
    }
    fn visit_defined_view_nodes(&self, visit: impl Fn(&dyn V3dViewSource)) {
        for pass_subviews in [true, false] {
            let mut cursor = self.defined_view_iterator();
            while cursor.more() {
                let reference = cursor.value_ref().observe();
                if let Some(view) = reference.get().filter(|v| v.is_subview() == pass_subviews) {
                    visit(view);
                }
                cursor.next();
            }
        }
    }
    /// `Remove`: removes every structure of the manager; `viewer.remove()`.
    pub fn remove(&self) {
        if let Some(manager) = self.structure_manager.get() {
            manager.remove();
        }
    }
    /// The graphic driver; `viewer.driver()`.
    pub fn driver(&self) -> Handle<dyn Graphic3dGraphicDriver> {
        self.driver.clone()
    }
    /// Borrow the actual immutable driver handle slot; `viewer.driver_ref().observe()`.
    pub fn driver_ref(&self) -> &Handle<dyn Graphic3dGraphicDriver> {
        &self.driver
    }
    /// The structure manager; `viewer.structure_manager()`.
    pub fn structure_manager(&self) -> Handle<Graphic3dStructureManager> {
        self.structure_manager.clone()
    }
    /// `Erase`: erases every structure; `viewer.erase()`.
    pub fn erase(&self) {
        if let Some(manager) = self.structure_manager.get() {
            manager.erase();
        }
    }
    /// `UnHighlight`: unhighlights every structure; `viewer.un_highlight()`.
    pub fn un_highlight(&self) {
        if let Some(manager) = self.structure_manager.get() {
            manager.unhighlight();
        }
    }
    /// `AddView` (private, called by the view constructor).
    pub(crate) fn add_view(&self, view: &Handle<dyn V3dViewSource>) {
        let mut state = self.state.write();
        if !state.defined_views.iter().any(|stored| stored == view) {
            state.defined_views.append(view.clone());
        }
    }
    /// `DelView` (private): forgets a view, first in the active list then the defined one.
    pub(crate) fn del_view(&self, view: *const ()) {
        let mut state = self.state.write();
        remove_view_from(&mut state.active_views, view);
        remove_view_from(&mut state.defined_views, view);
    }
}

/// Removes the first view at an address from a list (`NCollection_List::Remove`).
pub(super) fn remove_view_from(
    list: &mut AllocatorList<Handle<dyn V3dViewSource>>,
    view: *const (),
) {
    let mut cursor = list.cursor();
    while cursor.more() {
        if cursor
            .value(list)
            .is_some_and(|value| value.object_address() == view)
        {
            let _ = list.remove(&mut cursor);
            return;
        }
        cursor.next(list);
    }
}

/// The default values of the views and the Z layers.
impl V3dViewer {
    /// `DefaultRenderingParams`; `viewer.default_rendering_params()`.
    pub fn default_rendering_params(&self) -> Graphic3dRenderingParams {
        self.state.read().default_rendering_params.clone()
    }
    /// `SetDefaultRenderingParams`; `viewer.set_default_rendering_params(&params)`.
    pub fn set_default_rendering_params(&self, params: &Graphic3dRenderingParams) {
        self.state.write().default_rendering_params = params.clone();
    }
    /// `SetDefaultBackgroundColor`; `viewer.set_default_background_color(black)`.
    pub fn set_default_background_color(&self, color: Color) {
        self.state.write().background.set_color(color);
    }
    /// `GetGradientBackground`; `viewer.get_gradient_background()`.
    pub fn get_gradient_background(&self) -> AspectGradientBackground {
        self.state.read().gradient_background
    }
    /// `GetBackgroundColor` (private natively, the views' default background).
    pub fn get_background_color(&self) -> AspectBackground {
        self.state.read().background
    }
    /// `SetDefaultBgGradientColors`; `viewer.set_default_bg_gradient_colors(c1, c2, method)`.
    pub fn set_default_bg_gradient_colors(
        &self,
        color1: Color,
        color2: Color,
        method: AspectGradientFillMethod,
    ) {
        self.state
            .write()
            .gradient_background
            .set_colors(color1, color2, method);
    }
    /// `DefaultViewSize`; `viewer.default_view_size()`.
    pub fn default_view_size(&self) -> f64 {
        self.state.read().view_size
    }
    /// `SetDefaultViewSize`; a size <= 0 raises `V3d_BadValue`. `viewer.set_default_view_size(500.0)?`.
    pub fn set_default_view_size(&self, size: f64) -> Result<(), Failure> {
        if size <= 0.0 {
            return Err(v3d_bad_value(&format!(
                "V3d_Viewer::SetDefaultViewSize, bad size {size}; expected > 0"
            )));
        }
        self.state.write().view_size = size;
        Ok(())
    }
    /// `DefaultViewProj`; `viewer.default_view_proj()`.
    pub fn default_view_proj(&self) -> V3dTypeOfOrientation {
        self.state.read().view_proj
    }
    /// `SetDefaultViewProj`; `viewer.set_default_view_proj(V3dTypeOfOrientation::Zpos)`.
    pub fn set_default_view_proj(&self, orientation: V3dTypeOfOrientation) {
        self.state.write().view_proj = orientation;
    }
    /// `DefaultVisualization`; `viewer.default_visualization()`.
    pub fn default_visualization(&self) -> V3dTypeOfVisualization {
        self.state.read().visualization
    }
    /// `SetDefaultVisualization`; `viewer.set_default_visualization(V3dTypeOfVisualization::Wireframe)`.
    pub fn set_default_visualization(&self, kind: V3dTypeOfVisualization) {
        self.state.write().visualization = kind;
    }
    /// `DefaultShadingModel`; `viewer.default_shading_model()`.
    pub fn default_shading_model(&self) -> Graphic3dTypeOfShadingModel {
        self.state.read().default_rendering_params.shading_model
    }
    /// `SetDefaultShadingModel`; `viewer.set_default_shading_model(model)`.
    pub fn set_default_shading_model(&self, model: Graphic3dTypeOfShadingModel) {
        self.state.write().default_rendering_params.shading_model = model;
    }
    /// `DefaultTypeOfView`; `viewer.default_type_of_view()`.
    pub fn default_type_of_view(&self) -> V3dTypeOfView {
        self.state.read().default_type_of_view
    }
    /// `SetDefaultTypeOfView`; `viewer.set_default_type_of_view(V3dTypeOfView::Perspective)`.
    pub fn set_default_type_of_view(&self, kind: V3dTypeOfView) {
        self.state.write().default_type_of_view = kind;
    }
    /// `DefaultBackgroundColor`; `viewer.default_background_color()`.
    pub fn default_background_color(&self) -> Color {
        self.state.read().background.color()
    }
    /// `DefaultBgGradientColors`; `viewer.default_bg_gradient_colors()`.
    pub fn default_bg_gradient_colors(&self) -> (Color, Color) {
        self.state.read().gradient_background.colors()
    }
    /// `ComputedMode`; `viewer.computed_mode()`.
    pub fn computed_mode(&self) -> bool {
        self.state.read().computed_mode
    }
    /// `SetComputedMode`; `viewer.set_computed_mode(false)`.
    pub fn set_computed_mode(&self, mode: bool) {
        self.state.write().computed_mode = mode;
    }
    /// `DefaultComputedMode`; `viewer.default_computed_mode()`.
    pub fn default_computed_mode(&self) -> bool {
        self.state.read().default_computed_mode
    }
    /// `SetDefaultComputedMode`; `viewer.set_default_computed_mode(true)`.
    pub fn set_default_computed_mode(&self, mode: bool) {
        self.state.write().default_computed_mode = mode;
    }
    /// `GetAllZLayers`: the layers of the driver; `viewer.get_all_z_layers()`.
    pub fn get_all_z_layers(&self) -> Vec<i32> {
        self.driver.get().map(|d| d.z_layers()).unwrap_or_default()
    }
    /// `AddZLayer`: a new layer before the top layer; `viewer.add_z_layer(&settings)?`.
    pub fn add_z_layer(
        &self,
        settings: &Graphic3dZLayerSettings,
    ) -> Result<Option<Graphic3dZLayerId>, Failure> {
        self.insert_layer_before(settings, GRAPHIC3D_ZLAYER_ID_TOP)
    }
    /// `InsertLayerBefore`: a new identifier, `None` when every identifier is used.
    /// Example: `viewer.insert_layer_before(&settings, GRAPHIC3D_ZLAYER_ID_TOP)?`.
    pub fn insert_layer_before(
        &self,
        settings: &Graphic3dZLayerSettings,
        layer_after: Graphic3dZLayerId,
    ) -> Result<Option<Graphic3dZLayerId>, Failure> {
        let Some(id) = self.next_layer_id() else {
            return Ok(None);
        };
        if let Some(driver) = self.driver.get() {
            driver.insert_layer_before(id, settings, layer_after)?;
        }
        Ok(Some(id))
    }
    /// `InsertLayerAfter`; `viewer.insert_layer_after(&settings, GRAPHIC3D_ZLAYER_ID_DEFAULT)?`.
    pub fn insert_layer_after(
        &self,
        settings: &Graphic3dZLayerSettings,
        layer_before: Graphic3dZLayerId,
    ) -> Result<Option<Graphic3dZLayerId>, Failure> {
        let Some(id) = self.next_layer_id() else {
            return Ok(None);
        };
        if let Some(driver) = self.driver.get() {
            driver.insert_layer_after(id, settings, layer_before)?;
        }
        Ok(Some(id))
    }
    fn next_layer_id(&self) -> Option<i32> {
        let mut state = self.state.write();
        let id = state.z_layer_gen_id.try_next_id()?;
        state.layer_ids.insert(id);
        Some(id)
    }
    /// `RemoveZLayer`: only the layers this viewer added; `viewer.remove_z_layer(id)?`.
    pub fn remove_z_layer(&self, layer_id: Graphic3dZLayerId) -> Result<bool, Failure> {
        {
            let state = self.state.read();
            let generator = &state.z_layer_gen_id;
            if !state.layer_ids.contains(&layer_id)
                || layer_id < generator.lower()
                || layer_id > generator.upper()
            {
                return Ok(false);
            }
        }
        if let Some(driver) = self.driver.get() {
            driver.remove_z_layer(layer_id)?;
        }
        let mut state = self.state.write();
        state.layer_ids.remove(&layer_id);
        state.z_layer_gen_id.free(layer_id);
        Ok(true)
    }
    /// `ZLayerSettings`; an unknown layer raises as the driver does. `viewer.z_layer_settings(id)?`.
    pub fn z_layer_settings(
        &self,
        layer_id: Graphic3dZLayerId,
    ) -> Result<Graphic3dZLayerSettings, Failure> {
        match self.driver.get() {
            Some(driver) => driver.z_layer_settings(layer_id),
            None => Ok(Graphic3dZLayerSettings::default()),
        }
    }
    /// `SetZLayerSettings`; `viewer.set_z_layer_settings(id, &settings)?`.
    pub fn set_z_layer_settings(
        &self,
        layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
    ) -> Result<(), Failure> {
        match self.driver.get() {
            Some(driver) => driver.set_z_layer_settings(layer_id, settings),
            None => Ok(()),
        }
    }
    /// `ActiveViews`; `viewer.active_views()`.
    pub fn active_views(&self) -> Vec<Handle<dyn V3dViewSource>> {
        self.state.read().active_views.iter().cloned().collect()
    }
    /// `LastActiveView`: exactly one active view; `viewer.last_active_view()`.
    pub fn last_active_view(&self) -> bool {
        self.state.read().active_views.len() == 1
    }
    /// `DefinedViews`; `viewer.defined_views()`.
    pub fn defined_views(&self) -> Vec<Handle<dyn V3dViewSource>> {
        self.state.read().defined_views.iter().cloned().collect()
    }
}
