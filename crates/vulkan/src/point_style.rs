//! Native screen-space marker sprites retain the ported Graphic3d bitmap/image definitions.
use crate::{RenderError, error::rejected};
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*, image::ImagePixMap};
use std::sync::Arc;
/// Immutable nearest-sampled marker pixels; `PointStyle::new(3., [1,1], vec![[1.;4]])?`.
#[derive(Clone, Debug, PartialEq)]
pub struct PointStyle {
    pub(crate) size: f32,
    pub(crate) extent: [u32; 2],
    pub(crate) pixels: Arc<Vec<[f32; 4]>>,
}
impl PointStyle {
    /// Validate size and RGBA sprite shape before GPU work; `PointStyle::new(1., [1,1], vec![[1.;4]])?`.
    pub fn new(size: f32, extent: [u32; 2], pixels: Vec<[f32; 4]>) -> Result<Self, RenderError> {
        let count = u64::from(extent[0]) * u64::from(extent[1]);
        if !size.is_finite() || size <= 0. || extent.contains(&0) || count != pixels.len() as u64 {
            return Err(rejected(
                (size, extent, pixels.len()),
                "a positive finite point size and width*height RGBA pixels",
            ));
        }
        validate_sprite_pixels(&pixels)?;
        Ok(Self {
            size,
            extent,
            pixels: Arc::new(pixels),
        })
    }
    /// Inspect `(size, dimensions)`; `style.parameters()`.
    pub fn parameters(&self) -> (f32, [u32; 2]) {
        (self.size, self.extent)
    }
    /// Read top-down linear RGBA pixels; `style.pixels()`.
    pub fn pixels(&self) -> &[[f32; 4]] {
        &self.pixels
    }
}
pub(crate) fn marker_style(
    aspect: &dyn Graphic3dAspectsSource,
    highlighted: bool,
) -> Result<PointStyle, RenderError> {
    let kind = aspect.marker_type();
    let image = aspect.marker_image();
    if kind == AspectTypeOfMarker::Point
        || (kind == AspectTypeOfMarker::Userdefined && image.is_null())
    {
        return PointStyle::new(aspect.marker_scale(), [1, 1], vec![[1.; 4]]);
    }
    let marker = resolve_marker_image(aspect)?;
    let normal = marker.get_image();
    let image = if highlighted {
        marker.get_image_alpha()
    } else {
        normal
    };
    image_style(&image, !marker.is_colored_image() || highlighted)
}
fn validate_sprite_pixels(pixels: &[[f32; 4]]) -> Result<(), RenderError> {
    if pixels
        .iter()
        .flatten()
        .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
    {
        return Err(rejected(pixels, "finite sprite components in 0..=1"));
    }
    Ok(())
}
fn resolve_marker_image(
    aspect: &dyn Graphic3dAspectsSource,
) -> Result<Handle<Graphic3dMarkerImage>, RenderError> {
    if aspect.marker_type() == AspectTypeOfMarker::Userdefined {
        return Ok(aspect.marker_image());
    }
    let rgba = aspect.interior_color_rgba();
    let rgb = rgba.rgb();
    Graphic3dMarkerImage::standard_marker(
        aspect.marker_type(),
        aspect.marker_scale(),
        Vec4::new(
            rgb.red() as f32,
            rgb.green() as f32,
            rgb.blue() as f32,
            rgba.alpha(),
        ),
    )
    .map_err(|e| rejected(e, "a valid native marker sprite"))
}
fn image_style(image: &Handle<ImagePixMap>, alpha: bool) -> Result<PointStyle, RenderError> {
    let image = image
        .get()
        .ok_or_else(|| rejected("null marker image", "an allocated sprite"))?;
    let width = u32::try_from(image.width()).map_err(|e| rejected(e, "a 32-bit sprite width"))?;
    let height =
        u32::try_from(image.height()).map_err(|e| rejected(e, "a 32-bit sprite height"))?;
    let pixels = (0..height)
        .flat_map(|y| (0..width).map(move |x| sprite_pixel(image, x, y, alpha)))
        .collect::<Result<Vec<_>, _>>()?;
    PointStyle::new(width.max(height) as f32, [width, height], pixels)
}
fn sprite_pixel(image: &ImagePixMap, x: u32, y: u32, alpha: bool) -> Result<[f32; 4], RenderError> {
    let color = image
        .pixel_color(x as i32, y as i32, false)
        .map_err(|e| rejected(e, "a valid sprite pixel"))?;
    if alpha {
        let opacity = if image.format() == rustycad_tkservice::image::ImageFormat::Gray {
            color.rgb().red() as f32
        } else {
            color.alpha()
        };
        return Ok([1., 1., 1., opacity]);
    }
    let rgb = color.rgb();
    Ok([
        rgb.red() as f32,
        rgb.green() as f32,
        rgb.blue() as f32,
        color.alpha(),
    ])
}
