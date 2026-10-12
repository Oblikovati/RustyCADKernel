//! Match actual native class calls rather than replacing presentation geometry with a quad.
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::{
    graphic3d::*,
    image::{ImageFormat, ImagePixMap},
};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::Prs3dDrawer};
/// Configure genuine native source/material calls; `configure_textured_capture(&object,"map-3")?`.
pub fn configure_textured_capture(object: &AisTexturedShape, mode: &str) -> Result<(), Failure> {
    let defaults = Handle::new(Prs3dDrawer::new());
    defaults.setup_own_defaults()?;
    let drawer = object.attributes();
    drawer.set_link(&defaults);
    drawer.setup_own_defaults()?;
    drawer.set_auto_triangulation(false);
    drawer.u_iso_aspect().set_number(0);
    drawer.v_iso_aspect().set_number(0);
    object.set_color(Color::named(NamedColor::WHITE))?;
    drawer
        .shading_aspect()
        .aspect()
        .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    object.set_texture_pix_map(&textured_capture_pixels(mode != "bottom-up", false));
    configure_textured_mapping(object, mode)?;
    configure_textured_aspect_flags(object, mode)
}
fn configure_textured_mapping(object: &AisTexturedShape, mode: &str) -> Result<(), Failure> {
    if mode == "map-off" || !mode.starts_with("map-") {
        return Ok(());
    }
    let mapping = parse_textured_mapping(mode)?;
    if !(0..8).contains(&mapping) {
        return Err(Failure::new(&format!(
            "received mapping={mode}; expected map-0 through map-7"
        )));
    }
    object.set_texture_origin(mapping & 1 != 0, 0.25, -0.5);
    object.set_texture_repeat(mapping & 2 != 0, 2., 3.);
    object.set_texture_scale(mapping & 4 != 0, 0.5, 2.);
    Ok(())
}
fn parse_textured_mapping(mode: &str) -> Result<i32, Failure> {
    mode[4..].parse().map_err(|_| {
        Failure::new(&format!(
            "received mapping={mode}; expected map-0 through map-7"
        ))
    })
}
fn configure_textured_aspect_flags(object: &AisTexturedShape, mode: &str) -> Result<(), Failure> {
    match mode {
        "no-modulate" => object.disable_texture_modulate(),
        "map-off" => object.set_texture_map_off(),
        "transparent" => object.set_transparency(0.5)?,
        "triangles" => object.set_show_triangles(true),
        "face-boundaries" | "face-boundaries-meshed" => {
            object.attributes().set_face_boundary_draw(true)
        }
        _ => (),
    }
    Ok(())
}
/// Preserve live override ordering after initial display; `mutate_textured_capture(&object,"unset-red")?`.
pub fn mutate_textured_capture(object: &AisTexturedShape, mode: &str) -> Result<(), Failure> {
    match mode {
        "red" => object.set_color(Color::named(NamedColor::RED))?,
        "unset-red" => {
            object.set_color(Color::named(NamedColor::RED))?;
            object.unset_color()?;
        }
        "updated-pixels" => object.set_texture_pix_map(&textured_capture_pixels(true, true)),
        _ => mutate_textured_material(object, mode)?,
    }
    Ok(())
}
fn mutate_textured_material(object: &AisTexturedShape, mode: &str) -> Result<(), Failure> {
    if mode != "gold" && mode != "unset-gold" {
        return Ok(());
    }
    object.set_material(&Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold))?;
    if mode == "unset-gold" {
        object.unset_material()?;
    }
    Ok(())
}
fn textured_capture_pixels(top_down: bool, updated: bool) -> Handle<ImagePixMap> {
    let pixels = Handle::new(ImagePixMap::default());
    let first = if updated {
        [0, 255, 255, 255]
    } else {
        [255, 0, 0, 255]
    };
    let colors = textured_pixel_bytes(first);
    assert!(pixels.init_wrapper(ImageFormat::Rgba, Some(colors), 2, 2, 8));
    pixels.set_top_down(top_down);
    pixels
}
fn textured_pixel_bytes(first: [u8; 4]) -> Vec<u8> {
    // Native bytes are retained unchanged; SetTopDown selects interpretation afterward.
    [
        first,
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [255, 255, 255, 255],
    ]
    .concat()
}
