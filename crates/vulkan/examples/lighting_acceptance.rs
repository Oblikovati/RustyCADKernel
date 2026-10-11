//! Analytic Phong-family GPU checks supplement full native view captures.
use martensite_vulkan::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/line_lighting_acceptance.rs"]
mod line_lighting_acceptance;
fn material() -> RenderMaterial {
    let mut material = RenderMaterial::default();
    for face in &mut material.common {
        face.diffuse = [0.25, 0., 0., 1.];
        face.ambient = [0., 0.25, 0., 1.];
        face.specular_shininess = [0., 0., 0., 32.];
        face.emission = [0., 0., 0.25, 1.];
    }
    material
}
fn lights(kind: Graphic3dTypeOfLightSource) -> RenderLightSet {
    let set = Handle::new(Graphic3dLightSet::default());
    let light = Handle::new(Graphic3dCLight::new(kind));
    match kind {
        Graphic3dTypeOfLightSource::Directional => light.set_direction_coords(0., 0., -1.).unwrap(),
        Graphic3dTypeOfLightSource::Positional | Graphic3dTypeOfLightSource::Spot => {
            light.set_position_coords(0., 0., 2.).unwrap()
        }
        _ => (),
    }
    if kind == Graphic3dTypeOfLightSource::Spot {
        light.set_direction_coords(0., 0., -1.).unwrap();
    }
    set.add(&light).unwrap();
    RenderLightSet::snapshot(&set, identity_clip()).unwrap()
}
fn quad(
    shading: RenderShading,
    lights: RenderLightSet,
    back: bool,
    colored: bool,
) -> PrimitiveDraw {
    let vertices = [[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.]]
        .map(|p| RenderVertex::new(p, [0., 0., 0., 1.]));
    let indices = if back {
        vec![0, 2, 1, 0, 3, 2]
    } else {
        vec![0, 1, 2, 0, 2, 3]
    };
    PrimitiveDraw::new(vertices.to_vec(), indices, identity_clip())
        .unwrap()
        .with_lighting(quad_lighting(shading, lights, colored))
        .unwrap()
}
fn quad_lighting(shading: RenderShading, lights: RenderLightSet, colored: bool) -> RenderLighting {
    RenderLighting::new(
        vec![[0., 0., 1.]; 4],
        material(),
        identity_clip(),
        identity_clip(),
        false,
        shading,
    )
    .unwrap()
    .with_lights(lights)
    .with_vertex_colors(colored, false)
}
fn check_constant(renderer: &mut VulkanRenderer, draw: PrimitiveDraw, expected: [u8; 4]) {
    let capture = renderer
        .render(&RenderFrame::new(64, 64, [0., 0., 0., 1.], vec![draw]).unwrap())
        .unwrap();
    for pixel in capture.rgba.as_chunks::<4>().0 {
        assert_eq!(*pixel, expected);
    }
    for depth in capture.depth {
        assert_eq!(depth, 0.5);
    }
}
fn check_clipped(renderer: &mut VulkanRenderer, shading: RenderShading) {
    let draw = quad(
        shading,
        lights(Graphic3dTypeOfLightSource::Directional),
        false,
        false,
    )
    .with_clipping(RenderClipping::new(vec![vec![[1., 0., 0., 0.]]], identity_clip()).unwrap());
    let capture = renderer
        .render(&RenderFrame::new(64, 64, [0., 0., 0., 1.], vec![draw]).unwrap())
        .unwrap();
    for (index, pixel) in capture.rgba.as_chunks::<4>().0.iter().enumerate() {
        let retained = index % 64 >= 32;
        assert_eq!(
            *pixel,
            if retained {
                [137, 0, 137, 255]
            } else {
                [0, 0, 0, 255]
            }
        );
        assert_eq!(capture.depth[index], if retained { 0.5 } else { 1. });
    }
}
fn check_algorithm(renderer: &mut VulkanRenderer, shading: RenderShading) {
    use Graphic3dTypeOfLightSource::{Ambient, Directional};
    let back_color = if shading == RenderShading::PhongFacet {
        [137, 0, 137, 255]
    } else {
        [0, 0, 137, 255]
    };
    for (kind, back, colored, expected) in [
        (Directional, false, false, [137, 0, 137, 255]),
        (Directional, true, false, back_color),
        (Ambient, false, false, [0, 137, 137, 255]),
        (Directional, false, true, [0, 0, 137, 255]),
    ] {
        check_constant(
            renderer,
            quad(shading, lights(kind), back, colored),
            expected,
        );
    }
    check_clipped(renderer, shading);
}
fn main() {
    let mut renderer = VulkanRenderer::new(device_options::validation_options()).unwrap();
    line_lighting_acceptance::check_lit_lines(&mut renderer);
    for shading in [
        RenderShading::PhongFacet,
        RenderShading::Gouraud,
        RenderShading::Phong,
    ] {
        check_algorithm(&mut renderer, shading);
    }
    assert!(
        renderer.validation_errors().is_empty(),
        "{:?}",
        renderer.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", renderer.device_name());
    println!("PASS: 21 lit scenes, 86016 analytic color/depth pixels; validation errors: 0");
}
