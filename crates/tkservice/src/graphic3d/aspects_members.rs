//! The interior, material, texture, shader and polygon-offset members of `Graphic3d_Aspects`.
use super::{
    Graphic3dAlphaMode, Graphic3dAspectsSource, Graphic3dMaterialAspect, Graphic3dPolygonOffset,
    Graphic3dShaderProgram, Graphic3dTextureMap, Graphic3dTextureSet,
    Graphic3dTypeOfBackfacingModel, Graphic3dTypeOfShadingModel,
};
use crate::aspect::{AspectInteriorStyle, AspectPolygonOffsetMode};
use rustycad_tkernel::quantity::{Color, ColorRgba};
use rustycad_tkernel::standard::Handle;

/// Reads one field of the aspects.
macro_rules! read_field {
    ($self:ident . $field:ident) => {
        $self.aspects_base().fields.read().$field
    };
}

/// The interior, material and texture members of `Graphic3d_Aspects`.
pub trait Graphic3dAspectsMembers: Graphic3dAspectsSource {
    /// The interior style; `aspects.interior_style()`.
    fn interior_style(&self) -> AspectInteriorStyle {
        read_field!(self.interior_style)
    }
    /// Sets the interior style; `aspects.set_interior_style(AspectInteriorStyle::Hatch)`.
    fn set_interior_style(&self, style: AspectInteriorStyle) {
        self.aspects_base().fields.write().interior_style = style;
    }
    /// The shading model; `aspects.shading_model()`.
    fn shading_model(&self) -> Graphic3dTypeOfShadingModel {
        read_field!(self.shading_model)
    }
    /// Sets the shading model; `aspects.set_shading_model(Graphic3dTypeOfShadingModel::Phong)`.
    fn set_shading_model(&self, model: Graphic3dTypeOfShadingModel) {
        self.aspects_base().fields.write().shading_model = model;
    }
    /// The alpha mode; `aspects.alpha_mode()`.
    fn alpha_mode(&self) -> Graphic3dAlphaMode {
        read_field!(self.alpha_mode)
    }
    /// The alpha cutoff; `aspects.alpha_cutoff()`.
    fn alpha_cutoff(&self) -> f32 {
        read_field!(self.alpha_cutoff)
    }
    /// Sets the alpha mode and cutoff (0.5 natively by default); `aspects.set_alpha_mode(mode, 0.5)`.
    fn set_alpha_mode(&self, mode: Graphic3dAlphaMode, cutoff: f32) {
        let mut fields = self.aspects_base().fields.write();
        (fields.alpha_mode, fields.alpha_cutoff) = (mode, cutoff);
    }
    /// The interior color with alpha (`ColorRGBA`, `InteriorColorRGBA`); `aspects.interior_color_rgba()`.
    fn interior_color_rgba(&self) -> ColorRgba {
        read_field!(self.interior_color)
    }
    /// The interior color (`Color`, `InteriorColor`); `aspects.interior_color()`.
    fn interior_color(&self) -> Color {
        *read_field!(self.interior_color).rgb()
    }
    /// Sets the interior RGB, keeping alpha (`SetColor`, `SetInteriorColor`); `aspects.set_interior_color(red)`.
    fn set_interior_color(&self, color: Color) {
        self.aspects_base()
            .fields
            .write()
            .interior_color
            .set_rgb(color);
    }
    /// Sets the interior color with alpha; `aspects.set_interior_color_rgba(rgba)`.
    fn set_interior_color_rgba(&self, color: ColorRgba) {
        self.aspects_base().fields.write().interior_color = color;
    }
    /// The back interior color; `aspects.back_interior_color()`.
    fn back_interior_color(&self) -> Color {
        *read_field!(self.back_interior_color).rgb()
    }
    /// The back interior color with alpha; `aspects.back_interior_color_rgba()`.
    fn back_interior_color_rgba(&self) -> ColorRgba {
        read_field!(self.back_interior_color)
    }
    /// Sets the back interior RGB, keeping alpha; `aspects.set_back_interior_color(red)`.
    fn set_back_interior_color(&self, color: Color) {
        self.aspects_base()
            .fields
            .write()
            .back_interior_color
            .set_rgb(color);
    }
    /// Sets the back interior color with alpha; `aspects.set_back_interior_color_rgba(rgba)`.
    fn set_back_interior_color_rgba(&self, color: ColorRgba) {
        self.aspects_base().fields.write().back_interior_color = color;
    }
    /// The front material; `aspects.front_material()`.
    fn front_material(&self) -> Graphic3dMaterialAspect {
        read_field!(self.front_material).clone()
    }
    /// `ChangeFrontMaterial`: edits the front material in place; `aspects.change_front_material(|m| m.set_transparency(0.5))`.
    fn change_front_material<R>(&self, edit: impl FnOnce(&mut Graphic3dMaterialAspect) -> R) -> R {
        edit(&mut self.aspects_base().fields.write().front_material)
    }
    /// Sets the front material; `aspects.set_front_material(material)`.
    fn set_front_material(&self, material: Graphic3dMaterialAspect) {
        self.aspects_base().fields.write().front_material = material;
    }
    /// The back material; `aspects.back_material()`.
    fn back_material(&self) -> Graphic3dMaterialAspect {
        read_field!(self.back_material).clone()
    }
    /// `ChangeBackMaterial`: edits the back material in place; `aspects.change_back_material(|m| m.reset())`.
    fn change_back_material<R>(&self, edit: impl FnOnce(&mut Graphic3dMaterialAspect) -> R) -> R {
        edit(&mut self.aspects_base().fields.write().back_material)
    }
    /// Sets the back material; `aspects.set_back_material(material)`.
    fn set_back_material(&self, material: Graphic3dMaterialAspect) {
        self.aspects_base().fields.write().back_material = material;
    }
    /// The face culling; `aspects.face_culling()`.
    fn face_culling(&self) -> Graphic3dTypeOfBackfacingModel {
        read_field!(self.face_culling)
    }
    /// Sets the face culling; `aspects.set_face_culling(Graphic3dTypeOfBackfacingModel::BackCulled)`.
    fn set_face_culling(&self, culling: Graphic3dTypeOfBackfacingModel) {
        self.aspects_base().fields.write().face_culling = culling;
    }
}
impl<T: Graphic3dAspectsSource + ?Sized> Graphic3dAspectsMembers for T {}

/// The material distinction, texture, shader and polygon-offset members of `Graphic3d_Aspects`.
pub trait Graphic3dAspectsRenderMembers: Graphic3dAspectsSource {
    /// Whether front and back materials differ; `aspects.distinguish()`.
    fn distinguish(&self) -> bool {
        read_field!(self.to_distinguish_materials)
    }
    /// Sets material distinction (`SetDistinguish`, `SetDistinguishOn/Off`); `aspects.set_distinguish(true)`.
    fn set_distinguish(&self, to_distinguish: bool) {
        self.aspects_base().fields.write().to_distinguish_materials = to_distinguish;
    }
    /// Whether vertex colors apply to back faces; `aspects.to_use_vertex_color_for_back_faces()`.
    fn to_use_vertex_color_for_back_faces(&self) -> bool {
        read_field!(self.to_use_vertex_color_for_back_faces)
    }
    /// Sets vertex colors on back faces; `aspects.set_use_vertex_color_for_back_faces(false)`.
    fn set_use_vertex_color_for_back_faces(&self, to_use: bool) {
        self.aspects_base()
            .fields
            .write()
            .to_use_vertex_color_for_back_faces = to_use;
    }
    /// The custom shader program; `aspects.shader_program()`.
    fn shader_program(&self) -> Handle<Graphic3dShaderProgram> {
        read_field!(self.program).clone()
    }
    /// Sets the shader program; `aspects.set_shader_program(program)`.
    fn set_shader_program(&self, program: Handle<Graphic3dShaderProgram>) {
        self.aspects_base().fields.write().program = program;
    }
    /// The texture set; `aspects.texture_set()`.
    fn texture_set(&self) -> Handle<Graphic3dTextureSet> {
        read_field!(self.texture_set).clone()
    }
    /// Sets the texture set; `aspects.set_texture_set(set)`.
    fn set_texture_set(&self, textures: Handle<Graphic3dTextureSet>) {
        self.aspects_base().fields.write().texture_set = textures;
    }
    /// The first texture of a non-empty set, else null; `aspects.texture_map()`.
    fn texture_map(&self) -> Handle<dyn Graphic3dTextureMap> {
        let set = self.texture_set();
        set.get()
            .filter(|set| !set.is_empty())
            .and_then(|set| set.first().ok())
            .unwrap_or_default()
    }
    /// Sets a one-texture set, or no set for a null texture; `aspects.set_texture_map(texture)`.
    fn set_texture_map(&self, texture: Handle<dyn Graphic3dTextureMap>) {
        let set = if texture.is_null() {
            Handle::default()
        } else {
            Handle::new(Graphic3dTextureSet::with_texture(texture))
        };
        self.aspects_base().fields.write().texture_set = set;
    }
    /// Whether texture mapping is on (`ToMapTexture`, `TextureMapState`); `aspects.to_map_texture()`.
    fn to_map_texture(&self) -> bool {
        read_field!(self.to_map_texture)
    }
    /// Sets texture mapping (`SetTextureMapOn/Off`); `aspects.set_texture_map_on(true)`.
    fn set_texture_map_on(&self, to_map: bool) {
        self.aspects_base().fields.write().to_map_texture = to_map;
    }
    /// The polygon offset; `aspects.polygon_offset()`.
    fn polygon_offset(&self) -> Graphic3dPolygonOffset {
        read_field!(self.polygon_offset)
    }
    /// Sets the polygon offset; `aspects.set_polygon_offset(offset)`.
    fn set_polygon_offset(&self, offset: Graphic3dPolygonOffset) {
        self.aspects_base().fields.write().polygon_offset = offset;
    }
    /// `PolygonOffsets`: `(mode, factor, units)`; `aspects.polygon_offsets()`.
    fn polygon_offsets(&self) -> (i32, f32, f32) {
        let offset = self.polygon_offset();
        (offset.mode.0, offset.factor, offset.units)
    }
    /// `SetPolygonOffsets`: the mode is masked by `Aspect_POM_Mask` (factor 1, units 0 natively
    /// by default); `aspects.set_polygon_offsets(1, 1.0, 0.0)`.
    fn set_polygon_offsets(&self, mode: i32, factor: f32, units: f32) {
        let mode = AspectPolygonOffsetMode(mode & AspectPolygonOffsetMode::MASK.0);
        self.set_polygon_offset(Graphic3dPolygonOffset {
            mode,
            factor,
            units,
        });
    }
}
impl<T: Graphic3dAspectsSource + ?Sized> Graphic3dAspectsRenderMembers for T {}
