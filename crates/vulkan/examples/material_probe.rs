//! Exact shader coefficient oracle counterpart; run with `cargo run --example material_probe`.
use martensite_vulkan::{RenderMaterial, RenderMaterialColorSpace as Space};
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkservice::graphic3d::{
    Graphic3dMaterialAspect as Material, Graphic3dNameOfMaterial as Name,
};
const NAMES: [Name; 26] = [
    Name::Brass,
    Name::Bronze,
    Name::Copper,
    Name::Gold,
    Name::Pewter,
    Name::Plastered,
    Name::Plastified,
    Name::Silver,
    Name::Steel,
    Name::Stone,
    Name::ShinyPlastified,
    Name::Satin,
    Name::Metalized,
    Name::Ionized,
    Name::Chrome,
    Name::Aluminum,
    Name::Obsidian,
    Name::Neon,
    Name::Jade,
    Name::Charcoal,
    Name::Water,
    Name::Glass,
    Name::Diamond,
    Name::Transparent,
    Name::Default,
    Name::UserDefined,
];
fn color([r, g, b]: [f64; 3]) -> Color {
    Color::new(r, g, b, ColorSpace::Rgb).unwrap()
}
fn emit(key: &str, material: &RenderMaterial) {
    let bits: Vec<u32> = material
        .packed_common()
        .into_iter()
        .chain(material.packed_pbr())
        .flatten()
        .map(f32::to_bits)
        .collect();
    println!("{{\"case\":{key:?},\"bits\":{bits:?}}}");
}
fn initialized(rank: usize, linear: bool, alias: bool, modified: bool) -> RenderMaterial {
    let mut front = Material::new(NAMES[rank]);
    let back = Material::new(NAMES[25 - rank]);
    if modified {
        front.set_alpha(0.375).unwrap();
        front.set_shininess(0.375).unwrap();
        front.set_color(color([0.125, 0.5, 0.875])).unwrap();
    }
    let mut packed = RenderMaterial::default();
    packed.pbr[0].params = [7., 8., 9., 10.];
    packed.pbr[1].params = [11., 12., 13., 14.];
    let space = if linear {
        Space::Linear
    } else {
        Space::GammaEncoded
    };
    packed.init(
        space,
        &front,
        color([0.25, 0.5, 0.75]),
        if alias { &front } else { &back },
        color([0.875, 0.125, 0.5]),
    );
    packed
}
fn variants(rank: usize, linear: bool, alias: bool) {
    for modified in [false, true] {
        let key = format!("{rank}-{linear}-{alias}-{modified}");
        let mut material = initialized(rank, linear, alias, modified);
        emit(&key, &material);
        material.set_color([0.75, 0.25, 0.125]);
        emit(&format!("{key}-highlight"), &material);
    }
}
fn main() {
    emit("default", &RenderMaterial::default());
    for rank in 0..NAMES.len() {
        for linear in [false, true] {
            variants(rank, linear, false);
            variants(rank, linear, true);
        }
    }
}
