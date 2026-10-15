// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_VisMaterial.cxx (FillMaterialAspect and
// FillAspect) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The graphic aspects of a visualization material: its `Graphic3d_MaterialAspect` (with
//! the PBR material and BSDF) and its textures.
use super::vis_material::{VisMaterialFields, metallic_of};
use super::{XCAFDocVisMaterial, XCAFDocVisMaterialCommon, XCAFDocVisMaterialPbr};
use crate::xcaf_prs::XCAFPrsTexture;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsMembers, Graphic3dAspectsRenderMembers, Graphic3dAspectsSource, Graphic3dBsdf,
    Graphic3dMaterialAspect, Graphic3dNameOfMaterial, Graphic3dPbrMaterial, Graphic3dTextureMap,
    Graphic3dTextureSet, Graphic3dTextureUnit,
};
use rustycad_tkservice::image::ImageTextureSource;

/// The common definition as the colors of a user-defined material aspect.
fn common_aspect(common: &XCAFDocVisMaterialCommon) -> Result<Graphic3dMaterialAspect, Failure> {
    let mut aspect = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::UserDefined);
    aspect.set_ambient_color(common.ambient_color);
    aspect.set_diffuse_color(common.diffuse_color);
    aspect.set_specular_color(common.specular_color);
    aspect.set_emissive_color(common.emissive_color);
    aspect.set_transparency(common.transparency)?;
    aspect.set_shininess(common.shininess)?;
    Ok(aspect)
}

/// The PBR material converted from a common definition.
fn common_pbr(common: &XCAFDocVisMaterialCommon) -> Result<Graphic3dPbrMaterial, Failure> {
    let mut pbr = Graphic3dPbrMaterial::default();
    pbr.set_color(common.diffuse_color);
    pbr.set_metallic(metallic_of(common))?;
    pbr.set_roughness(Graphic3dPbrMaterial::roughness_from_specular(
        common.specular_color,
        f64::from(common.shininess),
    ))?;
    pbr.set_emission(common.emissive_color.rgb_vec3())?;
    Ok(pbr)
}

/// The PBR definition as the colors of a user-defined material aspect.
fn pbr_aspect(pbr: &XCAFDocVisMaterialPbr) -> Result<Graphic3dMaterialAspect, Failure> {
    let mut aspect = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::UserDefined);
    aspect.set_diffuse_color(*pbr.base_color.rgb());
    aspect.set_alpha(pbr.base_color.alpha())?;
    aspect.set_specular_color(Color::from_vec3(
        pbr.base_color.rgb().rgb_vec3() * pbr.metallic,
    )?);
    aspect.set_shininess(1.0 - pbr.roughness)?;
    if aspect.shininess() < 0.01 {
        aspect.set_shininess(0.01)?;
    }
    aspect.set_emissive_color(Color::from_vec3(
        pbr.emissive_factor.cwise_min(&Vec3::new(1.0, 1.0, 1.0)),
    )?);
    Ok(aspect)
}

/// The PBR material of a PBR definition.
fn pbr_material(pbr: &XCAFDocVisMaterialPbr) -> Result<Graphic3dPbrMaterial, Failure> {
    let mut material = Graphic3dPbrMaterial::default();
    material.set_color_rgba(&pbr.base_color)?;
    material.set_metallic(pbr.metallic)?;
    material.set_roughness(pbr.roughness)?;
    material.set_emission(pbr.emissive_factor)?;
    material.set_ior(pbr.refraction_index)?;
    Ok(material)
}

/// Sets the PBR material and its metal-roughness BSDF.
fn set_pbr(aspect: &mut Graphic3dMaterialAspect, pbr: Graphic3dPbrMaterial) {
    aspect.set_bsdf(Graphic3dBsdf::create_metallic_roughness(&pbr));
    aspect.set_pbr_material(pbr);
}

impl XCAFDocVisMaterial {
    /// `FillMaterialAspect(theAspect)`: the aspect of the common definition (with a PBR
    /// material converted from it when there is no PBR definition), then the PBR definition.
    pub fn fill_material_aspect(
        &self,
        aspect: &mut Graphic3dMaterialAspect,
    ) -> Result<(), Failure> {
        let fields = self.fields.read().clone();
        if fields.common.is_defined {
            *aspect = common_aspect(&fields.common)?;
            if !fields.pbr.is_defined {
                set_pbr(aspect, common_pbr(&fields.common)?);
            }
        }
        if fields.pbr.is_defined {
            if !fields.common.is_defined {
                *aspect = pbr_aspect(&fields.pbr)?;
            }
            aspect.set_refraction_index(fields.pbr.refraction_index)?;
            set_pbr(aspect, pbr_material(&fields.pbr)?);
        }
        Ok(())
    }
    /// `FillAspect(theAspect)`: the front material, alpha mode, culling and textures of a
    /// defined material (`Standard_NullObject` for a null aspect).
    pub fn fill_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) -> Result<(), Failure> {
        if self.is_empty() {
            return Ok(());
        }
        let mut material = Graphic3dMaterialAspect::default();
        self.fill_material_aspect(&mut material)?;
        let target = rustycad_tklcaf::live(aspect, "XCAFDoc_VisMaterial::FillAspect")?;
        let fields = self.fields.read().clone();
        target.set_front_material(material);
        target.set_alpha_mode(fields.alpha_mode, fields.alpha_cut_off);
        target.set_face_culling(fields.face_culling);
        let textures = texture_units(&fields);
        if textures.is_empty() {
            return Ok(());
        }
        let set = Graphic3dTextureSet::with_size(textures.len() as i32)?;
        for (index, (source, unit)) in textures.into_iter().enumerate() {
            let texture: Handle<dyn Graphic3dTextureMap> =
                Handle::new(XCAFPrsTexture::new(source, unit)).cast();
            set.set_value(index as i32, texture)?;
        }
        target.set_texture_set(Handle::new(set));
        target.set_texture_map_on(true);
        Ok(())
    }
}

/// The textures of the material in the native unit order (color from the PBR definition,
/// else the common one; emission, occlusion, normal, metal-roughness).
fn texture_units(
    fields: &VisMaterialFields,
) -> Vec<(Handle<dyn ImageTextureSource>, Graphic3dTextureUnit)> {
    let pbr = &fields.pbr;
    let color = if pbr.base_color_texture.is_null() {
        fields.common.diffuse_texture.clone()
    } else {
        pbr.base_color_texture.clone()
    };
    [
        (color, Graphic3dTextureUnit::BASE_COLOR),
        (pbr.emissive_texture.clone(), Graphic3dTextureUnit::EMISSIVE),
        (
            pbr.occlusion_texture.clone(),
            Graphic3dTextureUnit::OCCLUSION,
        ),
        (pbr.normal_texture.clone(), Graphic3dTextureUnit::NORMAL),
        (
            pbr.metallic_roughness_texture.clone(),
            Graphic3dTextureUnit::METALLIC_ROUGHNESS,
        ),
    ]
    .into_iter()
    .filter(|(texture, _)| !texture.is_null())
    .collect()
}
