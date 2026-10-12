//! Native DDS compressed-first loading retains supplied BC mip levels without codec dependencies.
use crate::{RenderError, error::rejected, surface_texture::*};
use rustycad_tkservice::{graphic3d::*, image::*};
use std::sync::Arc;
pub(crate) fn snapshot_compressed_texture(
    image: &ImageCompressedPixMap,
    texture: &dyn Graphic3dTextureMap,
    id: Vec<u8>,
    revision: usize,
) -> Result<SurfaceTextureImage, RenderError> {
    let source = read_compressed_surface(image, texture)?;
    Ok(SurfaceTextureImage {
        lifetime: Arc::default(),
        id,
        revision,
        extent: source.extent,
        encoding: source.encoding,
        bytes: Arc::new(source.bytes),
        top_down: true,
        mipmaps: source.levels.len() > 1,
        supplied_levels: source.levels,
    })
}
struct CompressedSurfaceSource {
    extent: [u32; 2],
    encoding: SurfaceImageEncoding,
    bytes: Vec<u8>,
    levels: Vec<usize>,
}
fn read_compressed_surface(
    image: &ImageCompressedPixMap,
    texture: &dyn Graphic3dTextureMap,
) -> Result<CompressedSurfaceSource, RenderError> {
    let encoding = compressed_encoding(image.compressed_format(), texture.is_color_map())?;
    let levels = compressed_level_bytes(image)?;
    let bytes = compressed_face_bytes(image)?;
    let extent = compressed_extent(image.size_x(), image.size_y())?;
    validate_compressed_levels(extent, encoding, &levels)?;
    if levels.is_empty() || levels.iter().sum::<usize>() != bytes.len() {
        return Err(rejected(
            (extent, levels, bytes.len()),
            "complete compressed 2D image mip buffers",
        ));
    }
    Ok(CompressedSurfaceSource {
        extent,
        encoding,
        bytes,
        levels,
    })
}
fn compressed_level_bytes(image: &ImageCompressedPixMap) -> Result<Vec<usize>, RenderError> {
    image
        .mip_maps()
        .into_iter()
        .map(|length| {
            usize::try_from(length).map_err(|_| rejected(length, "positive compressed mip bytes"))
        })
        .collect()
}
fn compressed_face_bytes(image: &ImageCompressedPixMap) -> Result<Vec<u8>, RenderError> {
    let handle = image.face_data();
    let face = handle
        .get()
        .ok_or_else(|| rejected("null DDS data", "allocated compressed face"))?;
    Ok(face.as_slice().to_vec())
}
fn compressed_encoding(
    format: ImageCompressedFormat,
    color: bool,
) -> Result<SurfaceImageEncoding, RenderError> {
    use ImageCompressedFormat::*;
    use SurfaceImageEncoding::*;
    Ok(match format {
        RgbS3tcDxt1 => Bc1Rgb(color),
        RgbaS3tcDxt1 => Bc1Rgba(color),
        RgbaS3tcDxt3 => Bc2(color),
        RgbaS3tcDxt5 => Bc3(color),
        _ => return Err(rejected(format, "native BC1/BC2/BC3 compressed format")),
    })
}

fn compressed_extent(width: i32, height: i32) -> Result<[u32; 2], RenderError> {
    let width = u32::try_from(width).map_err(|_| rejected(width, "positive compressed width"))?;
    let height =
        u32::try_from(height).map_err(|_| rejected(height, "positive compressed height"))?;
    Ok([width, height])
}
fn validate_compressed_levels(
    extent: [u32; 2],
    encoding: SurfaceImageEncoding,
    levels: &[usize],
) -> Result<(), RenderError> {
    let maximum = u32::BITS - extent[0].max(extent[1]).leading_zeros();
    if extent.contains(&0) || levels.len() > maximum as usize {
        return Err(rejected(
            (extent, levels.len()),
            "nonzero compressed extent with valid mip count",
        ));
    }
    for (level, length) in levels.iter().enumerate() {
        validate_bc_mip_size(extent, encoding, level, *length)?;
    }
    Ok(())
}
fn validate_bc_mip_size(
    extent: [u32; 2],
    encoding: SurfaceImageEncoding,
    level: usize,
    length: usize,
) -> Result<(), RenderError> {
    let block = compressed_block_bytes(encoding);
    let width = (extent[0] >> level).max(1).div_ceil(4) as usize;
    let height = (extent[1] >> level).max(1).div_ceil(4) as usize;
    let expected = width
        .checked_mul(height)
        .and_then(|value| value.checked_mul(block));
    if expected != Some(length) {
        return Err(rejected(
            (level, length, expected),
            "complete BC mip block bytes",
        ));
    }
    Ok(())
}

fn compressed_block_bytes(encoding: SurfaceImageEncoding) -> usize {
    if matches!(
        encoding,
        SurfaceImageEncoding::Bc1Rgb(_) | SurfaceImageEncoding::Bc1Rgba(_)
    ) {
        return 8;
    }
    16
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::ncollection::{BaseAllocator, Buffer};
    use rustycad_tkernel::standard::Handle;
    #[test]
    fn native_dds_compressed_first_keeps_all_supplied_bc_mips() {
        let texture = Graphic3dTexture2D::from_pix_map(Handle::default());
        let fixtures: [&[u8]; 4] = [
            include_bytes!("../tests/fixtures/surface_textures/bc1-rgb.dds"),
            include_bytes!("../tests/fixtures/surface_textures/bc1-rgba.dds"),
            include_bytes!("../tests/fixtures/surface_textures/bc2.dds"),
            include_bytes!("../tests/fixtures/surface_textures/bc3.dds"),
        ];
        for bytes in fixtures {
            let mut buffer = Buffer::new(Some(BaseAllocator::common()), bytes.len()).unwrap();
            buffer.as_mut_slice().copy_from_slice(bytes);
            let image = ImageDdsParser::load_buffer(&Handle::default(), &Handle::new(buffer), 0);
            let snapshot = snapshot_compressed_texture(&image, &texture, vec![1], 3).unwrap();
            assert_eq!(snapshot.extent, [4, 4]);
            assert_eq!(*snapshot.bytes, bytes[128..]);
            assert!(snapshot.mipmaps);
            assert_eq!(snapshot.supplied_levels.len(), 3);
        }
    }
    #[test]
    fn invalid_compressed_extent_or_block_shape_is_rejected_before_upload() {
        assert!(compressed_extent(-1, 4).is_err());
        assert!(
            validate_compressed_levels([4, 4], SurfaceImageEncoding::Bc1Rgb(true), &[7]).is_err()
        );
        assert!(
            validate_compressed_levels([4, 4], SurfaceImageEncoding::Bc1Rgb(true), &[8, 8, 8, 8])
                .is_err()
        );
        assert!(
            validate_compressed_levels([0, 4], SurfaceImageEncoding::Bc1Rgb(true), &[8]).is_err()
        );
    }
}
