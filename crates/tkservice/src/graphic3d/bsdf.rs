//! `Graphic3d_Fresnel` and `Graphic3d_BSDF`: the physically based (path-tracing) material
//! description.
use super::Graphic3dPbrMaterial;
use crate::json_dump::{class_begin, field_dumped, field_integer};
use rustycad_tkernel::ncollection::{Vec3, Vec4};
use rustycad_tkernel::standard::dump::DumpStream;
use std::io;

/// The Fresnel models (`Graphic3d_FresnelModel`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dFresnelModel {
    /// `Graphic3d_FM_SCHLICK`.
    Schlick = 0,
    /// `Graphic3d_FM_CONSTANT`.
    Constant = 1,
    /// `Graphic3d_FM_CONDUCTOR`.
    Conductor = 2,
    /// `Graphic3d_FM_DIELECTRIC`.
    Dielectric = 3,
}

/// A Fresnel reflectance model and its data (`Graphic3d_Fresnel`).
///
/// Example: `Graphic3dFresnel::create_dielectric(1.5).fresnel_type()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Graphic3dFresnel {
    fresnel_type: Graphic3dFresnelModel,
    fresnel_data: Vec3<f32>,
}
impl Default for Graphic3dFresnel {
    /// A constant model with data (0, 1, 0).
    fn default() -> Self {
        Self::new(Graphic3dFresnelModel::Constant, Vec3::new(0.0, 1.0, 0.0))
    }
}

/// `fresnelNormal`: the normal-incidence reflectance of a conductor.
fn fresnel_normal(n: f32, k: f32) -> f32 {
    ((n - 1.0) * (n - 1.0) + k * k) / ((n + 1.0) * (n + 1.0) + k * k)
}

impl Graphic3dFresnel {
    /// The protected constructor: a model and its raw data; `Graphic3dFresnel::new(model, data)`.
    pub fn new(fresnel_type: Graphic3dFresnelModel, fresnel_data: Vec3<f32>) -> Self {
        Self {
            fresnel_type,
            fresnel_data,
        }
    }
    /// Schlick approximation with a specular color; `Graphic3dFresnel::create_schlick(c)`.
    pub fn create_schlick(specular_color: Vec3<f32>) -> Self {
        Self::new(Graphic3dFresnelModel::Schlick, specular_color)
    }
    /// Constant reflection; `Graphic3dFresnel::create_constant(0.5)`.
    pub fn create_constant(reflection: f32) -> Self {
        Self::new(
            Graphic3dFresnelModel::Constant,
            Vec3::new(0.0, 1.0, reflection),
        )
    }
    /// Dielectric of a refraction index; `Graphic3dFresnel::create_dielectric(1.5)`.
    pub fn create_dielectric(refraction_index: f32) -> Self {
        Self::new(
            Graphic3dFresnelModel::Dielectric,
            Vec3::new(0.0, refraction_index, 0.0),
        )
    }
    /// Conductor of scalar indices; `Graphic3dFresnel::create_conductor(0.2, 3.0)`.
    pub fn create_conductor(refraction_index: f32, absorption_index: f32) -> Self {
        Self::new(
            Graphic3dFresnelModel::Conductor,
            Vec3::new(0.0, refraction_index, absorption_index),
        )
    }
    /// Conductor of per-channel indices, as a Schlick model of the normal reflectance.
    ///
    /// Example: `Graphic3dFresnel::create_conductor_rgb(n, k)`.
    pub fn create_conductor_rgb(refraction_index: Vec3<f32>, absorption_index: Vec3<f32>) -> Self {
        let normal = Vec3::new(
            fresnel_normal(refraction_index.x, absorption_index.x),
            fresnel_normal(refraction_index.y, absorption_index.y),
            fresnel_normal(refraction_index.z, absorption_index.z),
        );
        Self::create_schlick(normal)
    }
    /// The data as sent to shaders: the data with w = 0, x = -model for non-Schlick models.
    ///
    /// Example: `fresnel.serialize()`.
    pub fn serialize(&self) -> Vec4<f32> {
        let mut data = Vec4::new(
            self.fresnel_data.x,
            self.fresnel_data.y,
            self.fresnel_data.z,
            0.0,
        );
        if self.fresnel_type != Graphic3dFresnelModel::Schlick {
            data.x = -(self.fresnel_type as i32 as f32);
        }
        data
    }
    /// The model; `fresnel.fresnel_type()`.
    pub fn fresnel_type(&self) -> Graphic3dFresnelModel {
        self.fresnel_type
    }
    /// `DumpJson`: class name, model and data; `fresnel.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Graphic3d_Fresnel")?;
        field_integer(stream, "myFresnelType", self.fresnel_type as i32)?;
        let data = self.fresnel_data;
        field_dumped(stream, depth, "&myFresnelData", &|s, _| data.dump_json(s))
    }
}

/// A bidirectional scattering distribution (`Graphic3d_BSDF`): coat, diffuse, specular,
/// transmission, emission, absorption and two Fresnel layers.
///
/// Example: `Graphic3dBsdf::create_diffuse(Vec3::new(0.5, 0.5, 0.5))`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Graphic3dBsdf {
    /// `Kc`: coat weight (rgb) and roughness (w).
    pub kc: Vec4<f32>,
    /// `Kd`: diffuse weight.
    pub kd: Vec3<f32>,
    /// `Ks`: specular weight (rgb) and roughness (w).
    pub ks: Vec4<f32>,
    /// `Kt`: transmission weight.
    pub kt: Vec3<f32>,
    /// `Le`: emitted radiance.
    pub le: Vec3<f32>,
    /// `Absorption`: absorption color (rgb) and coefficient (w).
    pub absorption: Vec4<f32>,
    /// `FresnelCoat`.
    pub fresnel_coat: Graphic3dFresnel,
    /// `FresnelBase`.
    pub fresnel_base: Graphic3dFresnel,
}
impl Default for Graphic3dBsdf {
    /// Black, specular roughness 1, no coat reflection, full base reflection.
    fn default() -> Self {
        Self {
            kc: Vec4::new(0.0, 0.0, 0.0, 0.0),
            kd: Vec3::new(0.0, 0.0, 0.0),
            ks: Vec4::new(0.0, 0.0, 0.0, 1.0),
            kt: Vec3::new(0.0, 0.0, 0.0),
            le: Vec3::new(0.0, 0.0, 0.0),
            absorption: Vec4::new(0.0, 0.0, 0.0, 0.0),
            fresnel_coat: Graphic3dFresnel::create_constant(0.0),
            fresnel_base: Graphic3dFresnel::create_constant(1.0),
        }
    }
}

/// A transmissive BSDF (the coat takes the transmission weight).
fn transmissive(
    weight: Vec3<f32>,
    absorption_color: Vec3<f32>,
    absorption_coeff: f32,
    coat: Graphic3dFresnel,
) -> Graphic3dBsdf {
    let mut bsdf = Graphic3dBsdf {
        fresnel_coat: coat,
        kt: weight,
        ..Default::default()
    };
    (bsdf.kc.x, bsdf.kc.y, bsdf.kc.z) = (weight.x, weight.y, weight.z);
    bsdf.absorption = Vec4::new(
        absorption_color.x,
        absorption_color.y,
        absorption_color.z,
        absorption_coeff,
    );
    bsdf
}

impl Graphic3dBsdf {
    /// A Lambertian BSDF; `Graphic3dBsdf::create_diffuse(weight)`.
    pub fn create_diffuse(weight: Vec3<f32>) -> Self {
        Self {
            kd: weight,
            ..Default::default()
        }
    }
    /// A glossy metal; `Graphic3dBsdf::create_metallic(weight, fresnel, 0.1)`.
    pub fn create_metallic(weight: Vec3<f32>, fresnel: Graphic3dFresnel, roughness: f32) -> Self {
        let ks = Vec4::new(weight.x, weight.y, weight.z, roughness);
        Self {
            fresnel_base: fresnel,
            ks,
            ..Default::default()
        }
    }
    /// A thin transparent layer; `Graphic3dBsdf::create_transparent(weight, color, 0.1)`.
    pub fn create_transparent(
        weight: Vec3<f32>,
        absorption_color: Vec3<f32>,
        absorption_coeff: f32,
    ) -> Self {
        transmissive(
            weight,
            absorption_color,
            absorption_coeff,
            Graphic3dFresnel::create_constant(0.0),
        )
    }
    /// A glass; `Graphic3dBsdf::create_glass(weight, color, 0.1, 1.5)`.
    pub fn create_glass(
        weight: Vec3<f32>,
        absorption_color: Vec3<f32>,
        absorption_coeff: f32,
        refraction_index: f32,
    ) -> Self {
        transmissive(
            weight,
            absorption_color,
            absorption_coeff,
            Graphic3dFresnel::create_dielectric(refraction_index),
        )
    }
    /// The BSDF of a metallic-roughness PBR material; `Graphic3dBsdf::create_metallic_roughness(&pbr)`.
    pub fn create_metallic_roughness(pbr: &Graphic3dPbrMaterial) -> Self {
        let rgb = pbr.color().rgb().rgb();
        let color = Vec3::new(rgb[0], rgb[1], rgb[2]);
        let diffuse = color * pbr.alpha();
        let roughness2 = pbr.normalized_roughness() * pbr.normalized_roughness();
        let mut bsdf = Self {
            le: pbr.emission(),
            ..Default::default()
        };
        if pbr.ior() > 1.0 && pbr.alpha() < 1.0 && pbr.metallic() <= f32::EPSILON {
            bsdf = transmissive(
                Vec3::new(1.0, 1.0, 1.0),
                color,
                pbr.alpha() * 0.25,
                Graphic3dFresnel::create_dielectric(pbr.ior()),
            );
            bsdf.le = pbr.emission();
            return bsdf;
        }
        bsdf.fresnel_base = Graphic3dFresnel::create_schlick(diffuse * pbr.metallic());
        bsdf.ks = Vec4::new(pbr.alpha(), pbr.alpha(), pbr.alpha(), roughness2);
        let transmission = 1.0 - pbr.alpha();
        bsdf.kt = Vec3::new(transmission, transmission, transmission);
        bsdf.kd = diffuse * (1.0 - pbr.metallic());
        bsdf
    }
    /// Scales the diffuse, specular and transmission weights so no channel exceeds 1.
    ///
    /// Example: `bsdf.normalize()`.
    pub fn normalize(&mut self) {
        let channels = [
            (self.kd.x, self.ks.x, self.kt.x),
            (self.kd.y, self.ks.y, self.kt.y),
            (self.kd.z, self.ks.z, self.kt.z),
        ];
        let max = channels
            .iter()
            .fold(0.0f32, |max, (d, s, t)| max.max(d + s + t));
        if max <= 1.0 {
            return;
        }
        (self.kd.x, self.kd.y, self.kd.z) = (self.kd.x / max, self.kd.y / max, self.kd.z / max);
        (self.ks.x, self.ks.y, self.ks.z) = (self.ks.x / max, self.ks.y / max, self.ks.z / max);
        (self.kt.x, self.kt.y, self.kt.z) = (self.kt.x / max, self.kt.y / max, self.kt.z / max);
    }
    /// `DumpJson`: every layer; `bsdf.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Graphic3d_BSDF")?;
        let (kc, kd, ks, kt, le, absorption) =
            (self.kc, self.kd, self.ks, self.kt, self.le, self.absorption);
        field_dumped(stream, depth, "&Kc", &|s, _| kc.dump_json(s))?;
        field_dumped(stream, depth, "&Kd", &|s, _| kd.dump_json(s))?;
        field_dumped(stream, depth, "&Ks", &|s, _| ks.dump_json(s))?;
        field_dumped(stream, depth, "&Kt", &|s, _| kt.dump_json(s))?;
        field_dumped(stream, depth, "&Le", &|s, _| le.dump_json(s))?;
        field_dumped(stream, depth, "&Absorption", &|s, _| {
            absorption.dump_json(s)
        })?;
        let (coat, base) = (self.fresnel_coat, self.fresnel_base);
        field_dumped(stream, depth, "&FresnelCoat", &|s, d| coat.dump_json(s, d))?;
        field_dumped(stream, depth, "&FresnelBase", &|s, d| base.dump_json(s, d))
    }
}
