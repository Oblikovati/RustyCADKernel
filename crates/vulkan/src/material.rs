//! Martensite shader material packing, translated from OpenGl_Workspace.cxx at
//! OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871. These records prepare the
//! lighting path; initialization itself requires no graphics context.
use crate::{RenderMaterialCommon, RenderMaterialPbr};
use rustycad_tkernel::quantity::{Color, ColorRgba};
use rustycad_tkservice::graphic3d::{Graphic3dMaterialAspect, Graphic3dTypeOfMaterial};

/// Color policy of the target framebuffer; `RenderMaterialColorSpace::Linear`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderMaterialColorSpace {
    /// An sRGB attachment encodes the linear shader result on write.
    Linear,
    /// Legacy non-sRGB attachments require gamma-encoded common coefficients.
    GammaEncoded,
}
impl RenderMaterialColorSpace {
    fn convert(self, color: [f32; 4]) -> [f32; 4] {
        match self {
            Self::Linear => color,
            Self::GammaEncoded => ColorRgba::linear_to_srgb_vec4(color),
        }
    }
}
/// Front/back material snapshots matching the native shader records; `RenderMaterial::default()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderMaterial {
    /// Common coefficients for front and back faces.
    pub common: [RenderMaterialCommon; 2],
    /// PBR coefficients for front and back faces.
    pub pbr: [RenderMaterialPbr; 2],
}
impl RenderMaterial {
    /// Initialize both faces from Graphic3d materials; `material.init(space, &front, red, &back, blue)`.
    /// Identical source references copy the front record, including its interior color.
    pub fn init(
        &mut self,
        space: RenderMaterialColorSpace,
        front: &Graphic3dMaterialAspect,
        front_color: Color,
        back: &Graphic3dMaterialAspect,
        back_color: Color,
    ) {
        self.init_face(space, front, front_color, 0);
        if std::ptr::eq(front, back) {
            self.common[1] = self.common[0];
            self.pbr[1] = self.pbr[0];
            return;
        }
        self.init_face(space, back, back_color, 1);
    }
    fn init_face(
        &mut self,
        space: RenderMaterialColorSpace,
        source: &Graphic3dMaterialAspect,
        interior: Color,
        index: usize,
    ) {
        self.common[index] = common_coefficients(space, source, interior);
        let pbr = source.pbr_material();
        self.pbr[index].params[1] = pbr.normalized_roughness();
        self.pbr[index].params[2] = pbr.metallic();
        self.pbr[index].emission_ior = with_fourth(pbr.emission().data(), pbr.ior());
        self.pbr[index].base_color = if source.material_type() == Graphic3dTypeOfMaterial::Aspect {
            with_fourth(interior.rgb(), source.alpha())
        } else {
            with_fourth(pbr.color().rgb().rgb(), pbr.color().alpha())
        };
    }
    /// Override both faces for highlight, preserving gloss/emission/alpha; `material.set_color([1.,0.,0.])`.
    pub fn set_color(&mut self, color: [f32; 3]) {
        for common in &mut self.common {
            common.set_color(color);
        }
        for pbr in &mut self.pbr {
            pbr.set_color(color);
        }
    }
    /// Serialize front then back Phong records; `material.packed_common()`.
    pub fn packed_common(&self) -> [[f32; 4]; 8] {
        let [front, back] = self.common.map(|face| face.packed());
        [
            front[0], front[1], front[2], front[3], back[0], back[1], back[2], back[3],
        ]
    }
    /// Serialize front then back PBR records; `material.packed_pbr()`.
    pub fn packed_pbr(&self) -> [[f32; 4]; 6] {
        let [front, back] = self.pbr.map(|face| face.packed());
        [front[0], front[1], front[2], back[0], back[1], back[2]]
    }
    /// Number of vec4 records in the Phong block; `RenderMaterial::nb_of_vec4_common()`.
    pub const fn nb_of_vec4_common() -> usize {
        8
    }
    /// Number of vec4 records in the PBR block; `RenderMaterial::nb_of_vec4_pbr()`.
    pub const fn nb_of_vec4_pbr() -> usize {
        6
    }
    /// Compare exact bit patterns, including signed zeros and NaNs; `material.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        let left = self.packed_common().into_iter().chain(self.packed_pbr());
        let right = other.packed_common().into_iter().chain(other.packed_pbr());
        left.zip(right)
            .all(|(a, b)| a.map(f32::to_bits) == b.map(f32::to_bits))
    }
}
impl PartialEq for RenderMaterial {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
impl Eq for RenderMaterial {}

fn common_coefficients(
    space: RenderMaterialColorSpace,
    source: &Graphic3dMaterialAspect,
    interior: Color,
) -> RenderMaterialCommon {
    let tint = if source.material_type() == Graphic3dTypeOfMaterial::Aspect {
        Some(interior.rgb())
    } else {
        None
    };
    RenderMaterialCommon {
        diffuse: space.convert(tinted(source.diffuse_color(), tint, source.alpha())),
        emission: space.convert(tinted(source.emissive_color(), tint, 1.)),
        ambient: space.convert(tinted(source.ambient_color(), tint, 1.)),
        specular_shininess: space.convert(with_fourth(
            source.specular_color().rgb(),
            128. * source.shininess(),
        )),
    }
}
fn tinted(color: Color, tint: Option<[f32; 3]>, fourth: f32) -> [f32; 4] {
    let rgb = color.rgb();
    with_fourth(
        tint.map_or(rgb, |factor| {
            std::array::from_fn(|index| rgb[index] * factor[index])
        }),
        fourth,
    )
}
fn with_fourth([r, g, b]: [f32; 3], fourth: f32) -> [f32; 4] {
    [r, g, b, fourth]
}
