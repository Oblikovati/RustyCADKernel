//! Native unlit FFP textures retain OpenGl_Sampler component and encoded-target semantics.
use crate::PrimitiveDraw;
pub(crate) fn fixed_surface_environment_flags(draw: &PrimitiveDraw) -> u32 {
    if draw.surface_texture_binding_only {
        return 0;
    }
    if let Some(style) = &draw.surface_texture {
        if !style.fixed_environment || draw.lighting.is_some() {
            return 0;
        }
        return fixed_loaded_texture_flags(style)
            + 2048 * u32::from(fixed_uniform_alpha(draw).is_some());
    }
    if draw.polygon_edge_style.is_none() {
        return 0;
    }
    draw.surface_texture_unavailable
        .as_ref()
        .map_or(0, |style| fixed_environment_flag(style.modulate, false))
}
pub(crate) fn fixed_uniform_alpha(draw: &PrimitiveDraw) -> Option<f32> {
    let style = draw.surface_texture.as_ref()?;
    if !style.fixed_environment
        || draw.lighting.is_some()
        || (style.vertex_override && draw.polygon_edge_style.is_none())
    {
        return None;
    }
    // Native FFP glColor4fv supplies current alpha directly when no color array is enabled.
    draw.vertices.first().map(|vertex| vertex.color[3])
}
fn fixed_loaded_texture_flags(style: &crate::surface_texture::SurfaceTextureStyle) -> u32 {
    fixed_environment_flag(style.modulate, style.image.alpha_only)
        + 256
        + 512 * u32::from(fixed_source_is_srgb(style.image.encoding))
        + 1024 * u32::from(style.image.alpha_only)
}
fn fixed_source_is_srgb(encoding: crate::surface_texture::SurfaceImageEncoding) -> bool {
    use crate::surface_texture::SurfaceImageEncoding::*;
    // OCCT3d097a0 FindCompressedFormat also disables sRGB DXT decoding under FFP.
    matches!(
        encoding,
        Srgb8 | Bc1Rgb(true) | Bc1Rgba(true) | Bc2(true) | Bc3(true)
    )
}
fn fixed_environment_flag(modulate: bool, alpha_only: bool) -> u32 {
    // OCCT3d097a0 applyGlobalTextureParams chooses MODULATE, ALPHA REPLACE, or RGBA DECAL.
    if modulate {
        return 32;
    }
    if alpha_only {
        return 128;
    }
    64
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface_texture::{SurfaceSamplerPolicy, SurfaceTextureImage, SurfaceTextureStyle};
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkservice::{
        graphic3d::*,
        image::{ImageFormat, ImagePixMap},
    };
    fn environment_snapshot(
        format: ImageFormat,
        pixels: Vec<u8>,
    ) -> std::sync::Arc<SurfaceTextureImage> {
        let image = Handle::new(ImagePixMap::default());
        assert!(image.init_wrapper(format, Some(pixels.clone()), 1, 1, pixels.len()));
        let texture = Handle::new(Graphic3dTexture2D::from_pix_map(image));
        crate::retained_texture_images::RetainedTextureImages::default()
            .snapshot(&texture.cast())
            .unwrap()
            .unwrap()
    }
    fn environment_image(format: ImageFormat, pixels: Vec<u8>) -> SurfaceTextureStyle {
        SurfaceTextureStyle {
            image: environment_snapshot(format, pixels),
            pending_texgen: None,
            fixed_environment: true,
            sampler: SurfaceSamplerPolicy::read(&Graphic3dTextureParams::default()),
            uv: vec![],
            back_vertex: false,
            vertex_override: false,
            modulate: true,
            native_gen_mode: Graphic3dTypeOfTextureMode::Manual,
        }
    }
    #[test]
    fn completed_texgen_keeps_fixed_transfer_and_original_component_flags() {
        let mut draw = crate::PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
        draw.surface_texture = Some(environment_image(ImageFormat::Rgba, vec![255; 4]));
        assert_eq!(fixed_surface_environment_flags(&draw), 800);
        draw.surface_texture = Some(environment_image(ImageFormat::Alpha, vec![128]));
        assert_eq!(fixed_surface_environment_flags(&draw), 1312);
        draw.surface_texture.as_mut().unwrap().modulate = false;
        assert_eq!(fixed_surface_environment_flags(&draw), 1408);
        draw.surface_texture = Some(environment_image(ImageFormat::Gray, vec![128]));
        assert_eq!(fixed_surface_environment_flags(&draw), 288);
        draw.surface_texture.as_mut().unwrap().fixed_environment = false;
        assert_eq!(fixed_surface_environment_flags(&draw), 0);
    }
    #[test]
    fn compressed_color_images_keep_the_same_fixed_component_transfer() {
        use crate::surface_texture::SurfaceImageEncoding::*;
        for encoding in [Srgb8, Bc1Rgb(true), Bc1Rgba(true), Bc2(true), Bc3(true)] {
            assert!(fixed_source_is_srgb(encoding));
        }
        for encoding in [
            Linear8,
            Float32,
            Bc1Rgb(false),
            Bc1Rgba(false),
            Bc2(false),
            Bc3(false),
        ] {
            assert!(!fixed_source_is_srgb(encoding));
        }
    }
    #[test]
    fn fixed_current_alpha_stays_uniform_until_a_real_color_array_is_enabled() {
        let vertices = vec![crate::RenderVertex::new([0.; 3], [1., 1., 1., 0.5]); 3];
        let mut draw =
            crate::PrimitiveDraw::new(vertices, vec![0, 1, 2], crate::identity_clip()).unwrap();
        draw.surface_texture = Some(environment_image(ImageFormat::Rgba, vec![255; 4]));
        assert_eq!(fixed_uniform_alpha(&draw), Some(0.5));
        assert_eq!(fixed_surface_environment_flags(&draw), 2848);
        draw.surface_texture.as_mut().unwrap().vertex_override = true;
        assert_eq!(fixed_uniform_alpha(&draw), None);
        draw.polygon_edge_style = Some(crate::LineStyle::new(1., 0xffff, 1).unwrap());
        assert_eq!(fixed_uniform_alpha(&draw), Some(0.5));
        draw.surface_texture.as_mut().unwrap().fixed_environment = false;
        assert_eq!(fixed_uniform_alpha(&draw), None);
    }
    #[test]
    fn alpha_replacement_and_rgba_decal_remain_distinct_from_modulation() {
        assert_eq!(fixed_environment_flag(true, false), 32);
        assert_eq!(fixed_environment_flag(true, true), 32);
        assert_eq!(fixed_environment_flag(false, true), 128);
        assert_eq!(fixed_environment_flag(false, false), 64);
    }
    #[test]
    fn incomplete_fixed_polygon_texture_preserves_environment_and_programmable_bypass() {
        let mut draw = crate::PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
        draw.surface_texture_unavailable =
            Some(crate::surface_texture::SurfaceTextureUnavailable {
                resource_id: b"Martensite-incomplete-fixed-polygon".to_vec(),
                modulate: true,
                vertex_override: false,
                back_vertex: false,
            });
        assert_eq!(fixed_surface_environment_flags(&draw), 0);
        draw.polygon_edge_style = Some(crate::LineStyle::new(1., 0xffff, 1).unwrap());
        assert_eq!(fixed_surface_environment_flags(&draw), 32);
        draw.surface_texture_binding_only = true;
        assert_eq!(fixed_surface_environment_flags(&draw), 0);
    }
}
