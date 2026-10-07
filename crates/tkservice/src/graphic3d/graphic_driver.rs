//! `Graphic3d_GraphicDriver`: the abstract graphic driver (visualization-design.md section
//! 1): the [`Graphic3dGraphicDriver`] trait over the embedded [`Graphic3dGraphicDriverBase`]
//! (display connection, structure identifiers and Z layers), with the non-virtual members in
//! [`Graphic3dGraphicDriverMembers`].
use super::{
    GRAPHIC3D_ZLAYER_ID_BOT_OSD, GRAPHIC3D_ZLAYER_ID_DEFAULT, GRAPHIC3D_ZLAYER_ID_TOP,
    GRAPHIC3D_ZLAYER_ID_TOP_OSD, GRAPHIC3D_ZLAYER_ID_TOPMOST, GRAPHIC3D_ZLAYER_ID_UNKNOWN,
    Graphic3dCStructure, Graphic3dCView, Graphic3dLayer, Graphic3dPolygonOffset,
    Graphic3dStructureManager, Graphic3dTypeOfLimit, Graphic3dZLayerId, Graphic3dZLayerSettings,
};
use crate::aspect::{AspectDisplayConnection, AspectGenId, AspectWindow};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::{out_of_range, program_error};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkernel::tcollection::AsciiString;
use std::collections::HashMap;
use std::io;

/// The protected fields of a graphic driver.
pub struct Graphic3dGraphicDriverFields {
    /// `myDisplayConnection`.
    pub display_connection: Handle<AspectDisplayConnection>,
    /// `myStructGenId`: the structure identifiers.
    pub struct_gen_id: AspectGenId,
    /// `myLayers`: the layers in rendering order.
    pub layers: Vec<Handle<Graphic3dLayer>>,
    /// `myLayerIds`: the layers by identifier.
    pub layer_ids: HashMap<Graphic3dZLayerId, Handle<Graphic3dLayer>>,
}

/// The embedded state of a graphic driver.
pub struct Graphic3dGraphicDriverBase {
    /// The fields, shared by every handle.
    pub fields: SharedState<Graphic3dGraphicDriverFields>,
}

/// The settings of a default layer: name and the immediate, ray-tracing, environment
/// texture, depth test, depth write and depth clear flags.
fn default_layer(id: Graphic3dZLayerId, name: &str, flags: [bool; 6]) -> Handle<Graphic3dLayer> {
    let mut settings = Graphic3dZLayerSettings::default();
    settings.set_name(&AsciiString::from(name));
    settings.set_immediate(flags[0]);
    settings.set_raytracable(flags[1]);
    settings.set_environment_texture(flags[2]);
    settings.set_enable_depth_test(flags[3]);
    settings.set_enable_depth_write(flags[4]);
    settings.set_clear_depth(flags[5]);
    settings.set_polygon_offset(Graphic3dPolygonOffset::default());
    let layer = Graphic3dLayer::new(id, None);
    layer.set_layer_settings(&settings);
    Handle::new(layer)
}

impl Graphic3dGraphicDriverBase {
    /// The protected constructor: the display connection and the five default layers
    /// (UNDERLAY, DEFAULT, TOP, TOPMOST, OVERLAY); `Graphic3dGraphicDriverBase::new(&display)`.
    pub fn new(display: &Handle<AspectDisplayConnection>) -> Self {
        let layers = vec![
            default_layer(
                GRAPHIC3D_ZLAYER_ID_BOT_OSD,
                "UNDERLAY",
                [false, false, false, false, false, false],
            ),
            default_layer(
                GRAPHIC3D_ZLAYER_ID_DEFAULT,
                "DEFAULT",
                [false, true, true, true, true, false],
            ),
            default_layer(
                GRAPHIC3D_ZLAYER_ID_TOP,
                "TOP",
                [true, false, true, true, true, false],
            ),
            default_layer(
                GRAPHIC3D_ZLAYER_ID_TOPMOST,
                "TOPMOST",
                [true, false, true, true, true, true],
            ),
            default_layer(
                GRAPHIC3D_ZLAYER_ID_TOP_OSD,
                "OVERLAY",
                [true, false, false, false, false, false],
            ),
        ];
        let layer_ids = layers
            .iter()
            .map(|l| (l.get().map_or(0, Graphic3dLayer::layer_id), l.clone()))
            .collect();
        let fields = Graphic3dGraphicDriverFields {
            display_connection: display.clone(),
            struct_gen_id: AspectGenId::default(),
            layers,
            layer_ids,
        };
        Self {
            fields: SharedState::new(fields),
        }
    }
    /// `NewIdentification`: a new structure identifier; `base.new_identification()?`.
    pub fn new_identification(&self) -> Result<i32, Failure> {
        self.fields.write().struct_gen_id.next_id()
    }
    /// The layer of an identifier, or null.
    fn layer(&self, layer_id: Graphic3dZLayerId) -> Handle<Graphic3dLayer> {
        self.fields
            .read()
            .layer_ids
            .get(&layer_id)
            .cloned()
            .unwrap_or_default()
    }
    /// A new layer of `settings` for `InsertLayerBefore` / `InsertLayerAfter`, after the checks.
    fn new_layer(
        &self,
        member: &str,
        layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
    ) -> Result<Handle<Graphic3dLayer>, Failure> {
        if layer_id <= 0 {
            return Err(program_error(&format!(
                "Graphic3d_GraphicDriver::{member}, negative and zero IDs are reserved: {layer_id}; expected > 0"
            )));
        }
        if self.fields.read().layer_ids.contains_key(&layer_id) {
            return Err(program_error(&format!(
                "Graphic3d_GraphicDriver::{member}, Layer with theLayerId already exists: {layer_id}"
            )));
        }
        let layer = Graphic3dLayer::new(layer_id, None);
        layer.set_layer_settings(settings);
        Ok(Handle::new(layer))
    }
    /// The body of `InsertLayerBefore`: before `layer_after`, or first when it is unknown;
    /// non-positive or existing identifiers raise; `base.insert_layer_before(1, &settings, 0)?`.
    pub fn insert_layer_before(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_after: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        let layer = self.new_layer("InsertLayerBefore", new_layer_id, settings)?;
        let other = if layer_after != GRAPHIC3D_ZLAYER_ID_UNKNOWN {
            self.layer(layer_after)
        } else {
            Handle::default()
        };
        let mut fields = self.fields.write();
        match fields
            .layers
            .iter()
            .position(|l| !other.is_null() && *l == other)
        {
            Some(index) => fields.layers.insert(index, layer.clone()),
            None if other.is_null() => fields.layers.insert(0, layer.clone()),
            None => {}
        }
        fields.layer_ids.insert(new_layer_id, layer);
        Ok(())
    }
    /// The body of `InsertLayerAfter`: after `layer_before`, or last when it is unknown;
    /// `base.insert_layer_after(1, &settings, 0)?`.
    pub fn insert_layer_after(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_before: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        let layer = self.new_layer("InsertLayerAfter", new_layer_id, settings)?;
        let other = if layer_before != GRAPHIC3D_ZLAYER_ID_UNKNOWN {
            self.layer(layer_before)
        } else {
            Handle::default()
        };
        let mut fields = self.fields.write();
        match fields
            .layers
            .iter()
            .position(|l| !other.is_null() && *l == other)
        {
            Some(index) => fields.layers.insert(index + 1, layer.clone()),
            None if other.is_null() => fields.layers.push(layer.clone()),
            None => {}
        }
        fields.layer_ids.insert(new_layer_id, layer);
        Ok(())
    }
    /// The body of `RemoveZLayer`: non-positive or unknown identifiers raise; `base.remove_z_layer(1)?`.
    pub fn remove_z_layer(&self, layer_id: Graphic3dZLayerId) -> Result<(), Failure> {
        if layer_id <= 0 {
            return Err(program_error(&format!(
                "Graphic3d_GraphicDriver::RemoveZLayer, negative and zero IDs are reserved and cannot be removed: {layer_id}"
            )));
        }
        let layer = self.layer(layer_id);
        if layer.is_null() {
            return Err(program_error(&format!(
                "Graphic3d_GraphicDriver::RemoveZLayer, Layer with theLayerId does not exist: {layer_id}"
            )));
        }
        let mut fields = self.fields.write();
        fields.layers.retain(|l| *l != layer);
        fields.layer_ids.remove(&layer_id);
        Ok(())
    }
    /// The body of `SetZLayerSettings`: an unknown identifier raises; `base.set_z_layer_settings(0, &settings)?`.
    pub fn set_z_layer_settings(
        &self,
        layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
    ) -> Result<(), Failure> {
        let layer = self.layer(layer_id);
        let Some(layer) = layer.get() else {
            return Err(program_error(&format!(
                "Graphic3d_GraphicDriver::SetZLayerSettings, Layer with theLayerId does not exist: {layer_id}"
            )));
        };
        layer.set_layer_settings(settings);
        Ok(())
    }
}

/// A graphic driver; the rendering members are implemented by the driver crate.
pub trait Graphic3dGraphicDriver: Transient {
    /// The embedded base; `driver.graphic_driver_base()`.
    fn graphic_driver_base(&self) -> &Graphic3dGraphicDriverBase;
    /// `InquireLimit`: a resource limit.
    fn inquire_limit(&self, limit: Graphic3dTypeOfLimit) -> i32;
    /// `CreateStructure`: a new driver structure of the manager.
    fn create_structure(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure>;
    /// `RemoveStructure`: releases a driver structure (and nullifies the handle).
    fn remove_structure(&self, cstructure: &mut Handle<dyn Graphic3dCStructure>);
    /// `CreateView`: a new view of the manager.
    fn create_view(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCView>, Failure>;
    /// `RemoveView`: releases a view.
    fn remove_view(&self, view: &Handle<dyn Graphic3dCView>);
    /// `EnableVBO`.
    fn enable_vbo(&self, status: bool);
    /// `IsVerticalSync`.
    fn is_vertical_sync(&self) -> bool;
    /// `SetVerticalSync`.
    fn set_vertical_sync(&self, to_enable: bool);
    /// `MemoryInfo`: whether the information is known, with the free bytes and a description.
    fn memory_info(&self, free_bytes: &mut usize, info: &mut AsciiString) -> bool;
    /// `DefaultTextHeight`.
    fn default_text_height(&self) -> f32;
    /// `TextSize`: `[width, ascent, descent]` of a text in a view.
    fn text_size(&self, view: &Handle<dyn Graphic3dCView>, text: &str, height: f32) -> [f32; 3];
    /// `InsertLayerBefore` (pure natively, with the base body in
    /// [`Graphic3dGraphicDriverBase::insert_layer_before`]).
    fn insert_layer_before(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_after: Graphic3dZLayerId,
    ) -> Result<(), Failure>;
    /// `InsertLayerAfter` (pure natively, base body in [`Graphic3dGraphicDriverBase::insert_layer_after`]).
    fn insert_layer_after(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_before: Graphic3dZLayerId,
    ) -> Result<(), Failure>;
    /// `RemoveZLayer` (pure natively, base body in [`Graphic3dGraphicDriverBase::remove_z_layer`]).
    fn remove_z_layer(&self, layer_id: Graphic3dZLayerId) -> Result<(), Failure>;
    /// `ZLayers`: the identifiers of the normal layers, then of the immediate ones;
    /// `driver.z_layers()`.
    fn z_layers(&self) -> Vec<i32> {
        let layers = self.graphic_driver_base().fields.read().layers.clone();
        let (immediate, normal): (Vec<_>, Vec<_>) = layers
            .iter()
            .filter_map(Handle::get)
            .partition(|l| l.is_immediate());
        normal
            .iter()
            .chain(immediate.iter())
            .map(|l| l.layer_id())
            .collect()
    }
    /// `SetZLayerSettings` (pure natively, base body in [`Graphic3dGraphicDriverBase::set_z_layer_settings`]).
    fn set_z_layer_settings(
        &self,
        layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
    ) -> Result<(), Failure>;
    /// `ZLayerSettings`: the settings of a layer; an unknown identifier raises
    /// `Standard_OutOfRange`; `driver.z_layer_settings(0)?`.
    fn z_layer_settings(
        &self,
        layer_id: Graphic3dZLayerId,
    ) -> Result<Graphic3dZLayerSettings, Failure> {
        let layer = self.graphic_driver_base().layer(layer_id);
        let Some(layer) = layer.get() else {
            return Err(out_of_range(&format!(
                "Graphic3d_GraphicDriver::ZLayerSettings, Layer with theLayerId does not exist: {layer_id}"
            )));
        };
        Ok(layer.layer_settings())
    }
    /// `ViewExists`: whether a view of the window exists, stored in `view`.
    fn view_exists(
        &self,
        window: &Handle<dyn AspectWindow>,
        view: &mut Handle<dyn Graphic3dCView>,
    ) -> bool;
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dGraphicDriver,
    "Graphic3d_GraphicDriver",
    StandardTransient
);

/// The non-virtual members of every graphic driver.
pub trait Graphic3dGraphicDriverMembers: Graphic3dGraphicDriver {
    /// `InquireLightLimit`; `driver.inquire_light_limit()`.
    fn inquire_light_limit(&self) -> i32 {
        self.inquire_limit(Graphic3dTypeOfLimit::MaxNbLights)
    }
    /// `InquirePlaneLimit`; `driver.inquire_plane_limit()`.
    fn inquire_plane_limit(&self) -> i32 {
        self.inquire_limit(Graphic3dTypeOfLimit::MaxNbClipPlanes)
    }
    /// `InquireViewLimit`; `driver.inquire_view_limit()`.
    fn inquire_view_limit(&self) -> i32 {
        self.inquire_limit(Graphic3dTypeOfLimit::MaxNbViews)
    }
    /// `GetDisplayConnection`; `driver.get_display_connection()`.
    fn get_display_connection(&self) -> Handle<AspectDisplayConnection> {
        self.graphic_driver_base()
            .fields
            .read()
            .display_connection
            .clone()
    }
    /// `NewIdentification`; `driver.new_identification()?`.
    fn new_identification(&self) -> Result<i32, Failure> {
        self.graphic_driver_base().new_identification()
    }
    /// `RemoveIdentification`; `driver.remove_identification(3)`.
    fn remove_identification(&self, id: i32) {
        self.graphic_driver_base()
            .fields
            .write()
            .struct_gen_id
            .free(id);
    }
    /// `DumpJson`: the identifier generator and the layers; `driver.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        super::driver_dump::dump_graphic_driver(self, stream, depth)
    }
    /// The layers in rendering order (`myLayers`); `driver.layers()`.
    fn layers(&self) -> Vec<Handle<Graphic3dLayer>> {
        self.graphic_driver_base().fields.read().layers.clone()
    }
}
impl<T: Graphic3dGraphicDriver + ?Sized> Graphic3dGraphicDriverMembers for T {}
