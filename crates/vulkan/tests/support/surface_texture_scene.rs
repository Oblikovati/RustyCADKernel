//! Real retained ImagePixMap and Graphic3d_Texture2D commands feed sampled surface draws.
use super::scene::ViewerScene;
#[path = "surface_texture_formats.rs"]
mod formats;
#[path = "surface_texture_geometry.rs"]
pub mod geometry;
#[path = "surface_texture_lighting.rs"]
pub mod lighting;
#[path = "surface_texture_modes.rs"]
pub mod modes;
#[path = "surface_texture_projective.rs"]
pub mod projective;
#[path = "surface_texture_view.rs"]
mod texture_view;
#[path = "surface_texture_updates.rs"]
pub mod updates;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{
    graphic3d::*,
    image::{ImageFormat, ImagePixMap},
};
pub struct SurfaceTextureScene {
    pub scene: ViewerScene,
    pub texture: Handle<Graphic3dTexture2D>,
    pub aspects: Handle<Graphic3dAspects>,
    pub vertices: Handle<Graphic3dArrayOfPrimitives>,
}
impl SurfaceTextureScene {
    /// Submit ordinary retained textured geometry; `SurfaceTextureScene::with_mode(&driver,"nearest")`.
    pub fn with_mode(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Self {
        let scene = ViewerScene::new(driver);
        scene.group.clear(true);
        let texture = texture_handle(mode);
        let aspects = texture_fixture_aspects(&texture, mode);
        lighting::texture_lighting(&scene, &*aspects);
        let vertices = geometry::surface_texture_quad(mode);
        scene.group.set_group_primitives_aspect(&aspects.cast());
        scene.group.add_primitive_array(&vertices, true);
        if mode == "projective" {
            projective::projective_surface_group(&*scene.group);
        }
        texture_view::configure_surface_view(&scene, &vertices, mode);
        Self {
            scene,
            texture,
            aspects,
            vertices,
        }
    }
    /// Apply native retained updates after the first draw; `fixture.mutate("image-updated")`.
    pub fn mutate(&self, mode: &str) {
        if mode == "image-repaired" {
            self.mutate("image-empty-updated");
            self.scene.render_view().try_redraw().unwrap();
            self.mutate("image-updated");
            return;
        }
        updates::mutate_texture_content(&self.texture, mode);
        updates::mutate_texture_sampler(&self.texture, mode);
        self.scene.group.synchronize_aspects();
    }
}
pub fn texture_image(mode: &str) -> Handle<ImagePixMap> {
    if let Some(image) = formats::surface_format_image(mode) {
        return image;
    }
    let image = Handle::new(ImagePixMap::default());
    assert!(image.init_wrapper(
        ImageFormat::Rgba,
        Some(modes::texture_mode_pixels(mode)),
        2,
        2,
        8
    ));
    image.set_top_down(mode != "bottom-up");
    image
}
fn texture_fixture_aspects(
    texture: &Handle<Graphic3dTexture2D>,
    mode: &str,
) -> Handle<Graphic3dAspects> {
    let aspects = Handle::new(Graphic3dAspects::default());
    aspects.set_polygon_offsets(0, 0., 0.);
    aspects.set_texture_map(texture.cast());
    aspects.set_texture_map_on(true);
    modes::configure_texture_mode(texture, &aspects, mode);
    aspects
}

pub fn texture_handle(mode: &str) -> Handle<Graphic3dTexture2D> {
    if mode.starts_with("missing-file") {
        return Handle::new(Graphic3dTexture2D::from_file(
            &"Martensite-missing-standard2D-fixture.png".into(),
        ));
    }
    if mode.starts_with("bc") {
        let file = if mode.starts_with("bc1-") && mode != "bc1-rgba" {
            "bc1-rgb"
        } else {
            mode
        };
        let path = format!(
            "{}/tests/fixtures/surface_textures/{file}.dds",
            env!("CARGO_MANIFEST_DIR")
        );
        return Handle::new(Graphic3dTexture2D::from_file(&path.as_str().into()));
    }
    Handle::new(Graphic3dTexture2D::from_pix_map(texture_image(mode)))
}
