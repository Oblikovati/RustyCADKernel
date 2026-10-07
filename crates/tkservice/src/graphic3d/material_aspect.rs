//! `Graphic3d_MaterialAspect`: the surface material of a presentation (colors, shininess,
//! transparency, BSDF for path tracing and the PBR material), built from the predefined
//! materials or user values. Range checks compare like the native code, so a NaN passes them.
use super::material_table::{MATERIAL_NAMES, RawMaterial, the_materials};
use super::{
    Graphic3dBsdf, Graphic3dNameOfMaterial, Graphic3dPbrMaterial, Graphic3dTypeOfMaterial,
    Graphic3dTypeOfReflection, material_definition_error,
};
use crate::json_dump::{
    class_begin, field_color, field_dumped, field_integer, field_real, field_string,
};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::out_of_range;
use rustycad_tkernel::tcollection::AsciiString;
use std::io;

/// A surface material; `Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold)`.
#[derive(Clone, Debug)]
pub struct Graphic3dMaterialAspect {
    bsdf: Graphic3dBsdf,
    pbr_material: Graphic3dPbrMaterial,
    string_name: AsciiString,
    colors: [Color; 4],
    transparency_coef: f32,
    refraction_index: f32,
    shininess: f32,
    material_type: Graphic3dTypeOfMaterial,
    material_name: Graphic3dNameOfMaterial,
    requested_material_name: Graphic3dNameOfMaterial,
}

impl Default for Graphic3dMaterialAspect {
    /// The `Default` material.
    fn default() -> Self {
        Self::new(Graphic3dNameOfMaterial::Default)
    }
}

/// The predefined-material catalogue (static members).
impl Graphic3dMaterialAspect {
    /// The number of predefined materials (24 = `Graphic3d_NameOfMaterial_DEFAULT`, so the
    /// ranks 1..=24 exclude `Default`); `Graphic3dMaterialAspect::number_of_materials()`.
    pub fn number_of_materials() -> i32 {
        Graphic3dNameOfMaterial::Default as i32
    }
    /// The predefined material of a 1-based rank, or `Standard_OutOfRange`.
    fn raw_of_rank(rank: i32) -> Result<&'static RawMaterial, Failure> {
        if rank < 1 || rank > Self::number_of_materials() {
            let expected = Self::number_of_materials();
            return Err(out_of_range(&format!(
                "BAD index of material; got rank {rank}, expected [1, {expected}]"
            )));
        }
        Ok(&the_materials()[(rank - 1) as usize])
    }
    /// The name of the material of a rank; `material_name_of(1)? == "Brass"`.
    pub fn material_name_of(rank: i32) -> Result<&'static str, Failure> {
        Ok(Self::raw_of_rank(rank)?.string_name)
    }
    /// The type of the material of a rank; `material_type_of(1)? == Physic`.
    pub fn material_type_of(rank: i32) -> Result<Graphic3dTypeOfMaterial, Failure> {
        Ok(Self::raw_of_rank(rank)?.material_type)
    }
    /// The material named `name` (case-insensitive, plus the obsolete aliases), or `None`;
    /// `material_from_name("gold") == Some(Graphic3dNameOfMaterial::Gold)`.
    pub fn material_from_name(name: &str) -> Option<Graphic3dNameOfMaterial> {
        let mut text = AsciiString::from(name);
        text.lower_case();
        text.capitalize();
        // The native loop runs to NumberOfMaterials() inclusive, so "Default" matches too.
        let count = Self::number_of_materials() as usize;
        let found = MATERIAL_NAMES[..=count]
            .iter()
            .find(|(_, string_name)| text == *string_name);
        found
            .map(|(material, _)| *material)
            .or_else(|| Self::material_from_alias(&text))
    }
    /// The obsolete names that `MaterialFromName` still accepts.
    fn material_from_alias(text: &AsciiString) -> Option<Graphic3dNameOfMaterial> {
        use Graphic3dNameOfMaterial as N;
        let aliases = [
            ("Plastic", N::Plastified),
            ("Shiny_plastic", N::ShinyPlastified),
            ("Plaster", N::Plastered),
            ("Satin", N::Satin),
            ("Neon_gnc", N::Ionized),
            ("Neon_phc", N::Neon),
        ];
        aliases
            .iter()
            .find(|(alias, _)| *text == *alias)
            .map(|(_, material)| *material)
    }
    /// `MaterialFromName(const char*)`: the material, or `Default` when nothing matches;
    /// `material_from_name_or_default("x") == Graphic3dNameOfMaterial::Default`.
    pub fn material_from_name_or_default(name: &str) -> Graphic3dNameOfMaterial {
        Self::material_from_name(name).unwrap_or(Graphic3dNameOfMaterial::Default)
    }
}

/// Construction, names and colors.
impl Graphic3dMaterialAspect {
    /// The predefined material `name`; `Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Jade)`.
    pub fn new(name: Graphic3dNameOfMaterial) -> Self {
        let raw = &the_materials()[name as usize];
        Self {
            bsdf: raw.bsdf,
            pbr_material: raw.pbr_material,
            string_name: AsciiString::from(raw.string_name),
            colors: raw.colors,
            transparency_coef: raw.transparency_coef,
            refraction_index: raw.refraction_index,
            shininess: raw.shininess,
            material_type: raw.material_type,
            material_name: name,
            requested_material_name: name,
        }
    }
    /// The material name (`UserDefined` once modified); `material.name()`.
    pub fn name(&self) -> Graphic3dNameOfMaterial {
        self.material_name
    }
    /// The name given at construction; `material.requested_name()`.
    pub fn requested_name(&self) -> Graphic3dNameOfMaterial {
        self.requested_material_name
    }
    /// The material name as text; `material.string_name().as_bytes() == b"Gold"`.
    pub fn string_name(&self) -> &AsciiString {
        &self.string_name
    }
    /// `MaterialName() const`: the name bytes; `material.material_name() == b"Gold"`.
    pub fn material_name(&self) -> &[u8] {
        self.string_name.as_bytes()
    }
    /// Renames the material and marks it user-defined; `material.set_material_name("Mine".into())`.
    pub fn set_material_name(&mut self, name: AsciiString) {
        self.material_name = Graphic3dNameOfMaterial::UserDefined;
        self.string_name = name;
    }
    /// Restores the requested predefined material; `material.reset()`.
    pub fn reset(&mut self) {
        *self = Self::new(self.requested_material_name);
    }
    /// The diffuse color; `material.color()`.
    pub fn color(&self) -> Color {
        self.colors[Graphic3dTypeOfReflection::Diffuse as usize]
    }
    /// The color of one reflection kind; `material.reflection_color(Graphic3dTypeOfReflection::Ambient)`.
    fn reflection_color(&self, kind: Graphic3dTypeOfReflection) -> Color {
        self.colors[kind as usize]
    }
    /// Marks the material user-defined (`setUserMaterial`).
    fn set_user_material(&mut self) {
        if self.material_name != Graphic3dNameOfMaterial::UserDefined {
            self.material_name = Graphic3dNameOfMaterial::UserDefined;
            self.string_name = AsciiString::from("UserDefined");
        }
    }
    /// Sets the ambient and diffuse colors from one color, scaled by the coefficients of the
    /// requested material, and the PBR albedo; ignored for `Aspect` materials.
    /// `Quantity_Color(Vec3<float>)` cannot fail here (coefficients <= 1), but its range check
    /// still reaches the caller. Example: `material.set_color(red)?`.
    pub fn set_color(&mut self, color: Color) -> Result<(), Failure> {
        if self.material_type == Graphic3dTypeOfMaterial::Aspect {
            return Ok(());
        }
        self.pbr_material.set_color(color);
        let source = &the_materials()[self.requested_material_name as usize];
        let ambient = Color::from_vec3(color.rgb_vec3() * source.ambient_coef)?;
        let diffuse = Color::from_vec3(color.rgb_vec3() * source.diffuse_coef)?;
        let [old_ambient, old_diffuse] = [self.colors[0], self.colors[1]];
        if self.material_name != Graphic3dNameOfMaterial::UserDefined
            && (!old_ambient.is_equal(ambient) || !old_diffuse.is_equal(diffuse))
        {
            self.set_user_material();
        }
        self.colors[0] = ambient;
        self.colors[1] = diffuse;
        Ok(())
    }
    /// Sets one reflection color; a changed color of a predefined physical material makes it
    /// user-defined (`SetAmbientColor` .. `SetEmissiveColor`).
    fn set_reflection_color(&mut self, kind: Graphic3dTypeOfReflection, color: Color) {
        let slot = kind as usize;
        if self.material_type == Graphic3dTypeOfMaterial::Physic
            && self.material_name != Graphic3dNameOfMaterial::UserDefined
            && !self.colors[slot].is_equal(color)
        {
            self.set_user_material();
        }
        self.colors[slot] = color;
    }
    /// The ambient color; `material.ambient_color()`.
    pub fn ambient_color(&self) -> Color {
        self.reflection_color(Graphic3dTypeOfReflection::Ambient)
    }
    /// Sets the ambient color; `material.set_ambient_color(gray)`.
    pub fn set_ambient_color(&mut self, color: Color) {
        self.set_reflection_color(Graphic3dTypeOfReflection::Ambient, color);
    }
    /// The diffuse color; `material.diffuse_color()`.
    pub fn diffuse_color(&self) -> Color {
        self.reflection_color(Graphic3dTypeOfReflection::Diffuse)
    }
    /// Sets the diffuse color; `material.set_diffuse_color(gray)`.
    pub fn set_diffuse_color(&mut self, color: Color) {
        self.set_reflection_color(Graphic3dTypeOfReflection::Diffuse, color);
    }
    /// The specular color; `material.specular_color()`.
    pub fn specular_color(&self) -> Color {
        self.reflection_color(Graphic3dTypeOfReflection::Specular)
    }
    /// Sets the specular color; `material.set_specular_color(gray)`.
    pub fn set_specular_color(&mut self, color: Color) {
        self.set_reflection_color(Graphic3dTypeOfReflection::Specular, color);
    }
    /// The emissive color; `material.emissive_color()`.
    pub fn emissive_color(&self) -> Color {
        self.reflection_color(Graphic3dTypeOfReflection::Emission)
    }
    /// Sets the emissive color; `material.set_emissive_color(gray)`.
    pub fn set_emissive_color(&mut self, color: Color) {
        self.set_reflection_color(Graphic3dTypeOfReflection::Emission, color);
    }
    /// Whether a reflection color is not black; `material.reflection_mode(Graphic3dTypeOfReflection::Specular)`.
    pub fn reflection_mode(&self, kind: Graphic3dTypeOfReflection) -> bool {
        !self
            .reflection_color(kind)
            .is_equal(Color::named(NamedColor::BLACK))
    }
    /// Deprecated: sets a non-black reflection color to black; `material.set_reflection_mode_off(kind)`.
    #[deprecated(note = "Deprecated method, specific material component should be zerroed instead")]
    pub fn set_reflection_mode_off(&mut self, kind: Graphic3dTypeOfReflection) {
        if self.reflection_mode(kind) {
            self.set_reflection_color(kind, Color::named(NamedColor::BLACK));
        }
    }
}

/// Coefficients, BSDF, PBR material, type, comparison and dump.
impl Graphic3dMaterialAspect {
    /// The transparency in [0, 1]; `material.transparency()`.
    pub fn transparency(&self) -> f32 {
        self.transparency_coef
    }
    /// The opacity, `1 - transparency`; `material.alpha()`.
    pub fn alpha(&self) -> f32 {
        1.0 - self.transparency_coef
    }
    /// Sets the transparency and the PBR alpha; values outside [0, 1] raise
    /// `Graphic3d_MaterialDefinitionError`. Example: `material.set_transparency(0.5)?`.
    pub fn set_transparency(&mut self, value: f32) -> Result<(), Failure> {
        if value < 0.0 || value > 1.0 {
            return Err(material_definition_error(&format!(
                "Bad value for SetTransparency < 0. or > 1.0; got {value}"
            )));
        }
        self.transparency_coef = value;
        self.pbr_material.set_alpha(1.0 - value)
    }
    /// Sets the opacity; `material.set_alpha(0.25)?`.
    pub fn set_alpha(&mut self, value: f32) -> Result<(), Failure> {
        self.set_transparency(1.0 - value)
    }
    /// The shininess in [0, 1]; `material.shininess()`.
    pub fn shininess(&self) -> f32 {
        self.shininess
    }
    /// Sets the shininess; a change makes the material user-defined. Example: `material.set_shininess(0.5)?`.
    pub fn set_shininess(&mut self, value: f32) -> Result<(), Failure> {
        if value < 0.0 || value > 1.0 {
            return Err(material_definition_error(&format!(
                "Bad value for Shininess < 0. or > 1.0; got {value}"
            )));
        }
        if self.shininess != value {
            self.shininess = value;
            self.set_user_material();
        }
        Ok(())
    }
    /// Scales the shininess by `delta` percent unless the result leaves [0, 1];
    /// `material.increase_shine(10.0)`.
    pub fn increase_shine(&mut self, delta: f32) {
        let shine = self.shininess + self.shininess * delta / 100.0;
        // A NaN result is kept, as natively: only an ordered out-of-range value reverts.
        if !(shine > 1.0 || shine < 0.0) {
            self.shininess = shine;
        }
    }
    /// The refraction index (>= 1); `material.refraction_index()`.
    pub fn refraction_index(&self) -> f32 {
        self.refraction_index
    }
    /// Sets the refraction index; values below 1 raise `Graphic3d_MaterialDefinitionError`.
    /// Example: `material.set_refraction_index(1.5)?`.
    pub fn set_refraction_index(&mut self, value: f32) -> Result<(), Failure> {
        if value < 1.0 {
            return Err(material_definition_error(&format!(
                "Bad value for refraction index < 1.0; got {value}"
            )));
        }
        self.refraction_index = value;
        Ok(())
    }
    /// The path-tracing BSDF; `material.bsdf()`.
    pub fn bsdf(&self) -> &Graphic3dBsdf {
        &self.bsdf
    }
    /// Replaces the BSDF; `material.set_bsdf(bsdf)`.
    pub fn set_bsdf(&mut self, bsdf: Graphic3dBsdf) {
        self.bsdf = bsdf;
    }
    /// The PBR material; `material.pbr_material()`.
    pub fn pbr_material(&self) -> &Graphic3dPbrMaterial {
        &self.pbr_material
    }
    /// Replaces the PBR material; `material.set_pbr_material(pbr)`.
    pub fn set_pbr_material(&mut self, pbr_material: Graphic3dPbrMaterial) {
        self.pbr_material = pbr_material;
    }
    /// The material type; `material.material_type()`.
    pub fn material_type(&self) -> Graphic3dTypeOfMaterial {
        self.material_type
    }
    /// `MaterialType(theType)`: whether the type is `kind`; `material.is_material_type(Physic)`.
    pub fn is_material_type(&self, kind: Graphic3dTypeOfMaterial) -> bool {
        self.material_type == kind
    }
    /// Sets the type. Native quirk: the member is assigned before the comparison, so the
    /// material never becomes user-defined here. Example: `material.set_material_type(Physic)`.
    pub fn set_material_type(&mut self, kind: Graphic3dTypeOfMaterial) {
        self.material_type = kind;
    }
    /// `IsEqual`: coefficients, BSDF, PBR material and colors (not names or type);
    /// `material.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.transparency_coef == other.transparency_coef
            && self.refraction_index == other.refraction_index
            && self.bsdf == other.bsdf
            && self.pbr_material == other.pbr_material
            && self.shininess == other.shininess
            && self.colors == other.colors
    }
    /// `IsDifferent`; `material.is_different(&other)`.
    pub fn is_different(&self, other: &Self) -> bool {
        !self.is_equal(other)
    }
    /// `DumpJson`; `material.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Graphic3d_MaterialAspect")?;
        let (bsdf, pbr) = (self.bsdf, self.pbr_material);
        field_dumped(stream, depth, "&myBSDF", &|s, d| bsdf.dump_json(s, d))?;
        field_dumped(stream, depth, "&myPBRMaterial", &|s, d| pbr.dump_json(s, d))?;
        field_string(stream, "myStringName", self.string_name.as_bytes())?;
        let slots = ["AMBIENT", "DIFFUSE", "SPECULAR", "EMISSION"];
        for (color, slot) in self.colors.iter().zip(slots) {
            field_color(
                stream,
                depth,
                &format!("&myColors[Graphic3d_TOR_{slot}]"),
                *color,
            )?;
        }
        field_real(
            stream,
            "myTransparencyCoef",
            f64::from(self.transparency_coef),
        )?;
        field_real(
            stream,
            "myRefractionIndex",
            f64::from(self.refraction_index),
        )?;
        field_real(stream, "myShininess", f64::from(self.shininess))?;
        field_integer(stream, "myMaterialType", self.material_type as i32)?;
        field_integer(stream, "myMaterialName", self.material_name as i32)?;
        field_integer(
            stream,
            "myRequestedMaterialName",
            self.requested_material_name as i32,
        )
    }
}

impl PartialEq for Graphic3dMaterialAspect {
    /// `operator==` is `IsEqual`.
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
    /// `operator!=` is `IsDifferent`.
    #[allow(clippy::partialeq_ne_impl)]
    fn ne(&self, other: &Self) -> bool {
        self.is_different(other)
    }
}

#[cfg(test)]
#[path = "material_aspect_tests.rs"]
mod tests;
