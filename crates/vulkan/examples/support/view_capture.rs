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
    for (kind, extension) in [
        (Graphic3dBufferType::Rgba, "rgba"),
        (Graphic3dBufferType::Depth, "depth"),
    ] {
        let bytes = capture_buffer(view, width, height, full, kind)?;
        std::fs::write(format!("{prefix}.{extension}"), bytes)?;
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

/// Read native-compatible bytes without filesystem I/O; `capture_buffer(view,64,64,false,Graphic3dBufferType::Rgba)`.
pub fn capture_buffer(
    view: &V3dView,
    width: i32,
    height: i32,
    full: bool,
    kind: Graphic3dBufferType,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let options = V3dImageDumpOptions {
        buffer_type: kind,
        ..capture_options(width, height, full)
    };
    capture_image_bytes(view, &options)
}
fn capture_image_bytes(
    view: &V3dView,
    options: &V3dImageDumpOptions,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let image = ImagePixMap::default();
    if !view.to_pix_map(&image, options)? {
        return Err(format!(
            "capture failed: {:?}",
            view.view().cast::<Graphic3dRenderView>().capture()
        )
        .into());
    }
    Ok(tight_capture_bytes(&image, options))
}
fn tight_capture_bytes(image: &ImagePixMap, options: &V3dImageDumpOptions) -> Vec<u8> {
    let pixels = image.data();
    let bytes: Vec<u8> = (0..options.height as usize)
        .flat_map(|y| {
            pixels.data.row(y).unwrap()[..options.width as usize * 4]
                .iter()
                .copied()
        })
        .collect();
    bytes
}
