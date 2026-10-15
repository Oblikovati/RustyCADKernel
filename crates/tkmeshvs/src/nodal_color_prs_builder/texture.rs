// SPDX-License-Identifier: GPL-2.0-only
//! The color-map texture of `MeshVS_NodalColorPrsBuilder`: the anonymous
//! `MeshVS_ImageTexture2D` (a modulated, bilinear 2D texture of a pixmap), `getNearestPow2`
//! and `CreateTexture`.
use super::MeshVSNodalColorFields;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkservice::graphic3d::{
    Graphic3dTexture2DBase, Graphic3dTexture2DSource, Graphic3dTextureMap, Graphic3dTextureRoot,
    Graphic3dTextureRootBase, Graphic3dTypeOfTexture, Graphic3dTypeOfTextureFilter,
};
use rustycad_tkservice::image::{ImageFormat, ImagePixMap};

/// The texture of the nodal colors; `MeshVSImageTexture2D::new(pixmap)`.
pub struct MeshVSImageTexture2D {
    transient: StandardTransient,
    base: Graphic3dTexture2DBase,
}
rustycad_tkernel::impl_transient!(MeshVSImageTexture2D, "MeshVS_ImageTexture2D", dyn Graphic3dTexture2DSource, transient,
    interfaces: [dyn Graphic3dTextureRoot, dyn Graphic3dTextureMap, dyn Graphic3dTexture2DSource]);

impl MeshVSImageTexture2D {
    /// The protected 2D constructor of a pixmap, then modulation and bilinear filtering.
    pub fn new(image: Handle<ImagePixMap>) -> Self {
        let base = Graphic3dTexture2DBase::from_pix_map(image, Graphic3dTypeOfTexture::TOT_2D);
        base.root.params.set_modulate(true);
        base.root
            .params
            .set_filter(Graphic3dTypeOfTextureFilter::Bilinear);
        Self {
            transient: StandardTransient::default(),
            base,
        }
    }
}

impl Graphic3dTextureRoot for MeshVSImageTexture2D {
    fn texture_root_base(&self) -> &Graphic3dTextureRootBase {
        &self.base.root
    }
}
impl Graphic3dTextureMap for MeshVSImageTexture2D {}
impl Graphic3dTexture2DSource for MeshVSImageTexture2D {
    fn texture_2d_base(&self) -> &Graphic3dTexture2DBase {
        &self.base
    }
}

/// `getNearestPow2`: the smallest power of two at least the value (clamped to half
/// `IntegerLast` against overflow). Example: `get_nearest_pow2(5) == 8`.
pub fn get_nearest_pow2(value: i32) -> i32 {
    let half_max = i32::MAX >> 1;
    let value = value.min(half_max);
    let mut result = 1;
    while result < value {
        result <<= 1;
    }
    result
}

/// `static_cast<uint8_t>(255.0 * c)`: truncation toward zero.
fn channel(component: f64) -> u8 {
    (255.0 * component) as u8
}

fn rgba(color: &Color) -> [u8; 4] {
    [
        channel(color.red()),
        channel(color.green()),
        channel(color.blue()),
        0xFF,
    ]
}

/// `CreateTexture`: row 0 holds the color map padded with its last color, row 1 the
/// invalid color; null without color.
pub(super) fn create_texture(fields: &MeshVSNodalColorFields) -> Handle<MeshVSImageTexture2D> {
    let colors: Vec<Color> = fields.texture_color_map.iter().copied().collect();
    let Some(last) = colors.last() else {
        return Handle::default();
    };
    let image = Handle::new(ImagePixMap::default());
    let width = get_nearest_pow2(colors.len() as i32) as usize;
    if !image.init_trash(ImageFormat::Rgba, width, 2, 0) {
        return Handle::default();
    }
    image.set_top_down(false);
    {
        let mut state = image.change_data();
        let pixels = (0..width).map(|col| colors.get(col).unwrap_or(last));
        for (col, color) in pixels.enumerate() {
            write_pixel(&mut state.data, 0, col, rgba(color));
            write_pixel(&mut state.data, 1, col, rgba(&fields.invalid_color));
        }
    }
    Handle::new(MeshVSImageTexture2D::new(image))
}

fn write_pixel(
    data: &mut rustycad_tkservice::image::ImagePixMapData,
    row: usize,
    col: usize,
    value: [u8; 4],
) {
    if let Some(pixel) = data.change_value(row, col) {
        pixel.copy_from_slice(&value);
    }
}
