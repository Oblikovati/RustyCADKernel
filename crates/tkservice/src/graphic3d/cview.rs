//! `Graphic3d_CView` and `Graphic3d_DataStructureManager`: the abstract view
//! (visualization-design.md section 1). The view is the [`Graphic3dCView`] trait over the
//! embedded [`Graphic3dCViewBase`]; its non-virtual members are split by topic in
//! `cview_members`, `cview_structures`, `cview_computed`, `cview_xr` and `cview_subviews`.
use super::cview_virtuals as defaults;
use super::{
    GRAPHIC3D_ZLAYER_ID_BOT_OSD, Graphic3dBufferType, Graphic3dCStructure, Graphic3dCamera,
    Graphic3dDiagnosticInfo, Graphic3dDisplayPriority, Graphic3dFrameStatsDict,
    Graphic3dGraduatedTrihedron, Graphic3dLayer, Graphic3dLightSet, Graphic3dRenderingParams,
    Graphic3dSequenceOfHClipPlane, Graphic3dStructureManager, Graphic3dStructureSource,
    Graphic3dTextureEnv, Graphic3dTextureMap, Graphic3dTypeOfBackfacingModel,
    Graphic3dTypeOfBackground, Graphic3dTypeOfVisualization, Graphic3dVertex, Graphic3dZLayerId,
    Graphic3dZLayerSettings,
};
use super::{Graphic3dCubeMap, Graphic3dStructureSet};
use crate::aspect::{
    AspectBackground, AspectFillMethod, AspectGradientBackground, AspectGridParams,
    AspectRenderingContext, AspectSkydomeBackground, AspectTypeOfTriedronPosition, AspectWindow,
    AspectXRSession,
};
use crate::image::ImagePixMap;
use rustycad_tkernel::ncollection::{Vec2, Vec3};
use rustycad_tkernel::quantity::{ColorRgba, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{
    Failure, Handle, SharedState, StandardTransient, Transient, WeakHandle,
};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Axis3;
use std::io;

/// `Graphic3d_DataStructureManager`: an object with a camera that structures are computed
/// for (the views).
pub trait Graphic3dDataStructureManager: Transient {
    /// `Camera`; `manager.camera()`.
    fn camera(&self) -> Handle<Graphic3dCamera>;
    /// `DumpJson`: the class name; `manager.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let _ = depth;
        super::driver_dump::dump_data_structure_manager(stream)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dDataStructureManager,
    "Graphic3d_DataStructureManager",
    StandardTransient
);

/// The protected fields of a view.
pub struct Graphic3dCViewFields {
    /// `myId`.
    pub id: i32,
    /// `myRenderParams`.
    pub render_params: Graphic3dRenderingParams,
    /// `mySubviews`.
    pub subviews: Vec<Handle<dyn Graphic3dCView>>,
    /// `myParentView` (a raw pointer natively).
    pub parent_view: WeakHandle<dyn Graphic3dCView>,
    /// `myIsSubviewComposer`.
    pub is_subview_composer: bool,
    /// `mySubviewCorner`.
    pub subview_corner: AspectTypeOfTriedronPosition,
    /// `mySubviewTopLeft`.
    pub subview_top_left: Vec2<i32>,
    /// `mySubviewMargins`.
    pub subview_margins: Vec2<i32>,
    /// `mySubviewSize`.
    pub subview_size: Vec2<f64>,
    /// `mySubviewOffset`.
    pub subview_offset: Vec2<f64>,
    /// `myStructureManager`.
    pub structure_manager: Handle<Graphic3dStructureManager>,
    /// `myCamera`.
    pub camera: Handle<Graphic3dCamera>,
    /// `myStructsToCompute`.
    pub structs_to_compute: Vec<Handle<dyn Graphic3dStructureSource>>,
    /// `myStructsComputed`.
    pub structs_computed: Vec<Handle<dyn Graphic3dStructureSource>>,
    /// `myStructsDisplayed`.
    pub structs_displayed: Graphic3dStructureSet,
    /// `myIsInComputedMode`.
    pub is_in_computed_mode: bool,
    /// `myIsActive`.
    pub is_active: bool,
    /// `myIsRemoved`.
    pub is_removed: bool,
    /// `myBackfacing`.
    pub backfacing: Graphic3dTypeOfBackfacingModel,
    /// `myVisualization`.
    pub visualization: Graphic3dTypeOfVisualization,
    /// `myZLayerTarget`.
    pub z_layer_target: Graphic3dZLayerId,
    /// `myZLayerRedrawMode`.
    pub z_layer_redraw_mode: bool,
    /// `myBgColor`.
    pub bg_color: ColorRgba,
    /// `myBackgroundImage`.
    pub background_image: Handle<dyn Graphic3dTextureMap>,
    /// `myCubeMapBackground`.
    pub cube_map_background: Handle<dyn Graphic3dCubeMap>,
    /// `myCubeMapIBL`.
    pub cube_map_ibl: Handle<dyn Graphic3dCubeMap>,
    /// `myTextureEnvData`.
    pub texture_env_data: Handle<Graphic3dTextureEnv>,
    /// `myGTrihedronData`.
    pub g_trihedron_data: Graphic3dGraduatedTrihedron,
    /// `myBackgroundType`.
    pub background_type: Graphic3dTypeOfBackground,
    /// `mySkydomeAspect`.
    pub skydome_aspect: AspectSkydomeBackground,
    /// `myToUpdateSkydome`.
    pub to_update_skydome: bool,
    /// `myXRSession`.
    pub xr_session: Handle<dyn AspectXRSession>,
    /// `myBackXRCamera`: the projection to restore after the XR session.
    pub back_xr_camera: Handle<Graphic3dCamera>,
    /// `myBaseXRCamera`: the neutral orientation of the head tracking.
    pub base_xr_camera: Handle<Graphic3dCamera>,
    /// `myPosedXRCamera`: the base camera with the head pose applied.
    pub posed_xr_camera: Handle<Graphic3dCamera>,
    /// `myPosedXRCameraCopy`.
    pub posed_xr_camera_copy: Handle<Graphic3dCamera>,
    /// `myUnitFactor`: the scale of a meter.
    pub unit_factor: f64,
}

/// The embedded state of a view.
pub struct Graphic3dCViewBase {
    /// The fields, shared by every handle.
    pub fields: SharedState<Graphic3dCViewFields>,
    /// The view itself (`this`), set by [`graphic3d_cview_register`].
    this: SharedState<WeakHandle<dyn Graphic3dCView>>,
}

impl Graphic3dCViewBase {
    /// The constructor fields: a new camera, inactive, wireframe, a black background; the
    /// identifier comes from [`graphic3d_cview_register`]. Example: `Graphic3dCViewBase::new(&manager)?`.
    pub fn new(manager: &Handle<Graphic3dStructureManager>) -> Result<Self, Failure> {
        let fields = Graphic3dCViewFields {
            id: 0,
            render_params: Graphic3dRenderingParams::new()?,
            subviews: Vec::new(),
            parent_view: WeakHandle::default(),
            is_subview_composer: false,
            subview_corner: AspectTypeOfTriedronPosition::LEFT_UPPER,
            subview_top_left: Vec2::new(0, 0),
            subview_margins: Vec2::new(0, 0),
            subview_size: Vec2::new(1.0, 1.0),
            subview_offset: Vec2::new(0.0, 0.0),
            structure_manager: manager.clone(),
            camera: Handle::new(Graphic3dCamera::default()),
            structs_to_compute: Vec::new(),
            structs_computed: Vec::new(),
            structs_displayed: Graphic3dStructureSet::default(),
            is_in_computed_mode: false,
            is_active: false,
            is_removed: false,
            backfacing: Graphic3dTypeOfBackfacingModel::Auto,
            visualization: Graphic3dTypeOfVisualization::Wireframe,
            z_layer_target: GRAPHIC3D_ZLAYER_ID_BOT_OSD,
            z_layer_redraw_mode: false,
            bg_color: ColorRgba::from_rgb(
                rustycad_tkernel::quantity::Color::named(NamedColor::BLACK),
                1.0,
            ),
            background_image: Handle::default(),
            cube_map_background: Handle::default(),
            cube_map_ibl: Handle::default(),
            texture_env_data: Handle::default(),
            g_trihedron_data: Graphic3dGraduatedTrihedron::default(),
            background_type: Graphic3dTypeOfBackground::None,
            skydome_aspect: AspectSkydomeBackground::default(),
            to_update_skydome: false,
            xr_session: Handle::default(),
            back_xr_camera: Handle::default(),
            base_xr_camera: Handle::default(),
            posed_xr_camera: Handle::default(),
            posed_xr_camera_copy: Handle::default(),
            unit_factor: 1.0,
        };
        Ok(Self {
            fields: SharedState::new(fields),
            this: SharedState::new(WeakHandle::default()),
        })
    }
    /// The view itself, null before [`graphic3d_cview_register`]; `base.this()`.
    pub fn this(&self) -> Handle<dyn Graphic3dCView> {
        self.this.read().upgrade()
    }
}

/// Records `view` as its own `this` and takes its identifier from the structure manager
/// (`myId = myStructureManager->Identification(this)` of the native constructor); every
/// driver calls it once the view handle exists. Example: `graphic3d_cview_register(&view)?`.
pub fn graphic3d_cview_register(view: &Handle<dyn Graphic3dCView>) -> Result<(), Failure> {
    let Some(value) = view.get() else {
        return Ok(());
    };
    *value.cview_base().this.write() = view.downgrade();
    let manager = value.cview_base().fields.read().structure_manager.clone();
    let id = match manager.get() {
        Some(manager) => manager.identification(view)?,
        None => 0,
    };
    value.cview_base().fields.write().id = id;
    Ok(())
}

impl Drop for Graphic3dCViewBase {
    /// The destructor releases the XR session and, unless removed, forgets the view in the
    /// structure manager.
    fn drop(&mut self) {
        let address = self.this.read().object_address();
        let mut fields = self.fields.write();
        fields.xr_session = Handle::default();
        if fields.is_removed {
            return;
        }
        if let Some(manager) = fields.structure_manager.get() {
            manager.un_identification(address, fields.id);
        }
    }
}

/// A view; the rendering members are implemented by the driver.
pub trait Graphic3dCView: Graphic3dDataStructureManager {
    /// The embedded base; `view.cview_base()`.
    fn cview_base(&self) -> &Graphic3dCViewBase;
    /// `Activate`: displays the accepted structures of the manager and updates; `view.activate()`.
    fn activate(&self) {
        defaults::activate(self);
    }
    /// `Deactivate`: erases the displayed structures and updates; `view.deactivate()`.
    fn deactivate(&self) {
        defaults::deactivate(self);
    }
    /// `Remove`: detaches the subviews and the parent, erases everything and forgets the view
    /// in the manager; `view.remove()`.
    fn remove(&self) {
        defaults::remove(self);
    }
    /// `SetCamera`; `view.set_camera(&camera)`.
    fn set_camera(&self, camera: &Handle<Graphic3dCamera>) {
        self.cview_base().fields.write().camera = camera.clone();
    }
    /// `ToFlipOutput`: false by default.
    fn to_flip_output(&self) -> bool {
        false
    }
    /// `SetToFlipOutput`: nothing by default.
    fn set_to_flip_output(&self, value: bool) {
        let _ = value;
    }
    /// `NumberOfDisplayedStructures`; `view.number_of_displayed_structures()`.
    fn number_of_displayed_structures(&self) -> i32 {
        self.cview_base().fields.read().structs_displayed.extent() as i32
    }
    /// `MinMaxValues(bool)`: the union of the layer boxes (void for an undefined view);
    /// `view.min_max_values(false)?`.
    fn min_max_values(&self, include_auxiliary: bool) -> Result<BoundingBox3, Failure> {
        defaults::min_max_values(self, include_auxiliary)
    }
    /// `ZFitAllBounds`: the primary and the graphic boxes; `view.z_fit_all_bounds()?`.
    fn z_fit_all_bounds(&self) -> Result<(BoundingBox3, BoundingBox3), Failure> {
        Ok((self.min_max_values(false)?, self.min_max_values(true)?))
    }
    /// `Redraw`.
    fn redraw(&self);
    /// `RedrawImmediate`.
    fn redraw_immediate(&self);
    /// `Invalidate`.
    fn invalidate(&self);
    /// `IsInvalidated`.
    fn is_invalidated(&self) -> bool;
    /// `Resized` (pure natively, with the base body [`defaults::resized`]).
    fn resized(&self);
    /// `SetImmediateModeDrawToFront`.
    fn set_immediate_mode_draw_to_front(&self, draw_to_front_buffer: bool) -> bool;
    /// `SetWindow`.
    fn set_window(
        &self,
        parent_view: &Handle<dyn Graphic3dCView>,
        window: &Handle<dyn AspectWindow>,
        context: AspectRenderingContext,
    );
    /// `Window`.
    fn window(&self) -> Handle<dyn AspectWindow>;
    /// `IsDefined`.
    fn is_defined(&self) -> bool;
    /// `BufferDump`.
    fn buffer_dump(&self, image: &ImagePixMap, buffer_type: &Graphic3dBufferType) -> bool;
    /// `ShadowMapDump`.
    fn shadow_map_dump(&self, image: &ImagePixMap, light_name: &AsciiString) -> bool;
    /// `InvalidateBVHData`.
    fn invalidate_bvh_data(&self, layer_id: Graphic3dZLayerId);
    /// `InsertLayerBefore`.
    fn insert_layer_before(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_after: Graphic3dZLayerId,
    ) -> Result<(), Failure>;
    /// `InsertLayerAfter`.
    fn insert_layer_after(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_before: Graphic3dZLayerId,
    ) -> Result<(), Failure>;
    /// `ZLayerMax`.
    fn z_layer_max(&self) -> i32;
    /// `Layers`: the layers in rendering order.
    fn layers(&self) -> Vec<Handle<Graphic3dLayer>>;
    /// `Layer`: a layer by identifier, or null.
    fn layer(&self, layer_id: Graphic3dZLayerId) -> Handle<Graphic3dLayer>;
    /// `InvalidateZLayerBoundingBox`: the box of one layer, or of every layer with transform
    /// persistence objects; `view.invalidate_z_layer_bounding_box(0)`.
    fn invalidate_z_layer_bounding_box(&self, layer_id: Graphic3dZLayerId) {
        defaults::invalidate_z_layer_bounding_box(self, layer_id);
    }
    /// `RemoveZLayer`.
    fn remove_z_layer(&self, layer_id: Graphic3dZLayerId) -> Result<(), Failure>;
    /// `SetZLayerSettings`.
    fn set_z_layer_settings(
        &self,
        layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
    ) -> Result<(), Failure>;
    /// `FBO`.
    fn fbo(&self) -> Handle<StandardTransient>;
    /// `SetFBO`.
    fn set_fbo(&self, fbo: &Handle<StandardTransient>);
    /// `FBOCreate`.
    fn fbo_create(&self, width: i32, height: i32) -> Handle<StandardTransient>;
    /// `FBORelease`.
    fn fbo_release(&self, fbo: &mut Handle<StandardTransient>);
    /// `FBOGetDimensions`: `[width, height, width max, height max]`.
    fn fbo_get_dimensions(&self, fbo: &Handle<StandardTransient>) -> [i32; 4];
    /// `FBOChangeViewport`.
    fn fbo_change_viewport(&self, fbo: &Handle<StandardTransient>, width: i32, height: i32);
    /// `CopySettings`: the rendering parameters, background, texture environment, shading,
    /// back-facing, a copy of the camera, lights and clip planes; `view.copy_settings(&other)?`.
    fn copy_settings(&self, other: &Handle<dyn Graphic3dCView>) -> Result<(), Failure> {
        defaults::copy_settings(self, other)
    }
    /// `Background`: the background color; `view.background()`.
    fn background(&self) -> AspectBackground {
        AspectBackground::new(*self.cview_base().fields.read().bg_color.rgb())
    }
    /// `SetBackground`: the background color (the alpha is kept); `view.set_background(&bg)`.
    fn set_background(&self, background: &AspectBackground) {
        self.cview_base()
            .fields
            .write()
            .bg_color
            .set_rgb(background.color());
    }
    /// `GradientBackground`.
    fn gradient_background(&self) -> AspectGradientBackground;
    /// `SetGradientBackground`.
    fn set_gradient_background(&self, background: &AspectGradientBackground);
    /// `SetBackgroundImage`.
    fn set_background_image(
        &self,
        texture: &Handle<dyn Graphic3dTextureMap>,
        to_update_pbr_env: bool,
    );
    /// `BackgroundImageStyle`.
    fn background_image_style(&self) -> AspectFillMethod;
    /// `SetBackgroundImageStyle`.
    fn set_background_image_style(&self, style: AspectFillMethod);
    /// `SetImageBasedLighting`.
    fn set_image_based_lighting(&self, to_enable_ibl: bool);
    /// `GridDisplay`: nothing by default.
    fn grid_display(&self, params: &AspectGridParams, plane: &Axis3) {
        let _ = (params, plane);
    }
    /// `GridErase`: nothing by default.
    fn grid_erase(&self) {}
    /// `ShaderGridEcho(x, y, point)`: false by default.
    fn shader_grid_echo(&self, x: i32, y: i32, point: &mut Graphic3dVertex) -> bool {
        let _ = (x, y, point);
        false
    }
    /// `ShaderGridEcho(x, y, point, displayPoint)`: the echo, copied to the display point.
    fn shader_grid_echo_display(
        &self,
        x: i32,
        y: i32,
        point: &mut Graphic3dVertex,
        display_point: &mut Graphic3dVertex,
    ) -> bool {
        if !self.shader_grid_echo(x, y, point) {
            return false;
        }
        *display_point = *point;
        true
    }
    /// `ShaderGridSnapPoint`: false by default.
    fn shader_grid_snap_point(
        &self,
        point: &Graphic3dVertex,
        grid_point: &mut Graphic3dVertex,
    ) -> bool {
        let _ = (point, grid_point);
        false
    }
    /// `SetTextureEnv`.
    fn set_texture_env(&self, texture_env: &Handle<Graphic3dTextureEnv>);
    /// `Lights`.
    fn lights(&self) -> Handle<Graphic3dLightSet>;
    /// `SetLights`.
    fn set_lights(&self, lights: &Handle<Graphic3dLightSet>);
    /// `ClipPlanes`.
    fn clip_planes(&self) -> Handle<Graphic3dSequenceOfHClipPlane>;
    /// `SetClipPlanes`.
    fn set_clip_planes(&self, planes: &Handle<Graphic3dSequenceOfHClipPlane>);
    /// `DiagnosticInformation` (pure natively, with the base body
    /// [`defaults::diagnostic_information`] that adds the XR device strings).
    fn diagnostic_information(
        &self,
        dict: &mut Graphic3dFrameStatsDict,
        flags: Graphic3dDiagnosticInfo,
    );
    /// `StatisticInformation()`.
    fn statistic_information(&self) -> AsciiString;
    /// `StatisticInformation(dict)`.
    fn statistic_information_dict(&self, dict: &mut Graphic3dFrameStatsDict);
    /// `InitXR`: an OpenVR session opened at need (keeping the camera to restore); true when
    /// open; `view.init_xr()`.
    fn init_xr(&self) -> bool {
        defaults::init_xr(self)
    }
    /// `ReleaseXR`: restores the camera projection and closes the session; `view.release_xr()`.
    fn release_xr(&self) {
        defaults::release_xr(self);
    }
    /// `ProcessXRInput`: opens or closes the session for the stereo mode and, when active,
    /// sets the camera from the head-mounted display; `view.process_xr_input()?`.
    fn process_xr_input(&self) -> Result<(), Failure> {
        defaults::process_xr_input(self)
    }
    /// `GetGraduatedTrihedron`; `view.get_graduated_trihedron()`.
    fn get_graduated_trihedron(&self) -> Graphic3dGraduatedTrihedron {
        self.cview_base().fields.read().g_trihedron_data.clone()
    }
    /// `GraduatedTrihedronDisplay`: nothing by default.
    fn graduated_trihedron_display(&self, data: &Graphic3dGraduatedTrihedron) {
        let _ = data;
    }
    /// `GraduatedTrihedronErase`: nothing by default.
    fn graduated_trihedron_erase(&self) {}
    /// `GraduatedTrihedronMinMaxValues`: nothing by default.
    fn graduated_trihedron_min_max_values(&self, min: Vec3<f32>, max: Vec3<f32>) {
        let _ = (min, max);
    }
    /// `DumpJson` of the view (dispatched by [`Graphic3dDataStructureManager::dump_json`]);
    /// `view.cview_dump_json(&mut stream, -1)?`.
    fn cview_dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        super::driver_dump::dump_cview(self, stream, depth)
    }
    /// `displayStructure`: displays a driver structure at a priority.
    fn display_cstructure(
        &self,
        cstructure: &Handle<dyn Graphic3dCStructure>,
        priority: Graphic3dDisplayPriority,
    );
    /// `eraseStructure`: erases a driver structure.
    fn erase_cstructure(&self, cstructure: &Handle<dyn Graphic3dCStructure>);
    /// `changeZLayer`: moves a driver structure to a layer.
    fn change_cstructure_z_layer(
        &self,
        cstructure: &Handle<dyn Graphic3dCStructure>,
        layer_id: Graphic3dZLayerId,
    );
    /// `changePriority`: moves a driver structure to a priority.
    fn change_cstructure_priority(
        &self,
        cstructure: &Handle<dyn Graphic3dCStructure>,
        priority: Graphic3dDisplayPriority,
    );
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dCView,
    "Graphic3d_CView",
    dyn Graphic3dDataStructureManager
);

impl<T: Graphic3dCView> Graphic3dDataStructureManager for T {
    /// `Camera`: the view camera.
    fn camera(&self) -> Handle<Graphic3dCamera> {
        self.cview_base().fields.read().camera.clone()
    }
    /// `DumpJson`: the view dump.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        self.cview_dump_json(stream, depth)
    }
}
