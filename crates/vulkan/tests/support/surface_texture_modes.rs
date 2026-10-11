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
    "bc1-rgb",
    "bc1-rgba",
    "bc2",
    "bc3",
    "bc1-minified",
    "bc1-base-level",
    "image-stale",
    "image-updated",
    "sampler-updated",
];
pub fn configure_texture_mode(
    texture: &Graphic3dTexture2D,
    aspects: &Graphic3dAspects,
    mode: &str,
) {
    assert!(
        (SURFACE_TEXTURE_MODES.contains(&mode) || mode.starts_with("format-")),
        "received mode={mode:?}; expected standard2D fixture mode"
    );
    configure_texture_parameters(&texture.get_params(), mode);
    if mode == "linear" {
        texture.set_color_map(false);
    }
    configure_texture_alpha(aspects, mode);
    if mode == "back-vertex" {
        aspects.set_use_vertex_color_for_back_faces(false);
    }
    let model = match mode {
        "phong" | "no-modulate" => Graphic3dTypeOfShadingModel::Phong,
        "gouraud" => Graphic3dTypeOfShadingModel::Gouraud,
        "facet" => Graphic3dTypeOfShadingModel::PhongFacet,
        _ => Graphic3dTypeOfShadingModel::Unlit,
    };
    aspects.set_shading_model(model);
}
fn configure_texture_parameters(parameters: &Graphic3dTextureParams, mode: &str) {
    match mode {
        "nearest" => parameters.set_filter(Graphic3dTypeOfTextureFilter::Nearest),
        "bilinear" => parameters.set_filter(Graphic3dTypeOfTextureFilter::Bilinear),
        "clamp" => parameters.set_repeat(false),
        "no-modulate" => parameters.set_modulate(false),
        "bc1-minified" => parameters.set_scale(Vec2::new(128., 128.)),
        "bc1-base-level" => parameters.set_levels_range(1, 1000),
        "transformed" => {
            parameters.set_rotation(30.);
            parameters.set_scale(Vec2::new(2., 3.));
            parameters.set_translation(Vec2::new(0.125, 0.25));
        }
        _ => {}
    }
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
