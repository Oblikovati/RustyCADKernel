//! Native-compatible tight RGBA/depth dumps shared by raw and AIS capture examples.
use martensite_vulkan::Graphic3dRenderView;
use rustycad_tkservice::{graphic3d::*, image::ImagePixMap};
use rustycad_tkv3d::v3d::{V3dImageDumpOptions, V3dView};
pub fn capture_view(
    view: &V3dView,
    width: i32,
    height: i32,
    prefix: &str,
    full: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let options = capture_options(width, height, full);
    for (kind, extension) in [
        (Graphic3dBufferType::Rgba, "rgba"),
        (Graphic3dBufferType::Depth, "depth"),
    ] {
        let options = V3dImageDumpOptions {
            buffer_type: kind,
            ..options
        };
        capture_image(view, &options, &format!("{prefix}.{extension}"))?;
    }
    Ok(())
}

fn capture_options(width: i32, height: i32, full: bool) -> V3dImageDumpOptions<'static> {
    V3dImageDumpOptions {
        width,
        height,
        to_adjust_aspect: false,
        target_z_layer_id: if full {
            GRAPHIC3D_ZLAYER_ID_BOT_OSD
        } else {
            GRAPHIC3D_ZLAYER_ID_DEFAULT
        },
        is_single_layer: !full,
        ..Default::default()
    }
}

fn capture_image(
    view: &V3dView,
    options: &V3dImageDumpOptions,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let image = ImagePixMap::default();
    if !view.to_pix_map(&image, options)? {
        return Err(format!(
            "capture failed: {:?}",
            view.view().cast::<Graphic3dRenderView>().capture()
        )
        .into());
    }
    let pixels = image.data();
    let bytes: Vec<u8> = (0..options.height as usize)
        .flat_map(|y| {
            pixels.data.row(y).unwrap()[..options.width as usize * 4]
                .iter()
                .copied()
        })
        .collect();
    std::fs::write(path, bytes)?;
    Ok(())
}
