//! Content failures and sampler revisions are independent native retained update contracts.
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::{graphic3d::*, image::ImagePixMap};
pub fn mutate_texture_content(texture: &Graphic3dTexture2D, mode: &str) {
    let update = match mode {
        "image-stale" | "image-updated" => {
            Some((replacement_surface_image(), mode == "image-updated"))
        }
        "image-null-updated" => Some((Handle::default(), true)),
        "image-empty-updated" => Some((Handle::new(ImagePixMap::default()), true)),
        _ => None,
    };
    if let Some((image, revise)) = update {
        texture.set_image(image);
        if revise {
            texture.update_revision();
        }
    }
}
fn replacement_surface_image() -> Handle<ImagePixMap> {
    let image = super::texture_image("default");
    image.set_pixel_color_rgba(0, 0, &ColorRgba::new(0., 0., 0., 1.).unwrap(), false);
    image
}
pub fn mutate_texture_sampler(texture: &Graphic3dTexture2D, mode: &str) {
    let parameters = texture.get_params();
    if matches!(mode, "bc1-levels-updated" | "bc1-filter-levels-updated") {
        parameters.set_levels_range(1, 1000);
    }
    if matches!(mode, "sampler-updated" | "bc1-filter-levels-updated") {
        parameters.set_filter(Graphic3dTypeOfTextureFilter::Nearest);
    }
}
