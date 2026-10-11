//! Real retained ImagePixMap and Graphic3d_Texture2D commands feed sampled surface draws.
use super::scene::ViewerScene;
#[path = "surface_texture_formats.rs"]
mod formats;
#[path = "surface_texture_lighting.rs"]
mod lighting;
#[path = "surface_texture_modes.rs"]
pub mod modes;
#[path = "surface_texture_projective.rs"]
mod projective;
#[path = "surface_texture_view.rs"]
mod texture_view;
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
    pub fn new(driver: &Handle<Graphic3dRenderDriver>) -> Self {
        Self::with_mode(driver, "default")
    }
    pub fn with_mode(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Self {
        let scene = ViewerScene::new(driver);
        scene.group.clear(true);
        let texture = texture_handle(mode);
        let aspects = Handle::new(Graphic3dAspects::default());
        aspects.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
        aspects.set_polygon_offsets(0, 0., 0.);
        aspects.set_texture_map(texture.cast());
        aspects.set_texture_map_on(true);
        modes::configure_texture_mode(&texture, &aspects, mode);
        lighting::texture_lighting(&scene, &aspects);
        let vertices = texture_quad(mode);
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
    pub fn mutate(&self, mode: &str) {
        if matches!(mode, "image-stale" | "image-updated") {
            let image = texture_image("default");
            image.set_pixel_color_rgba(
                0,
                0,
                &rustycad_tkernel::quantity::ColorRgba::new(0., 0., 0., 1.).unwrap(),
                false,
            );
            self.texture.set_image(image);
            if mode == "image-updated" {
                self.texture.update_revision();
            }
        }
        if mode == "sampler-updated" {
            self.texture
                .get_params()
                .set_filter(Graphic3dTypeOfTextureFilter::Nearest);
        }
        self.scene.group.synchronize_aspects();
    }
}
fn texture_image(mode: &str) -> Handle<ImagePixMap> {
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
fn texture_quad(mode: &str) -> Handle<Graphic3dArrayOfPrimitives> {
    let flags = GRAPHIC3D_ARRAY_FLAGS_VERTEX_TEXEL
        | GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL
        | if mode.ends_with("vertex") {
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR
        } else {
            0
        };
    let array = Handle::new(
        Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Triangles, 4, 0, 6, flags)
            .unwrap(),
    );
    for (p, uv) in [
        ([-8., -8., 0.], [0., 0.]),
        ([8., -8., 0.], [1., 0.]),
        ([8., 8., 0.], [1., 1.]),
        ([-8., 8., 0.], [0., 1.]),
    ] {
        array.add_vertex_normal_texel(p, [0., 0., 1.], uv).unwrap();
    }
    let reversed = mode.starts_with("back");
    array
        .add_edges3(
            1,
            if reversed { 3 } else { 2 },
            if reversed { 2 } else { 3 },
        )
        .unwrap();
    array
        .add_edges3(
            1,
            if reversed { 4 } else { 3 },
            if reversed { 3 } else { 4 },
        )
        .unwrap();
    if mode.ends_with("vertex") {
        for index in 1..=4 {
            array.set_vertex_color_rgb(index, 0.25, 0.5, 0.75).unwrap();
        }
    }

    array
}

fn texture_handle(mode: &str) -> Handle<Graphic3dTexture2D> {
    if mode.starts_with("bc") {
        let file = if matches!(mode, "bc1-minified" | "bc1-base-level") {
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
