//! Controlled Graphic3d camera/geometry capture for comparison with the pinned OpenGL renderer.
#[path = "../tests/support/graphic_scene.rs"]
mod scene;
use martensite_vulkan::{RenderBackend, RenderFrame, VulkanOptions, VulkanRenderer};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Direction3, Point3, Transform3, Vector3},
    toploc::Datum3,
};
use rustycad_tkservice::graphic3d::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 6 {
        return Err("expected width height perspective offset output-prefix".into());
    }
    let (width, height, perspective, offset) = (
        args[1].parse::<u32>()?,
        args[2].parse::<u32>()?,
        args[3].parse::<i32>()?,
        args[4].parse::<f64>()?,
    );
    let scene = prepare_scene(width, height, perspective, offset)?;
    capture_scene(&scene, width, height, &args[5])
}
fn prepare_scene(
    width: u32,
    height: u32,
    perspective: i32,
    offset: f64,
) -> Result<scene::GraphicScene, Box<dyn std::error::Error>> {
    let scene = scene::GraphicScene::new();
    configure(&scene, width, height, perspective, offset)?;
    let vertices = scene.triangle();
    for (index, [x, y, z]) in [[-8., -8., 0.], [8., -8., 0.], [0., 8., 0.]]
        .into_iter()
        .enumerate()
    {
        vertices.set_vertice(index as i32 + 1, x, y, z)?;
    }
    Ok(scene)
}
fn capture_scene(
    scene: &scene::GraphicScene,
    width: u32,
    height: u32,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = RenderFrame::new(
        width,
        height,
        [0., 0., 1., 1.],
        scene.group.triangle_draws(&scene.camera)?,
    )?;
    let mut renderer = VulkanRenderer::new(VulkanOptions {
        validation: true,
        ..Default::default()
    })?;
    let capture = renderer.render(&frame)?;
    check_validation(&renderer);
    write_capture(prefix, &capture)?;
    println!(
        "PASS: Graphic3d capture on {}; validation errors: 0",
        renderer.device_name()
    );
    Ok(())
}
fn configure(
    scene: &scene::GraphicScene,
    width: u32,
    height: u32,
    perspective: i32,
    offset: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    scene.camera.set_eye(&Point3::new(0., 0., 20.))?;
    scene.camera.set_center(&Point3::default())?;
    scene.camera.set_up(&Direction3::new(0., 1., 0.)?);
    scene.camera.set_projection_type(if perspective != 0 {
        Graphic3dCameraProjection::Perspective
    } else {
        Graphic3dCameraProjection::Orthographic
    });
    scene.camera.set_scale(20.);
    scene
        .camera
        .set_aspect(f64::from(width) / f64::from(height));
    scene.camera.set_z_range(1., 100.)?;
    let transform = Transform3::translation_vector(Vector3::new(offset, 0., 0.));
    scene
        .structure
        .set_transformation(&Handle::new(Datum3::new(transform)));
    Ok(())
}

fn write_capture(prefix: &str, capture: &martensite_vulkan::FrameCapture) -> std::io::Result<()> {
    std::fs::write(format!("{}.rgba", prefix), &capture.rgba)?;
    let depth: Vec<u8> = capture
        .depth
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect();
    std::fs::write(format!("{}.depth", prefix), depth)?;
    Ok(())
}

fn check_validation(renderer: &VulkanRenderer) {
    assert!(
        renderer.validation_errors().is_empty(),
        "{:?}",
        renderer.validation_errors()
    );
}
