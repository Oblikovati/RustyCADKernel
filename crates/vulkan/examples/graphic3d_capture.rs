//! Viewer-driven capture for comparison with the pinned OpenGL renderer.
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Datum3,
};
use rustycad_tkservice::{graphic3d::*, image::ImagePixMap};
use rustycad_tkv3d::v3d::V3dImageDumpOptions;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if !(6..=7).contains(&args.len()) {
        return Err("expected width height perspective offset output-prefix [full]".into());
    }
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = scene::ViewerScene::new(&driver);
    let (width, height) = (args[1].parse::<i32>()?, args[2].parse::<i32>()?);
    configure(&scene, width, height, args[3].parse()?, args[4].parse()?);
    capture_scene(
        &scene,
        width,
        height,
        &args[5],
        args.get(6).is_some_and(|s| s == "full"),
    )?;
    check_capture(&driver, &scene);
    Ok(())
}
fn configure(scene: &scene::ViewerScene, width: i32, height: i32, perspective: i32, offset: f64) {
    scene.window.set_size(width, height);
    scene.view.must_be_resized();
    scene
        .view
        .camera()
        .set_projection_type(if perspective != 0 {
            Graphic3dCameraProjection::Perspective
        } else {
            Graphic3dCameraProjection::Orthographic
        });
    scene.view.camera().set_scale(20.);
    scene.view.camera().set_z_range(1., 100.).unwrap();
    let transform = Transform3::translation_vector(Vector3::new(offset, 0., 0.));
    scene
        .structure
        .set_transformation(&Handle::new(Datum3::new(transform)));
}
fn capture_scene(
    scene: &scene::ViewerScene,
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
            ..options.clone()
        };
        capture_image(scene, &options, &format!("{prefix}.{extension}"))?;
    }
    Ok(())
}

fn check_capture(driver: &Graphic3dRenderDriver, scene: &scene::ViewerScene) {
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    println!("PASS: V3d viewer capture; validation errors: 0");
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert!(!scene.group.is_null() && scene.vertices.vertex_number() == 3);
    assert!(scene.render_view().capture().is_ok());
}

fn capture_options(width: i32, height: i32, full: bool) -> V3dImageDumpOptions {
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
    scene: &scene::ViewerScene,
    options: &V3dImageDumpOptions,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let image = ImagePixMap::default();
    if !scene.view.to_pix_map(&image, options)? {
        return Err(format!("capture failed: {:?}", scene.render_view().capture()).into());
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

#[path = "support/device_options.rs"]
mod device_options;
