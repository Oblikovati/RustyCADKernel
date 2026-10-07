//! Graphic3d_MaterialAspect, the twin of tools/oracle/tkservice_probe_material.cxx.
use super::common::{Reader, end, flag, int, real, text};
use super::pbr::{print_bsdf, vec3, vec4};
use super::tried;
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkservice::graphic3d::{
    Graphic3dMaterialAspect, Graphic3dNameOfMaterial, Graphic3dPbrMaterial,
    Graphic3dTypeOfReflection,
};

/// The material of a `Graphic3d_NameOfMaterial` value from the corpus (always in range).
pub fn material_name(value: i32) -> Graphic3dNameOfMaterial {
    const NAMES: [Graphic3dNameOfMaterial; 26] = {
        use Graphic3dNameOfMaterial as N;
        [
            N::Brass,
            N::Bronze,
            N::Copper,
            N::Gold,
            N::Pewter,
            N::Plastered,
            N::Plastified,
            N::Silver,
            N::Steel,
            N::Stone,
            N::ShinyPlastified,
            N::Satin,
            N::Metalized,
            N::Ionized,
            N::Chrome,
            N::Aluminum,
            N::Obsidian,
            N::Neon,
            N::Jade,
            N::Charcoal,
            N::Water,
            N::Glass,
            N::Diamond,
            N::Transparent,
            N::Default,
            N::UserDefined,
        ]
    };
    NAMES[value as usize]
}
fn color(value: Color) {
    value.values(ColorSpace::Rgb).into_iter().for_each(real);
}
fn colors(m: &Graphic3dMaterialAspect) {
    [
        m.ambient_color(),
        m.diffuse_color(),
        m.specular_color(),
        m.emissive_color(),
    ]
    .into_iter()
    .for_each(color);
}
fn pbr(p: &Graphic3dPbrMaterial) {
    vec4(p.color().to_vec4());
    [p.metallic(), p.normalized_roughness(), p.ior()]
        .into_iter()
        .for_each(|c| real(f64::from(c)));
    vec3(p.emission());
}

/// `sv_material Name`.
pub fn material_case(values: &[f64]) {
    let m = Graphic3dMaterialAspect::new(material_name(Reader::new(values).integer()));
    colors(&m);
    [m.transparency(), m.refraction_index(), m.shininess()]
        .into_iter()
        .for_each(|c| real(f64::from(c)));
    int(m.material_type() as i32);
    int(m.name() as i32);
    text(m.string_name().as_bytes());
    print_bsdf(m.bsdf());
    pbr(m.pbr_material());
    let mut dump = DumpStream::default();
    m.dump_json(&mut dump, -1).expect("in-memory dump");
    text(dump.bytes());
    end();
}

/// `sv_material_rank Rank`.
pub fn rank_case(values: &[f64]) {
    let rank = Reader::new(values).integer();
    match Graphic3dMaterialAspect::material_name_of(rank) {
        Ok(name) => text(name.as_bytes()),
        Err(error) => print!("{} ", error.exception_type()),
    }
    match Graphic3dMaterialAspect::material_type_of(rank) {
        Ok(kind) => int(kind as i32),
        Err(error) => print!("{} ", error.exception_type()),
    }
    end();
}

/// `sv_material_edit Name R G B Delta Shine Transparency Index`.
pub fn edit_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut m = Graphic3dMaterialAspect::new(material_name(r.integer()));
    let rgb = [r.real(), r.real(), r.real()];
    let [delta, shine, transparency, index] = [0; 4].map(|_| r.real() as f32);
    let c =
        Color::new(rgb[0], rgb[1], rgb[2], ColorSpace::Rgb).expect("corpus colors are in range");
    tried(m.set_color(c));
    int(m.name() as i32);
    colors(&m);
    pbr(m.pbr_material());
    m.increase_shine(delta);
    real(f64::from(m.shininess()));
    edit_coefficients(&mut m, shine, transparency, index);
    edit_reflection(&mut m, c);
    end();
}

fn edit_coefficients(m: &mut Graphic3dMaterialAspect, shine: f32, transparency: f32, index: f32) {
    tried(m.set_shininess(shine));
    real(f64::from(m.shininess()));
    int(m.name() as i32);
    tried(m.set_transparency(transparency));
    [m.transparency(), m.alpha(), m.pbr_material().alpha()]
        .into_iter()
        .for_each(|c| real(f64::from(c)));
    tried(m.set_refraction_index(index));
    real(f64::from(m.refraction_index()));
}

fn edit_reflection(m: &mut Graphic3dMaterialAspect, c: Color) {
    use Graphic3dTypeOfReflection as K;
    let mut fresh = Graphic3dMaterialAspect::new(m.requested_name());
    fresh.set_specular_color(c);
    int(fresh.name() as i32);
    text(fresh.string_name().as_bytes());
    [K::Ambient, K::Diffuse, K::Specular, K::Emission]
        .into_iter()
        .for_each(|kind| flag(fresh.reflection_mode(kind)));
    flag(m.is_equal(&fresh));
    m.reset();
    flag(m.is_equal(&Graphic3dMaterialAspect::new(m.requested_name())));
}
