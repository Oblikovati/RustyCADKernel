//! `Graphic3d_PBRMaterial`: the metallic-roughness material of the PBR shading model, and
//! the split-sum environment lookup table it generates.
use super::{Graphic3dBsdf, Graphic3dFresnelModel};
use crate::image::{ImageColorRgf, ImageFormat, ImagePixMap};
use crate::json_dump::{class_begin, field_dumped, field_real};
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};
use rustycad_tkernel::quantity::{Color, ColorRgba};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::program_error;
use std::f64::consts::PI;
use std::io;

/// `Graphic3d_MaterialDefinitionError` with the offending value.
pub fn material_definition_error(message: &str) -> Failure {
    Failure::with_type("Graphic3d_MaterialDefinitionError", message, "")
}

fn check_range(value: f32, lower: f32, upper: f32, what: &str) -> Result<(), Failure> {
    if value < lower || value > upper {
        return Err(material_definition_error(&format!(
            "'{what}' parameter of PBR material must be in range [{lower}, {upper}]; got {value}"
        )));
    }
    Ok(())
}

/// A metallic-roughness material; `Graphic3dPbrMaterial::default().ior() == 1.5`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Graphic3dPbrMaterial {
    color: ColorRgba,
    metallic: f32,
    roughness: f32,
    emission: Vec3<f32>,
    ior: f32,
}
impl Default for Graphic3dPbrMaterial {
    /// Opaque black, dielectric, fully rough, no emission, IOR 1.5.
    fn default() -> Self {
        Self {
            color: ColorRgba::new(0.0, 0.0, 0.0, 1.0).expect("black is in range"),
            metallic: 0.0,
            roughness: 1.0,
            emission: Vec3::new(0.0, 0.0, 0.0),
            ior: 1.5,
        }
    }
}

impl Graphic3dPbrMaterial {
    /// The material equivalent to a BSDF. Native quirk: the native constructor leaves the
    /// members SetBSDF does not set indeterminate; here they keep the default values.
    ///
    /// Example: `Graphic3dPbrMaterial::from_bsdf(&bsdf)?`.
    pub fn from_bsdf(bsdf: &Graphic3dBsdf) -> Result<Self, Failure> {
        let mut material = Self::default();
        material.set_bsdf(bsdf)?;
        Ok(material)
    }
    /// The metallic coefficient; `material.metallic()`.
    pub fn metallic(&self) -> f32 {
        self.metallic
    }
    /// Sets the metallic coefficient in [0, 1]; `material.set_metallic(1.0)?`.
    pub fn set_metallic(&mut self, metallic: f32) -> Result<(), Failure> {
        check_range(metallic, 0.0, 1.0, "metallic")?;
        self.metallic = metallic;
        Ok(())
    }
    /// The real roughness of a normalized roughness; `Graphic3dPbrMaterial::roughness_of(0.5)`.
    pub fn roughness_of(normalized_roughness: f32) -> f32 {
        normalized_roughness * (1.0 - Self::min_roughness()) + Self::min_roughness()
    }
    /// The real roughness; `material.roughness()`.
    pub fn roughness(&self) -> f32 {
        Self::roughness_of(self.roughness)
    }
    /// The normalized roughness in [0, 1]; `material.normalized_roughness()`.
    pub fn normalized_roughness(&self) -> f32 {
        self.roughness
    }
    /// Sets the normalized roughness in [0, 1]; `material.set_roughness(0.5)?`.
    pub fn set_roughness(&mut self, roughness: f32) -> Result<(), Failure> {
        check_range(roughness, 0.0, 1.0, "roughness")?;
        self.roughness = roughness;
        Ok(())
    }
    /// The index of refraction; `material.ior()`.
    pub fn ior(&self) -> f32 {
        self.ior
    }
    /// Sets the index of refraction in [1, 3]; `material.set_ior(1.33)?`.
    pub fn set_ior(&mut self, ior: f32) -> Result<(), Failure> {
        check_range(ior, 1.0, 3.0, "IOR")?;
        self.ior = ior;
        Ok(())
    }
    /// The albedo with alpha; `material.color()`.
    pub fn color(&self) -> ColorRgba {
        self.color
    }
    /// Sets the albedo and the alpha; `material.set_color_rgba(&rgba)?`.
    pub fn set_color_rgba(&mut self, color: &ColorRgba) -> Result<(), Failure> {
        self.color.set_rgb(*color.rgb());
        self.set_alpha(color.alpha())
    }
    /// Sets the albedo, keeping the alpha; `material.set_color(red)`.
    pub fn set_color(&mut self, color: Color) {
        self.color.set_rgb(color);
    }
    /// The alpha; `material.alpha()`.
    pub fn alpha(&self) -> f32 {
        self.color.alpha()
    }
    /// Sets the alpha in [0, 1]; `material.set_alpha(0.5)?`.
    pub fn set_alpha(&mut self, alpha: f32) -> Result<(), Failure> {
        check_range(alpha, 0.0, 1.0, "alpha")?;
        self.color.set_alpha(alpha);
        Ok(())
    }
    /// The emitted light; `material.emission()`.
    pub fn emission(&self) -> Vec3<f32> {
        self.emission
    }
    /// Sets the emitted light (no negative component); `material.set_emission(e)?`.
    pub fn set_emission(&mut self, emission: Vec3<f32>) -> Result<(), Failure> {
        if emission.x < 0.0 || emission.y < 0.0 || emission.z < 0.0 {
            return Err(material_definition_error(&format!(
                "all components of 'emission' parameter of PBR material must be greater than 0; got {emission:?}"
            )));
        }
        self.emission = emission;
        Ok(())
    }
    /// Takes the closest metallic-roughness parameters of a BSDF; the colours of the BSDF
    /// must be in [0, 1] (`Standard_OutOfRange` otherwise, as the colour constructor).
    ///
    /// Example: `material.set_bsdf(&bsdf)?`.
    pub fn set_bsdf(&mut self, bsdf: &Graphic3dBsdf) -> Result<(), Failure> {
        self.set_emission(bsdf.le)?;
        if bsdf.absorption != Vec4::new(0.0, 0.0, 0.0, 0.0) {
            return self.set_absorbing(bsdf);
        }
        if bsdf.fresnel_base.fresnel_type() == Graphic3dFresnelModel::Constant
            && bsdf.kt != Vec3::new(0.0, 0.0, 0.0)
        {
            self.set_ior(1.0)?;
            self.set_roughness(1.0)?;
            self.set_metallic(0.0)?;
            self.set_color(Color::from_vec3(bsdf.kt)?);
            return self.set_alpha(1.0 - (bsdf.kt.x + bsdf.kt.y + bsdf.kt.z) / 3.0);
        }
        self.set_roughness(bsdf.ks.w.sqrt())?;
        self.set_base_layer(bsdf)
    }
    /// The absorbing (glass-like) branch of `SetBSDF`.
    fn set_absorbing(&mut self, bsdf: &Graphic3dBsdf) -> Result<(), Failure> {
        self.set_metallic(0.0)?;
        let a = bsdf.absorption;
        self.set_color(Color::from_vec3(Vec3::new(a.x, a.y, a.z))?);
        if bsdf.fresnel_coat.fresnel_type() == Graphic3dFresnelModel::Dielectric {
            self.set_ior(bsdf.fresnel_coat.serialize().y)?;
            self.set_roughness(0.0)?;
            self.set_alpha(a.w * 4.0)?;
        }
        Ok(())
    }
    /// The diffuse or metallic branch of `SetBSDF`, by the base Fresnel model.
    fn set_base_layer(&mut self, bsdf: &Graphic3dBsdf) -> Result<(), Failure> {
        let base = bsdf.fresnel_base;
        let color = match base.fresnel_type() {
            Graphic3dFresnelModel::Dielectric | Graphic3dFresnelModel::Constant => {
                self.set_ior(1.5)?;
                self.set_color(Color::from_vec3(bsdf.kd)?);
                return self.set_metallic(0.0);
            }
            Graphic3dFresnelModel::Schlick => {
                let data = base.serialize();
                Vec3::new(data.x, data.y, data.z)
            }
            Graphic3dFresnelModel::Conductor => Vec3::new(bsdf.ks.x, bsdf.ks.y, bsdf.ks.z),
        };
        self.set_color(Color::from_vec3(color)?);
        self.set_metallic(1.0)
    }
    /// The roughness of a Phong-like material: 1 - shininess, lowered for weak speculars.
    ///
    /// Example: `Graphic3dPbrMaterial::roughness_from_specular(white, 0.5)`.
    pub fn roughness_from_specular(specular: Color, shininess: f64) -> f32 {
        let mut roughness = 1.0 - shininess;
        let intensity =
            specular.red() * 0.2125 + specular.green() * 0.7154 + specular.blue() * 0.0721;
        if intensity < 0.1 {
            roughness *= 1.0 - intensity;
        }
        roughness as f32
    }
    /// The metallic coefficient of a specular colour (its largest linear channel).
    ///
    /// Example: `Graphic3dPbrMaterial::metallic_from_specular(white) == 1.0`.
    pub fn metallic_from_specular(specular: Color) -> f32 {
        let [r, g, b] = specular.rgb();
        r.max(g).max(b)
    }
    /// The smallest real roughness (0.01); `Graphic3dPbrMaterial::min_roughness()`.
    pub fn min_roughness() -> f32 {
        0.01
    }
    /// `DumpJson`; `material.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Graphic3d_PBRMaterial")?;
        let (color, emission) = (self.color, self.emission);
        field_dumped(stream, depth, "&myColor", &|s, _| color.dump_json(s))?;
        field_real(stream, "myMetallic", f64::from(self.metallic))?;
        field_real(stream, "myRoughness", f64::from(self.roughness))?;
        field_dumped(stream, depth, "&myEmission", &|s, _| emission.dump_json(s))?;
        field_real(stream, "myIOR", f64::from(self.ior))
    }
}

/// The environment lookup table and the importance-sampling helpers.
impl Graphic3dPbrMaterial {
    /// Fills an `Image_Format_RGF` image with the split-sum BRDF integration table (scale
    /// and bias by roughness and view angle); `Standard_ProgramError` for another format.
    ///
    /// Example: `Graphic3dPbrMaterial::generate_env_lut(&lut, 1024)?`.
    pub fn generate_env_lut(lut: &ImagePixMap, nb_integral_samples: u32) -> Result<(), Failure> {
        if lut.format() != ImageFormat::Rgf {
            return Err(program_error(&format!(
                "LUT pix map format for PBR LUT generation must be Image_Format_RGF; got {:?}",
                lut.format()
            )));
        }
        let (size_x, size_y) = (lut.width(), lut.height());
        let mut state = lut.change_data();
        // The loops count in `unsigned int`, the divisors are `size_t` (SizeY() - 1 wraps for
        // an empty dimension, as natively).
        for y in 0..size_y as u32 {
            let roughness = Self::roughness_of(y as f32 / size_y.wrapping_sub(1) as f32);
            for x in 0..size_x as u32 {
                let cos_v = x as f32 / size_x.wrapping_sub(1) as f32;
                let result = lut_sample(cos_v, roughness, nb_integral_samples);
                let row = size_y - 1 - y as usize;
                state.data.set_typed(
                    row,
                    x as usize,
                    ImageColorRgf {
                        v: [result.x, result.y],
                    },
                );
            }
        }
        Ok(())
    }
    /// The importance-sampling cone factor; `Graphic3dPbrMaterial::spec_ibl_map_samples_factor(0.9, 0.5)`.
    pub fn spec_ibl_map_samples_factor(probability: f32, roughness: f32) -> f32 {
        lut_gen_importance_sample_cos_theta(probability, roughness).acos() * 2.0 / (PI as f32)
    }
}

/// One LUT cell: the averaged scale and bias over the Hammersley samples.
fn lut_sample(cos_v: f32, roughness: f32, samples: u32) -> Vec2<f32> {
    let view = lut_gen_view(cos_v);
    let mut result = Vec2::new(0.0f32, 0.0);
    for i in 0..samples {
        let half = lut_gen_importance_sample(lut_gen_hammersley(i, samples), roughness);
        let light = lut_gen_reflect(view, half);
        // `if (z >= 0)` natively: a NaN light (empty LUT dimension) is skipped.
        let is_lit = light.z >= 0.0;
        if !is_lit {
            continue;
        }
        let cos_vh = view.dot(&half);
        let geometry = lut_gen_geometry_factor(light.z, cos_v, roughness);
        let mut intermediate = 1.0 - cos_vh;
        intermediate *= intermediate;
        intermediate *= intermediate;
        intermediate *= 1.0 - cos_vh;
        result.x += geometry * (cos_vh / half.z) * (1.0 - intermediate);
        result.y += geometry * (cos_vh / half.z) * intermediate;
    }
    result / samples as f32
}

/// `lutGenGeometryFactor`: the Smith-Schlick geometry term.
fn lut_gen_geometry_factor(cos_l: f32, cos_v: f32, roughness: f32) -> f32 {
    let k = roughness * roughness * 0.5;
    let mut factor = cos_l;
    factor /= cos_l * (1.0 - k) + k;
    factor /= cos_v * (1.0 - k) + k;
    factor
}

/// `lutGenHammersley`: the i-th point of an n-point Hammersley set.
fn lut_gen_hammersley(number: u32, count: u32) -> Vec2<f32> {
    let mut phi2 = 0.0f32;
    for i in 0..32 {
        if (number >> i) == 0 {
            break;
        }
        // float(1 << (i + 1)): an int shift; 1 << 32 is undefined natively (x86: 1).
        let divisor = 1i32.wrapping_shl(i + 1) as f32;
        phi2 += ((number >> i) & 1) as f32 / divisor;
    }
    Vec2::new(number as f32 / count as f32, phi2)
}

/// `lutGenImportanceSampleCosTheta`: GGX sample of the half-vector elevation.
fn lut_gen_importance_sample_cos_theta(component: f32, roughness: f32) -> f32 {
    let mut quad = roughness * roughness;
    quad *= quad;
    let tmp = 1.0 + (quad - 1.0) * component;
    if tmp != 0.0 {
        ((1.0 - component) / tmp).sqrt()
    } else {
        0.0
    }
}

/// `lutGenImportanceSample`: GGX half vector of a Hammersley point.
fn lut_gen_importance_sample(point: Vec2<f32>, roughness: f32) -> Vec3<f32> {
    let phi = 2.0 * (PI as f32) * point.y;
    let cos_theta = lut_gen_importance_sample_cos_theta(point.x, roughness);
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    Vec3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta)
}

/// `lutGenView`: the view vector of a view-angle cosine.
fn lut_gen_view(cos_v: f32) -> Vec3<f32> {
    Vec3::new(0.0, (1.0 - cos_v * cos_v).sqrt(), cos_v)
}

/// `lutGenReflect`: the reflection of a vector about an axis.
fn lut_gen_reflect(vector: Vec3<f32>, axis: Vec3<f32>) -> Vec3<f32> {
    axis * axis.dot(&vector) * 2.0 - vector
}
