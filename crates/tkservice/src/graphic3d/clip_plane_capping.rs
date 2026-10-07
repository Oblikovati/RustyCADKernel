//! The capping attributes of `Graphic3d_ClipPlane`, kept in its fill-area aspect, and the
//! capping flags that take them from the clipped object instead.
use super::{
    Graphic3dAspectFillArea3d, Graphic3dAspectsEdgeMembers, Graphic3dAspectsMembers,
    Graphic3dAspectsRenderMembers, Graphic3dCappingFlags, Graphic3dClipPlane, Graphic3dHatchStyle,
    Graphic3dMaterialAspect, Graphic3dTextureMap, Graphic3dTextureSet, Graphic3dTypeOfMaterial,
};
use crate::aspect::{AspectHatchStyle, AspectInteriorStyle};
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle};

/// The predefined hatch styles, to convert a `HatchType` back to the enumeration.
const PREDEFINED_HATCHES: [AspectHatchStyle; 13] = [
    AspectHatchStyle::Solid,
    AspectHatchStyle::GridDiagonal,
    AspectHatchStyle::GridDiagonalWide,
    AspectHatchStyle::Grid,
    AspectHatchStyle::GridWide,
    AspectHatchStyle::Diagonal45,
    AspectHatchStyle::Diagonal135,
    AspectHatchStyle::Horizontal,
    AspectHatchStyle::Vertical,
    AspectHatchStyle::Diagonal45Wide,
    AspectHatchStyle::Diagonal135Wide,
    AspectHatchStyle::HorizontalWide,
    AspectHatchStyle::VerticalWide,
];

/// `textures` with `texture` first when it holds exactly one texture, else a new set of it.
fn one_texture_set(
    textures: Handle<Graphic3dTextureSet>,
    texture: Handle<dyn Graphic3dTextureMap>,
) -> Result<Handle<Graphic3dTextureSet>, Failure> {
    match textures.get() {
        Some(set) if set.size() == 1 => set.set_first(texture)?,
        _ => return Ok(Handle::new(Graphic3dTextureSet::with_texture(texture))),
    }
    Ok(textures)
}

impl Graphic3dClipPlane {
    /// The capping aspect (never null); `plane.capping_aspect()`.
    pub fn capping_aspect(&self) -> Handle<Graphic3dAspectFillArea3d> {
        self.state.read().aspect.clone()
    }
    /// Replaces the capping aspect. Natively a null aspect is stored and crashes the next
    /// capping call; here it raises `Standard_ProgramError`. Example: `plane.set_capping_aspect(aspect)?`.
    pub fn set_capping_aspect(
        &self,
        aspect: Handle<Graphic3dAspectFillArea3d>,
    ) -> Result<(), Failure> {
        if aspect.is_null() {
            return Err(program_error(
                "Graphic3d_ClipPlane::SetCappingAspect() - null aspect; expected a fill-area aspect",
            ));
        }
        let mut state = self.state.write();
        state.aspect = aspect;
        state.aspect_mod = state.aspect_mod.wrapping_add(1);
        Ok(())
    }
    /// Runs `edit` on the capping aspect, then counts an aspect modification.
    fn edit_aspect<R>(&self, edit: impl FnOnce(&Graphic3dAspectFillArea3d) -> R) -> R {
        let result = edit(&self.capping_aspect());
        self.count_aspect_change();
        result
    }
    fn count_aspect_change(&self) {
        let mut state = self.state.write();
        state.aspect_mod = state.aspect_mod.wrapping_add(1);
    }
    /// The material color of an `Aspect` material, else the interior color.
    /// Example: `plane.capping_color()`.
    pub fn capping_color(&self) -> Color {
        let aspect = self.capping_aspect();
        let material = aspect.front_material();
        match material.material_type() {
            Graphic3dTypeOfMaterial::Aspect => material.color(),
            Graphic3dTypeOfMaterial::Physic => aspect.interior_color(),
        }
    }
    /// Sets the interior and the front material colors; `plane.set_capping_color(red)?`.
    pub fn set_capping_color(&self, color: Color) -> Result<(), Failure> {
        let aspect = self.capping_aspect();
        aspect.set_interior_color(color);
        aspect.change_front_material(|material| material.set_color(color))?;
        self.count_aspect_change();
        Ok(())
    }
    /// Sets the front material; a physical material also sets the interior color.
    /// Example: `plane.set_capping_material(material)`.
    pub fn set_capping_material(&self, material: Graphic3dMaterialAspect) {
        self.edit_aspect(|aspect| {
            let color = material.color();
            aspect.set_front_material(material);
            if aspect.front_material().material_type() != Graphic3dTypeOfMaterial::Aspect {
                aspect.set_interior_color(color);
            }
        });
    }
    /// The front material of the capping aspect; `plane.capping_material()`.
    pub fn capping_material(&self) -> Graphic3dMaterialAspect {
        self.capping_aspect().front_material()
    }
    /// Sets (or with null removes) the capping texture, reusing a one-texture set.
    /// Example: `plane.set_capping_texture(texture)?`.
    pub fn set_capping_texture(
        &self,
        texture: Handle<dyn Graphic3dTextureMap>,
    ) -> Result<(), Failure> {
        let aspect = self.capping_aspect();
        if texture.is_null() {
            aspect.set_texture_map_on(false);
            aspect.set_texture_set(Handle::default());
            self.count_aspect_change();
            return Ok(());
        }
        aspect.set_texture_map_on(true);
        aspect.set_texture_set(one_texture_set(aspect.texture_set(), texture)?);
        self.count_aspect_change();
        Ok(())
    }
    /// The first texture of the capping texture set, or null; `plane.capping_texture()`.
    pub fn capping_texture(&self) -> Handle<dyn Graphic3dTextureMap> {
        let textures = self.capping_aspect().texture_set();
        match textures.get() {
            Some(set) if !set.is_empty() => set.first().unwrap_or_default(),
            _ => Handle::default(),
        }
    }
    /// Sets a predefined hatch style (SOLID removes it); `plane.set_capping_hatch(style)`.
    pub fn set_capping_hatch(&self, style: AspectHatchStyle) {
        self.edit_aspect(|aspect| aspect.set_hatch_style_type(style));
    }
    /// The predefined hatch style. Natively a missing style is a null dereference and a
    /// custom one an out-of-range cast; both raise `Standard_ProgramError` here.
    /// Example: `plane.capping_hatch()?`.
    pub fn capping_hatch(&self) -> Result<AspectHatchStyle, Failure> {
        let hatch = self.capping_aspect().hatch_style();
        let Some(style) = hatch.get() else {
            return Err(program_error(
                "Graphic3d_ClipPlane::CappingHatch() - no hatch style; expected a hatch style",
            ));
        };
        let kind = style.hatch_type();
        PREDEFINED_HATCHES
            .into_iter()
            .find(|candidate| *candidate as i32 == kind)
            .ok_or_else(|| {
                program_error(&format!(
                    "Graphic3d_ClipPlane::CappingHatch() - hatch type {kind}; expected a predefined Aspect_HatchStyle (0..12)"
                ))
            })
    }
    /// Sets a custom hatch style; `plane.set_capping_custom_hatch(style)`.
    pub fn set_capping_custom_hatch(&self, style: Handle<Graphic3dHatchStyle>) {
        self.edit_aspect(|aspect| aspect.set_hatch_style(style));
    }
    /// The hatch style of the capping aspect; `plane.capping_custom_hatch()`.
    pub fn capping_custom_hatch(&self) -> Handle<Graphic3dHatchStyle> {
        self.capping_aspect().hatch_style()
    }
    /// Draws the capping hatched; `plane.set_capping_hatch_on()`.
    pub fn set_capping_hatch_on(&self) {
        self.edit_aspect(|aspect| aspect.set_interior_style(AspectInteriorStyle::Hatch));
    }
    /// Draws the capping solid; `plane.set_capping_hatch_off()`.
    pub fn set_capping_hatch_off(&self) {
        self.edit_aspect(|aspect| aspect.set_interior_style(AspectInteriorStyle::Solid));
    }
    /// Whether the capping is hatched; `plane.is_hatch_on()`.
    pub fn is_hatch_on(&self) -> bool {
        self.capping_aspect().interior_style() == AspectInteriorStyle::Hatch
    }
    fn has_flag(&self, flag: Graphic3dCappingFlags) -> bool {
        self.state.read().flags & flag.0 != 0
    }
    /// `setCappingFlag`: sets or clears `flag`, counting an aspect modification.
    fn set_capping_flag(&self, to_use: bool, flag: Graphic3dCappingFlags) {
        let mut state = self.state.write();
        state.flags = if to_use {
            state.flags | flag.0
        } else {
            state.flags & !flag.0
        };
        state.aspect_mod = state.aspect_mod.wrapping_add(1);
    }
    /// Whether the object material is used; `plane.to_use_object_material()`.
    pub fn to_use_object_material(&self) -> bool {
        self.has_flag(Graphic3dCappingFlags::OBJECT_MATERIAL)
    }
    /// Uses the object material; `plane.set_use_object_material(true)`.
    pub fn set_use_object_material(&self, to_use: bool) {
        self.set_capping_flag(to_use, Graphic3dCappingFlags::OBJECT_MATERIAL);
    }
    /// Whether the object texture is used; `plane.to_use_object_texture()`.
    pub fn to_use_object_texture(&self) -> bool {
        self.has_flag(Graphic3dCappingFlags::OBJECT_TEXTURE)
    }
    /// Uses the object texture; `plane.set_use_object_texture(true)`.
    pub fn set_use_object_texture(&self, to_use: bool) {
        self.set_capping_flag(to_use, Graphic3dCappingFlags::OBJECT_TEXTURE);
    }
    /// Whether the object shader is used; `plane.to_use_object_shader()`.
    pub fn to_use_object_shader(&self) -> bool {
        self.has_flag(Graphic3dCappingFlags::OBJECT_SHADER)
    }
    /// Uses the object shader; `plane.set_use_object_shader(true)`.
    pub fn set_use_object_shader(&self, to_use: bool) {
        self.set_capping_flag(to_use, Graphic3dCappingFlags::OBJECT_SHADER);
    }
    /// Whether any object property is used; `plane.to_use_object_properties()`.
    pub fn to_use_object_properties(&self) -> bool {
        self.state.read().flags != Graphic3dCappingFlags::NONE.0
    }
}
