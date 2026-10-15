// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_VisMaterial.hxx and .cxx (the attribute,
// its values and conversions) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_VisMaterial`: the visualization material of shapes, defined in the common
//! (Phong) and/or the metal-roughness (PBR) model, with its alpha mode and face culling.
use super::support::downcast;
use super::{XCAFDocVisMaterialCommon, XCAFDocVisMaterialPbr};
use crate::json::field_raw_pointer;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::quantity::{Color, ColorRgba, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::short_real::short_real_epsilon;
use rustycad_tkernel::standard::{Failure, Guid, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::HAsciiString;
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::json::dump_attribute_json_base;
use rustycad_tklcaf::tdf::{TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfRelocationTable};
use rustycad_tkservice::graphic3d::{
    Graphic3dAlphaMode, Graphic3dPbrMaterial, Graphic3dTypeOfBackfacingModel,
};
use rustycad_tkservice::json_dump::{class_begin, field_dumped, field_integer, field_real};
use std::io;

/// `XCAFDoc_VisMaterial::GetID()`.
const VIS_MATERIAL_ID: Guid = guid_from_text("EBB00255-03A0-4845-BD3B-A70EEDEEFA78");

/// The values of `XCAFDoc_VisMaterial` (the raw name is transient data, not backed up).
#[derive(Clone)]
pub(super) struct VisMaterialFields {
    pub(super) pbr: XCAFDocVisMaterialPbr,
    pub(super) common: XCAFDocVisMaterialCommon,
    pub(super) alpha_mode: Graphic3dAlphaMode,
    pub(super) alpha_cut_off: f32,
    pub(super) face_culling: Graphic3dTypeOfBackfacingModel,
}

impl Default for VisMaterialFields {
    fn default() -> Self {
        let pbr = XCAFDocVisMaterialPbr {
            is_defined: false,
            ..Default::default()
        };
        let common = XCAFDocVisMaterialCommon {
            is_defined: false,
            ..Default::default()
        };
        Self {
            pbr,
            common,
            alpha_mode: Graphic3dAlphaMode::BlendAuto,
            alpha_cut_off: 0.5,
            face_culling: Graphic3dTypeOfBackfacingModel::Auto,
        }
    }
}

/// `XCAFDoc_VisMaterial`.
///
/// ```
/// use rustycad_tkxcaf::xcaf_doc::{XCAFDocVisMaterial, XCAFDocVisMaterialPbr};
/// let material = XCAFDocVisMaterial::new();
/// assert!(material.is_empty());
/// material.set_pbr_material(&XCAFDocVisMaterialPbr::default())?;
/// assert!(material.has_pbr_material() && !material.is_empty());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct XCAFDocVisMaterial {
    transient: StandardTransient,
    base: TdfAttributeBase,
    pub(super) fields: SharedState<VisMaterialFields>,
    raw_name: SharedState<Handle<HAsciiString>>,
}
rustycad_tkernel::impl_transient!(XCAFDocVisMaterial, "XCAFDoc_VisMaterial", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl XCAFDocVisMaterial {
    /// `XCAFDoc_VisMaterial()`: undefined, blend-auto alpha with a 0.5 cut-off, automatic
    /// culling.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        VIS_MATERIAL_ID
    }
    /// Backs up, then edits the values.
    fn edit(&self, change: impl FnOnce(&mut VisMaterialFields)) -> Result<(), Failure> {
        self.backup()?;
        change(&mut self.fields.write());
        Ok(())
    }
    /// `IsEmpty`: neither definition.
    pub fn is_empty(&self) -> bool {
        let fields = self.fields.read();
        !fields.pbr.is_defined && !fields.common.is_defined
    }
    /// `HasPbrMaterial`.
    pub fn has_pbr_material(&self) -> bool {
        self.fields.read().pbr.is_defined
    }
    /// `PbrMaterial`.
    pub fn pbr_material(&self) -> XCAFDocVisMaterialPbr {
        self.fields.read().pbr.clone()
    }
    /// `SetPbrMaterial(theMaterial)`.
    pub fn set_pbr_material(&self, material: &XCAFDocVisMaterialPbr) -> Result<(), Failure> {
        self.edit(|fields| fields.pbr = material.clone())
    }
    /// `UnsetPbrMaterial`: an undefined default PBR definition.
    pub fn unset_pbr_material(&self) -> Result<(), Failure> {
        self.set_pbr_material(&XCAFDocVisMaterialPbr {
            is_defined: false,
            ..Default::default()
        })
    }
    /// `HasCommonMaterial`.
    pub fn has_common_material(&self) -> bool {
        self.fields.read().common.is_defined
    }
    /// `CommonMaterial`.
    pub fn common_material(&self) -> XCAFDocVisMaterialCommon {
        self.fields.read().common.clone()
    }
    /// `SetCommonMaterial(theMaterial)`.
    pub fn set_common_material(&self, material: &XCAFDocVisMaterialCommon) -> Result<(), Failure> {
        self.edit(|fields| fields.common = material.clone())
    }
    /// `UnsetCommonMaterial`: an undefined default common definition.
    pub fn unset_common_material(&self) -> Result<(), Failure> {
        self.set_common_material(&XCAFDocVisMaterialCommon {
            is_defined: false,
            ..Default::default()
        })
    }
}

/// Alpha, culling, name and equality.
impl XCAFDocVisMaterial {
    /// `BaseColor`: the PBR base color, else the common diffuse color with its opacity, else
    /// white.
    pub fn base_color(&self) -> ColorRgba {
        let fields = self.fields.read();
        if fields.pbr.is_defined {
            return fields.pbr.base_color;
        }
        if fields.common.is_defined {
            return ColorRgba::from_rgb(
                fields.common.diffuse_color,
                1.0 - fields.common.transparency,
            );
        }
        ColorRgba::from_rgb(Color::named(NamedColor::WHITE), 1.0)
    }
    /// `AlphaMode`.
    pub fn alpha_mode(&self) -> Graphic3dAlphaMode {
        self.fields.read().alpha_mode
    }
    /// `AlphaCutOff`.
    pub fn alpha_cut_off(&self) -> f32 {
        self.fields.read().alpha_cut_off
    }
    /// `SetAlphaMode(theMode, theCutOff)` (the native default cut-off is 0.5).
    pub fn set_alpha_mode(&self, mode: Graphic3dAlphaMode, cut_off: f32) -> Result<(), Failure> {
        self.edit(|fields| {
            fields.alpha_mode = mode;
            fields.alpha_cut_off = cut_off;
        })
    }
    /// `FaceCulling`.
    pub fn face_culling(&self) -> Graphic3dTypeOfBackfacingModel {
        self.fields.read().face_culling
    }
    /// `SetFaceCulling(theFaceCulling)`.
    pub fn set_face_culling(&self, culling: Graphic3dTypeOfBackfacingModel) -> Result<(), Failure> {
        self.edit(|fields| fields.face_culling = culling)
    }
    /// `IsDoubleSided` (deprecated): the culling is `DoubleSided`.
    pub fn is_double_sided(&self) -> bool {
        self.face_culling() == Graphic3dTypeOfBackfacingModel::DoubleSided
    }
    /// `SetDoubleSided(theIsDoubleSided)` (deprecated): `DoubleSided`, else `Auto`.
    pub fn set_double_sided(&self, double_sided: bool) -> Result<(), Failure> {
        let culling = if double_sided {
            Graphic3dTypeOfBackfacingModel::DoubleSided
        } else {
            Graphic3dTypeOfBackfacingModel::Auto
        };
        self.set_face_culling(culling)
    }
    /// `RawName`.
    pub fn raw_name(&self) -> Handle<HAsciiString> {
        self.raw_name.read().clone()
    }
    /// `SetRawName(theName)`: not backed up (transient data).
    pub fn set_raw_name(&self, name: &Handle<HAsciiString>) {
        *self.raw_name.write() = name.clone();
    }
    /// `IsEqual(theOther)`: the same object, or equal culling, alpha and definitions
    /// (`Standard_NullObject` for a null other, a null dereference natively).
    pub fn is_equal(&self, other: &Handle<XCAFDocVisMaterial>) -> Result<bool, Failure> {
        if std::ptr::eq(other.object_address(), (self as *const Self).cast()) {
            return Ok(true);
        }
        let other = rustycad_tklcaf::live(other, "XCAFDoc_VisMaterial::IsEqual")?
            .fields
            .read()
            .clone();
        let own = self.fields.read();
        Ok(other.face_culling == own.face_culling
            && other.alpha_cut_off == own.alpha_cut_off
            && other.alpha_mode == own.alpha_mode
            && other.common.is_equal(&own.common)
            && other.pbr.is_equal(&own.pbr))
    }
}

/// The conversions between the definitions.
impl XCAFDocVisMaterial {
    /// `ConvertToCommonMaterial`: the common definition, else one converted from the PBR
    /// definition, else an undefined-looking default.
    pub fn convert_to_common_material(&self) -> Result<XCAFDocVisMaterialCommon, Failure> {
        let fields = self.fields.read();
        if fields.common.is_defined {
            return Ok(fields.common.clone());
        }
        if !fields.pbr.is_defined {
            return Ok(XCAFDocVisMaterialCommon::default());
        }
        let pbr = &fields.pbr;
        let mut common = XCAFDocVisMaterialCommon {
            diffuse_texture: pbr.base_color_texture.clone(),
            diffuse_color: *pbr.base_color.rgb(),
            specular_color: Color::from_vec3(pbr.base_color.rgb().rgb_vec3() * pbr.metallic)?,
            transparency: 1.0 - pbr.base_color.alpha(),
            shininess: 1.0 - pbr.roughness,
            ..Default::default()
        };
        if pbr.emissive_texture.is_null() {
            common.emissive_color =
                Color::from_vec3(pbr.emissive_factor.cwise_min(&Vec3::new(1.0, 1.0, 1.0)))?;
        }
        Ok(common)
    }
    /// `ConvertToPbrMaterial`: the PBR definition, else one converted from the common
    /// definition, else a default.
    pub fn convert_to_pbr_material(&self) -> XCAFDocVisMaterialPbr {
        let fields = self.fields.read();
        if fields.pbr.is_defined {
            return fields.pbr.clone();
        }
        if !fields.common.is_defined {
            return XCAFDocVisMaterialPbr::default();
        }
        let common = &fields.common;
        let mut pbr = XCAFDocVisMaterialPbr {
            base_color_texture: common.diffuse_texture.clone(),
            metallic: metallic_of(common),
            roughness: Graphic3dPbrMaterial::roughness_from_specular(
                common.specular_color,
                f64::from(common.shininess),
            ),
            emissive_factor: common.emissive_color.rgb_vec3(),
            ..Default::default()
        };
        *pbr.base_color.change_rgb() = common.diffuse_color;
        pbr.base_color.set_alpha(1.0 - common.transparency);
        pbr
    }
}

/// The metalness of a common definition: from its specular color when opaque, else 0.
pub(super) fn metallic_of(common: &XCAFDocVisMaterialCommon) -> f32 {
    if common.transparency <= short_real_epsilon() {
        Graphic3dPbrMaterial::metallic_from_specular(common.specular_color)
    } else {
        0.0
    }
}

impl TdfAttribute for XCAFDocVisMaterial {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        VIS_MATERIAL_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = downcast::<XCAFDocVisMaterial>(with, "XCAFDoc_VisMaterial::Restore")?;
        let values = rustycad_tklcaf::live(&other, "XCAFDoc_VisMaterial::Restore")?
            .fields
            .read()
            .clone();
        *self.fields.write() = values;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let other = downcast::<XCAFDocVisMaterial>(into, "XCAFDoc_VisMaterial::Paste")?;
        let values = self.fields.read().clone();
        rustycad_tklcaf::live(&other, "XCAFDoc_VisMaterial::Paste")?.edit(|fields| *fields = values)
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFDoc_VisMaterial")?;
        dump_attribute_json_base(self, stream, depth)?;
        field_raw_pointer(
            stream,
            "myRawName.get()",
            self.raw_name.read().object_address(),
        )?;
        let fields = self.fields.read().clone();
        field_dumped(stream, depth, "&myPbrMat", &|s, d| {
            fields.pbr.dump_json(s, d)
        })?;
        field_dumped(stream, depth, "&myCommonMat", &|s, d| {
            fields.common.dump_json(s, d)
        })?;
        field_integer(stream, "myAlphaMode", fields.alpha_mode as i32)?;
        field_real(stream, "myAlphaCutOff", f64::from(fields.alpha_cut_off))?;
        field_integer(stream, "myFaceCulling", fields.face_culling as i32)
    }
}
