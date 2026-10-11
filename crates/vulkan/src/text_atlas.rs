//! Pack unique glyph bitmaps with transparent gutters, preserving native glyph rectangles.
use crate::{GlyphLayout, RasterImage, RenderError, error::rejected};
use rustycad_tkservice::font::*;
use std::collections::BTreeMap;
struct GlyphBitmap {
    rect: FontRect,
    extent: [u32; 2],
    alpha: Vec<f32>,
}
struct GlyphInstance {
    symbol: u32,
    offset: [f32; 2],
}
pub(crate) fn build_glyph_layout(
    font: &FontFTFont,
    formatter: &FontTextFormatter,
) -> Result<Option<GlyphLayout>, RenderError> {
    let (glyphs, instances) = collect_glyphs(font, formatter)?;
    if instances.is_empty() {
        return Ok(None);
    }
    let (atlas, tiles) = pack_glyphs(&glyphs)?;
    let mut layout = initial_native_glyph_layout(font, formatter, atlas);
    append_native_instances(&mut layout, &glyphs, &tiles, instances);
    Ok(Some(layout))
}
fn append_native_instances(
    layout: &mut GlyphLayout,
    glyphs: &BTreeMap<u32, GlyphBitmap>,
    tiles: &BTreeMap<u32, [u32; 2]>,
    instances: Vec<GlyphInstance>,
) {
    for instance in instances {
        append_quad(
            layout,
            &glyphs[&instance.symbol],
            tiles[&instance.symbol],
            instance.offset,
        );
    }
}
// Subtitle rectangles use formatter advance/line bounds, never the union of rasterized glyph ink.
fn native_formatter_rectangle(formatter: &FontTextFormatter) -> FontRect {
    let mut rectangle = FontRect::default();
    formatter.bnd_box(&mut rectangle);
    rectangle
}
fn initial_native_glyph_layout(
    font: &FontFTFont,
    formatter: &FontTextFormatter,
    atlas: RasterImage,
) -> GlyphLayout {
    GlyphLayout {
        positions: vec![],
        uv: vec![],
        indices: vec![],
        atlas,
        formatter_bounds: native_formatter_rectangle(formatter),
        point_size: font.point_size(),
    }
}
fn collect_glyphs(
    font: &FontFTFont,
    formatter: &FontTextFormatter,
) -> Result<(BTreeMap<u32, GlyphBitmap>, Vec<GlyphInstance>), RenderError> {
    let mut glyphs = BTreeMap::new();
    let mut instances = Vec::new();
    let mut it = FontTextFormatterIterator::new(
        formatter,
        FontTextFormatterIterationFilter::ExcludeInvisible,
    );
    while it.more() {
        if let std::collections::btree_map::Entry::Vacant(slot) = glyphs.entry(it.symbol()) {
            slot.insert(copy_glyph(font, it.symbol())?);
        }
        let offset = formatter
            .bottom_left(it.symbol_position())
            .map_err(|e| rejected(e, "a formatted glyph position"))?;
        instances.push(GlyphInstance {
            symbol: it.symbol(),
            offset: [offset.x, offset.y],
        });
        it.next();
    }
    Ok((glyphs, instances))
}
fn copy_glyph(font: &FontFTFont, symbol: u32) -> Result<GlyphBitmap, RenderError> {
    if !font.render_glyph(symbol) {
        return Err(rejected(symbol, "a renderable font glyph"));
    }
    let image = font.glyph_image();
    let extent = [image.width() as u32, image.height() as u32];
    let alpha = (0..extent[1])
        .flat_map(|y| (0..extent[0]).map(move |x| (x, y)))
        .map(|(x, y)| crate::raster_image::coverage_pixel(image, x, y))
        .collect::<Result<Vec<_>, _>>()?;
    let mut rect = FontRect::default();
    font.glyph_rect(&mut rect);
    Ok(GlyphBitmap {
        rect,
        extent,
        alpha,
    })
}
fn pack_glyphs(
    glyphs: &BTreeMap<u32, GlyphBitmap>,
) -> Result<(RasterImage, BTreeMap<u32, [u32; 2]>), RenderError> {
    let width = glyphs
        .values()
        .map(|g| g.extent[0] + 2)
        .max()
        .unwrap_or(1)
        .max(256)
        .next_power_of_two();
    let (tiles, height) = tile_positions(glyphs, width);
    let mut pixels = vec![[1., 1., 1., 0.]; (width * height) as usize];
    for (symbol, glyph) in glyphs {
        copy_tile(&mut pixels, width, tiles[symbol], glyph);
    }
    Ok((RasterImage::new([width, height], pixels)?, tiles))
}
fn tile_positions(
    glyphs: &BTreeMap<u32, GlyphBitmap>,
    width: u32,
) -> (BTreeMap<u32, [u32; 2]>, u32) {
    let (mut x, mut y, mut row) = (1, 1, 0);
    let mut tiles = BTreeMap::new();
    for (symbol, glyph) in glyphs {
        if x + glyph.extent[0] + 1 > width {
            x = 1;
            y += row + 2;
            row = 0;
        }
        tiles.insert(*symbol, [x, y]);
        row = row.max(glyph.extent[1]);
        x += glyph.extent[0] + 2;
    }
    (tiles, (y + row + 1).next_power_of_two())
}
fn copy_tile(pixels: &mut [[f32; 4]], width: u32, origin: [u32; 2], glyph: &GlyphBitmap) {
    for y in 0..glyph.extent[1] {
        for x in 0..glyph.extent[0] {
            pixels[((origin[1] + y) * width + origin[0] + x) as usize][3] =
                glyph.alpha[(y * glyph.extent[0] + x) as usize];
        }
    }
}
fn append_quad(layout: &mut GlyphLayout, glyph: &GlyphBitmap, origin: [u32; 2], offset: [f32; 2]) {
    let start = layout.positions.len() as u32;
    let r = glyph.rect;
    layout.positions.extend(
        [
            [r.right, r.top],
            [r.left, r.top],
            [r.left, r.bottom],
            [r.right, r.bottom],
        ]
        .map(|p| [(p[0] + offset[0]).floor(), (p[1] + offset[1]).floor()]),
    );
    let [w, h] = layout.atlas.extent();
    let [x, y] = origin;
    let [gw, gh] = glyph.extent;
    layout.uv.extend(
        [[x + gw, y], [x, y], [x, y + gh], [x + gw, y + gh]]
            .map(|p| [p[0] as f32 / w as f32, p[1] as f32 / h as f32]),
    );
    layout.indices.extend([0, 1, 2, 3, 0, 2].map(|i| i + start));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atlas_gutters_wrapping_and_quad_uvs_preserve_coverage() {
        let glyph = GlyphBitmap {
            rect: FontRect {
                left: -1.,
                right: 129.,
                top: 2.,
                bottom: 0.,
            },
            extent: [130, 2],
            alpha: vec![0.5; 260],
        };
        let glyphs = BTreeMap::from([
            (65, glyph),
            (
                66,
                GlyphBitmap {
                    rect: FontRect::default(),
                    extent: [130, 2],
                    alpha: vec![1.; 260],
                },
            ),
        ]);
        let (atlas, tiles) = pack_glyphs(&glyphs).unwrap();
        assert_eq!(atlas.extent(), [256, 8]);
        assert_eq!(tiles[&65], [1, 1]);
        assert_eq!(tiles[&66], [1, 5]);
        assert_eq!(atlas.pixels()[0][3], 0.);
        assert_eq!(atlas.pixels()[257][3], 0.5);
        assert_eq!(atlas.pixels()[1281][3], 1.);
        let mut layout = GlyphLayout {
            positions: vec![],
            uv: vec![],
            indices: vec![],
            atlas,
            formatter_bounds: rustycad_tkservice::font::FontRect::default(),
            point_size: 16,
        };
        append_quad(&mut layout, &glyphs[&65], tiles[&65], [0.5, -0.5]);
        assert_eq!(
            layout.positions,
            vec![[129., 1.], [-1., 1.], [-1., -1.], [129., -1.]]
        );
        assert_eq!(layout.uv[1], [1. / 256., 1. / 8.]);
        assert_eq!(layout.indices, vec![0, 1, 2, 3, 0, 2]);
    }
}

#[cfg(test)]
#[path = "subtitle_bounds_tests.rs"]
mod subtitle_bounds_tests;
