//! The predefined materials of `Graphic3d_MaterialAspect.cxx` (`RawMaterial`, `THE_MATERIALS`).
//!
//! The native constructor is one `switch` over the material; here each case is one
//! `MaterialSpec` row of `MATERIAL_SPECS` (material_specs.rs) and `raw_material` builds the `RawMaterial` from it
//! in the native statement order (BSDF, then the PBR material from the BSDF).
use super::material_specs::{BLACK, BsdfSpec, MATERIAL_SPECS, MaterialSpec};
use super::{
    Graphic3dBsdf, Graphic3dFresnel, Graphic3dNameOfMaterial, Graphic3dPbrMaterial,
    Graphic3dTypeOfMaterial,
};
use rustycad_tkernel::ncollection::{Vec3, Vec4};
use rustycad_tkernel::quantity::Color;
use std::sync::OnceLock;

/// One predefined material (`RawMaterial`).
#[derive(Clone, Debug)]
pub(crate) struct RawMaterial {
    pub(crate) string_name: &'static str,
    pub(crate) bsdf: Graphic3dBsdf,
    pub(crate) pbr_material: Graphic3dPbrMaterial,
    /// Ambient, diffuse, specular and emission colors (`Colors[Graphic3d_TOR_*]`).
    pub(crate) colors: [Color; 4],
    pub(crate) transparency_coef: f32,
    pub(crate) refraction_index: f32,
    pub(crate) shininess: f32,
    /// Coefficients of `Graphic3d_MaterialAspect::SetColor`.
    pub(crate) ambient_coef: f32,
    pub(crate) diffuse_coef: f32,
    pub(crate) material_type: Graphic3dTypeOfMaterial,
}

/// The materials in `Graphic3d_NameOfMaterial` order.
pub(crate) const MATERIAL_NAMES: [(Graphic3dNameOfMaterial, &str); 26] = {
    use Graphic3dNameOfMaterial as N;
    [
        (N::Brass, "Brass"),
        (N::Bronze, "Bronze"),
        (N::Copper, "Copper"),
        (N::Gold, "Gold"),
        (N::Pewter, "Pewter"),
        (N::Plastered, "Plastered"),
        (N::Plastified, "Plastified"),
        (N::Silver, "Silver"),
        (N::Steel, "Steel"),
        (N::Stone, "Stone"),
        (N::ShinyPlastified, "Shiny_plastified"),
        (N::Satin, "Satined"),
        (N::Metalized, "Metalized"),
        (N::Ionized, "Ionized"),
        (N::Chrome, "Chrome"),
        (N::Aluminum, "Aluminium"),
        (N::Obsidian, "Obsidian"),
        (N::Neon, "Neon"),
        (N::Jade, "Jade"),
        (N::Charcoal, "Charcoal"),
        (N::Water, "Water"),
        (N::Glass, "Glass"),
        (N::Diamond, "Diamond"),
        (N::Transparent, "Transparent"),
        (N::Default, "Default"),
        (N::UserDefined, "UserDefined"),
    ]
};

fn vec3(value: [f32; 3]) -> Vec3<f32> {
    Vec3::new(value[0], value[1], value[2])
}

/// `CreateMetallic(Vec3(0.985), fresnel, 0.045)` of the metals.
fn metal(fresnel: Graphic3dFresnel) -> Graphic3dBsdf {
    Graphic3dBsdf::create_metallic(Vec3::new(0.985, 0.985, 0.985), fresnel, 0.045)
}

/// `BSDF.Kd`, `BSDF.Ks`, `FresnelBase` and `Normalize()` over `CreateDiffuse(0)`.
fn layered(kd: [f32; 3], ks: Option<[f32; 4]>, normalize: bool, dielectric: bool) -> Graphic3dBsdf {
    let mut bsdf = Graphic3dBsdf::create_diffuse(vec3(kd));
    if let Some([x, y, z, w]) = ks {
        bsdf.ks = Vec4::new(x, y, z, w);
    }
    if dielectric {
        bsdf.fresnel_base = Graphic3dFresnel::create_dielectric(1.5);
    }
    if normalize {
        bsdf.normalize();
    }
    bsdf
}

/// The BSDF of one case; `index` is the case's refraction index.
fn build_bsdf(spec: BsdfSpec, index: f32) -> Graphic3dBsdf {
    match spec {
        BsdfSpec::Black => Graphic3dBsdf::create_diffuse(Vec3::new(0.0, 0.0, 0.0)),
        BsdfSpec::Layers {
            kd,
            ks,
            normalize,
            dielectric,
        } => layered(kd, ks, normalize, dielectric),
        BsdfSpec::Glow(le) => Graphic3dBsdf {
            le: vec3(le),
            ..layered(BLACK, Some([0.5, 0.5, 0.5, 0.0]), false, true)
        },
        BsdfSpec::MetalSchlick(color) => metal(Graphic3dFresnel::create_schlick(vec3(color))),
        BsdfSpec::MetalConductor(n, k) => metal(Graphic3dFresnel::create_conductor(n, k)),
        BsdfSpec::MetalConductorRgb(n, k) => {
            metal(Graphic3dFresnel::create_conductor_rgb(vec3(n), vec3(k)))
        }
        BsdfSpec::Glass(absorption) => {
            Graphic3dBsdf::create_glass(Vec3::new(1.0, 1.0, 1.0), vec3(absorption), 0.05, index)
        }
        BsdfSpec::Transparent => Graphic3dBsdf {
            kt: Vec3::new(0.9, 0.9, 0.9),
            fresnel_base: Graphic3dFresnel::create_constant(0.0),
            ..Graphic3dBsdf::create_diffuse(Vec3::new(0.1, 0.1, 0.1))
        },
    }
}

/// `Quantity_Color(NCollection_Vec3<float>)`: the linear RGB values as stored. The table
/// values are all in [0, 1]; the unit tests build every material.
fn color(rgb: [f32; 3]) -> Color {
    Color::from_vec3(vec3(rgb)).expect("predefined material colors are in [0, 1]")
}

/// `RawMaterial(theName, theStringName)`: the case values, then the PBR equivalent.
fn raw_material(spec: &MaterialSpec, string_name: &'static str) -> RawMaterial {
    let (refraction_index, transparency_coef) = spec.optics.unwrap_or((1.0, 0.0));
    let (ambient_coef, diffuse_coef) = spec.coefs.unwrap_or((0.25, 1.0));
    let bsdf = build_bsdf(spec.bsdf, refraction_index);
    let mut pbr_material = Graphic3dPbrMaterial::default();
    pbr_material
        .set_bsdf(&bsdf)
        .expect("predefined BSDFs convert to PBR materials");
    let material_type = if spec.physic {
        Graphic3dTypeOfMaterial::Physic
    } else {
        Graphic3dTypeOfMaterial::Aspect
    };
    RawMaterial {
        string_name,
        bsdf,
        pbr_material,
        colors: spec.colors.map(color),
        transparency_coef,
        refraction_index,
        shininess: spec.shininess,
        ambient_coef,
        diffuse_coef,
        material_type,
    }
}

/// `THE_MATERIALS`, built once.
pub(crate) fn the_materials() -> &'static [RawMaterial] {
    static MATERIALS: OnceLock<Vec<RawMaterial>> = OnceLock::new();
    MATERIALS.get_or_init(|| {
        let rows = MATERIAL_SPECS.iter().zip(MATERIAL_NAMES);
        rows.map(|(spec, (_, text))| raw_material(spec, text))
            .collect()
    })
}
