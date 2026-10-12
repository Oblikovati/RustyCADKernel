//! Named fixture policies pair live Graphic3d commands with native sampler/alpha captures.
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkservice::graphic3d::*;
pub const SURFACE_TEXTURE_MODES: &[&str] = &[
    "default",
    "projective",
    "perspective",
    "clipped",
    "back",
    "front-vertex",
    "back-vertex",
    "nearest",
    "bilinear",
    "clamp",
    "transformed",
    "bottom-up",
    "linear",
    "mask",
    "mask-exact",
    "mask-blend",
    "blend",
    "opaque-alpha",
    "auto-alpha",
    "phong",
    "gouraud",
    "facet",
    "no-modulate",
    "generated-mips",
    "no-mips",
    "anisotropy-fast",
    "anisotropy-middle",
    "anisotropy-quality",
    "missing-file",
    "missing-file-phong",
    "missing-file-gouraud",
    "missing-file-facet",
    "missing-file-no-modulate",
    "missing-file-mask",
    "missing-file-blend",
    "bc1-rgb",
    "bc1-rgba",
    "bc2",
    "bc3",
    "bc1-minified",
    "bc1-base-level",
    "bc1-levels-updated",
    "bc1-filter-levels-updated",
    "image-stale",
    "image-updated",
    "image-null-updated",
    "image-empty-updated",
    "image-repaired",
    "sampler-updated",
];
/// Apply live public texture/aspect parameters; `configure_texture_mode(&texture,&aspects,"clamp")`.
pub fn configure_texture_mode(
    texture: &Graphic3dTexture2D,
    aspects: &Graphic3dAspects,
    mode: &str,
) {
    assert!(
        SURFACE_TEXTURE_MODES.contains(&mode) || mode.starts_with("format-"),
        "received mode={mode:?}; expected standard2D fixture mode"
    );
    let policy = mode.strip_prefix("missing-file-").unwrap_or(mode);
    configure_texture_parameters(&texture.get_params(), policy);
    texture.set_mipmaps_generation(policy != "no-mips");
    texture.set_color_map(policy != "linear");
    configure_texture_alpha(aspects, policy);
    aspects.set_use_vertex_color_for_back_faces(policy != "back-vertex");
    aspects.set_shading_model(texture_fixture_shading(policy));
}
fn texture_fixture_shading(mode: &str) -> Graphic3dTypeOfShadingModel {
    match mode {
        "phong" | "no-modulate" => Graphic3dTypeOfShadingModel::Phong,
        "gouraud" => Graphic3dTypeOfShadingModel::Gouraud,
        "facet" => Graphic3dTypeOfShadingModel::PhongFacet,
        _ => Graphic3dTypeOfShadingModel::Unlit,
    }
}
fn configure_texture_parameters(parameters: &Graphic3dTextureParams, mode: &str) {
    let filter = match mode {
        "nearest" => Graphic3dTypeOfTextureFilter::Nearest,
        "bilinear" => Graphic3dTypeOfTextureFilter::Bilinear,
        _ => Graphic3dTypeOfTextureFilter::Trilinear,
    };
    parameters.set_filter(filter);
    parameters.set_repeat(mode != "clamp");
    parameters.set_modulate(mode != "no-modulate");
    configure_texture_transform(parameters, mode);
    configure_texture_anisotropy(parameters, mode);
    if mode == "bc1-base-level" {
        parameters.set_levels_range(1, 1000);
    }
}
fn configure_texture_transform(parameters: &Graphic3dTextureParams, mode: &str) {
    if matches!(mode, "generated-mips" | "bc1-minified") {
        parameters.set_scale(Vec2::new(128., 128.));
    }
    if mode == "transformed" {
        parameters.set_rotation(30.);
        parameters.set_scale(Vec2::new(2., 3.));
        parameters.set_translation(Vec2::new(0.125, 0.25));
    }
}
fn configure_texture_anisotropy(parameters: &Graphic3dTextureParams, mode: &str) {
    let level = match mode {
        "anisotropy-fast" => Graphic3dLevelOfTextureAnisotropy::Fast,
        "anisotropy-middle" => Graphic3dLevelOfTextureAnisotropy::Middle,
        "anisotropy-quality" => Graphic3dLevelOfTextureAnisotropy::Quality,
        _ => Graphic3dLevelOfTextureAnisotropy::Off,
    };
    parameters.set_aniso_filter(level);
}
fn configure_texture_alpha(aspects: &Graphic3dAspects, mode: &str) {
    let alpha = match mode {
        "mask" | "mask-exact" => Graphic3dAlphaMode::Mask,
        "mask-blend" => Graphic3dAlphaMode::MaskBlend,
        "blend" => Graphic3dAlphaMode::Blend,
        "opaque-alpha" => Graphic3dAlphaMode::Opaque,
        _ => Graphic3dAlphaMode::BlendAuto,
    };
    aspects.set_alpha_mode(alpha, 0.5);
}
pub fn texture_mode_pixels(mode: &str) -> Vec<u8> {
    let mut bytes = vec![
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
    ];
    if matches!(
        mode,
        "mask" | "mask-blend" | "blend" | "opaque-alpha" | "auto-alpha"
    ) {
        for (index, alpha) in [64, 128, 192, 255].into_iter().enumerate() {
            bytes[index * 4 + 3] = alpha;
        }
    }
    bytes
}
