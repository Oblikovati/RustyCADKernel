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
    let encoding = compressed_encoding(image.compressed_format(), texture.is_color_map())?;
    let levels = image
        .mip_maps()
        .into_iter()
        .map(|v| usize::try_from(v).map_err(|_| rejected(v, "positive compressed mip bytes")))
        .collect::<Result<Vec<_>, _>>()?;
    let buffer = image.face_data();
    let bytes = buffer
        .get()
        .ok_or_else(|| rejected("null DDS data", "allocated compressed face"))?
        .as_slice()
        .to_vec();
    let extent = compressed_extent(image.size_x(), image.size_y())?;
    validate_compressed_levels(extent, encoding, &levels)?;
    if extent.contains(&0) || levels.is_empty() || levels.iter().sum::<usize>() != bytes.len() {
        return Err(rejected(
            (extent, levels, bytes.len()),
            "complete compressed 2D image mip buffers",
        ));
    }
    Ok(SurfaceTextureImage {
        id,
        revision,
        extent,
        encoding,
        bytes: Arc::new(bytes),
        top_down: true,
        mipmaps: levels.len() > 1,
        supplied_levels: levels,
    })
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
    let block = if matches!(
        encoding,
        SurfaceImageEncoding::Bc1Rgb(_) | SurfaceImageEncoding::Bc1Rgba(_)
    ) {
        8
    } else {
        16
    };
    for (level, length) in levels.iter().enumerate() {
        let width = (extent[0] >> level).max(1).div_ceil(4) as usize;
        let height = (extent[1] >> level).max(1).div_ceil(4) as usize;
        let expected = width.checked_mul(height).and_then(|v| v.checked_mul(block));
        if expected != Some(*length) {
            return Err(rejected(
                (level, length, expected),
                "complete BC mip block bytes",
            ));
        }
    }
    Ok(())
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
