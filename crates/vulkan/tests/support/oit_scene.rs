//! Real retained groups mirror martensite_oit_fixture.hxx, including independent alpha fields.
use super::scene::ViewerScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::{aspect::AspectInteriorStyle, graphic3d::*};
#[path = "oit_interaction.rs"]
mod interaction;
#[path = "oit_overlay.rs"]
mod overlay;

pub struct OitScene {
    pub scene: ViewerScene,
}
impl OitScene {
    /// Construct actual retained OIT geometry; `OitScene::new(&driver, false)`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, reverse: bool) -> Self {
        Self::with_scene(driver, reverse, "two")
    }
    /// Retain bounded alpha/depth fixtures; `OitScene::with_scene(&driver,false,"five")`.
    pub fn with_scene(
        driver: &Handle<Graphic3dRenderDriver>,
        reverse: bool,
        fixture: &str,
    ) -> Self {
        if fixture.starts_with("cap-") {
            return Self {
                scene: super::capping_texture_scene::CappingTextureScene::new(driver, fixture)
                    .scene,
            };
        }
        assert!(
            OIT_FIXTURES.contains(&fixture),
            "received fixture={fixture:?}; expected a retained OIT fixture from {OIT_FIXTURES:?}"
        );
        let scene = ViewerScene::new(driver);
        scene.group.clear(true);
        scene.structure.set_z_layer(GRAPHIC3D_ZLAYER_ID_DEFAULT);
        populate_oit_groups(&scene, reverse, fixture);
        overlay::install_overlay(&scene, fixture);
        Self { scene }
    }
    /// Replace live groups while retaining the same context; `fixture.replace_groups("two")`.
    pub fn replace_groups(&self, fixture: &str) {
        self.scene.structure.clear(true);
        populate_oit_groups(&self.scene, false, fixture);
        overlay::install_overlay(&self.scene, fixture);
    }
    /// Mutate real view parameters; `fixture.parameters("weighted", 0, 0., 4)`.
    pub fn parameters(&self, mode: &str, samples: i32, factor: f32, layers: i32) {
        self.scene.cview().change_rendering_params(&mut |params| {
            params.transparency_method = transparency_method(mode);
            params.nb_msaa_samples = samples;
            params.oit_depth_factor = factor;
            params.nb_oit_depth_peeling_layers = layers;
        });
    }
}
const OIT_FIXTURES: &[&str] = &[
    "two",
    "five",
    "opaque",
    "mask",
    "one-mask",
    "mask-blend",
    "auto",
    "clipped",
    "texture-nearest",
    "texture-phong",
    "texture-gouraud",
    "texture-facet",
    "texture-no-modulate",
    "sprite-mask",
    "sprite-low-mask",
    "sprite-opaque",
    "line-mask",
];
fn transparency_method(mode: &str) -> Graphic3dRenderTransparentMethod {
    match mode {
        "unordered" => Graphic3dRenderTransparentMethod::BlendUnordered,
        "weighted" => Graphic3dRenderTransparentMethod::BlendOit,
        "peeling" => Graphic3dRenderTransparentMethod::DepthPeelingOit,
        _ => panic!("received OIT mode={mode:?}; expected unordered, weighted or peeling"),
    }
}
fn populate_oit_groups(scene: &ViewerScene, reverse: bool, fixture: &str) {
    let count = if fixture.starts_with("one-") {
        1
    } else if fixture == "five" {
        5
    } else {
        2
    };
    for index in 0..count {
        let rank = if reverse { count - 1 - index } else { index };
        let group = if index == 0 {
            scene.group.clone()
        } else {
            scene.structure.new_group().unwrap()
        };
        let aspects = oit_fill_aspect(rank, fixture);
        interaction::configure_interaction(&scene, &aspects, rank, fixture);
        group.set_group_primitives_aspect(&aspects.cast());
        group.add_primitive_array(&oit_quad(rank, fixture), true);
    }
}
fn oit_fill_aspect(rank: usize, fixture: &str) -> Handle<Graphic3dAspects> {
    let style = Handle::new(Graphic3dAspects::default());
    style.set_interior_style(AspectInteriorStyle::Solid);
    style.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    style.set_face_culling(Graphic3dTypeOfBackfacingModel::DoubleSided);
    style.set_polygon_offsets(0, 0., 0.);
    let color = oit_palette(rank, fixture);
    style.set_interior_color_rgba(ColorRgba::new(color[0], color[1], color[2], color[3]).unwrap());
    let mut material = style.front_material();
    material.set_transparency(1. - color[3]).unwrap();
    style.set_front_material(material);
    style.set_alpha_mode(oit_alpha_mode(fixture), 0.5);
    style
}
fn oit_quad(rank: usize, fixture: &str) -> Handle<Graphic3dArrayOfPrimitives> {
    let flags = GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL | GRAPHIC3D_ARRAY_FLAGS_VERTEX_TEXEL;
    let vertices = Handle::new(
        Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Triangles, 6, 0, 0, flags)
            .unwrap(),
    );
    for [x, y] in [
        [-6., -6.],
        [6., -6.],
        [6., 6.],
        [-6., -6.],
        [6., 6.],
        [-6., 6.],
    ] {
        vertices
            .add_vertex_normal_texel(
                [
                    x,
                    y,
                    if fixture == "five" {
                        6. - 3. * rank as f32
                    } else if rank == 0 {
                        2.
                    } else {
                        -2.
                    },
                ],
                [0., 0., 1.],
                [(x + 6.) / 12., (y + 6.) / 12.],
            )
            .unwrap();
    }
    vertices
}
fn oit_palette(rank: usize, fixture: &str) -> [f32; 4] {
    if fixture == "five" {
        return [
            [1., 0., 0., 0.25],
            [0., 1., 0., 0.5],
            [0., 0., 1., 0.75],
            [1., 1., 0., 0.4],
            [1., 0., 1., 0.6],
        ][rank];
    }
    if rank == 0 {
        [1., 0., 0., 0.5]
    } else {
        [0., 1., 0., 0.75]
    }
}
fn oit_alpha_mode(fixture: &str) -> Graphic3dAlphaMode {
    match fixture {
        "opaque" => Graphic3dAlphaMode::Opaque,
        "mask" | "one-mask" => Graphic3dAlphaMode::Mask,
        "mask-blend" => Graphic3dAlphaMode::MaskBlend,
        "auto" => Graphic3dAlphaMode::BlendAuto,
        _ => Graphic3dAlphaMode::Blend,
    }
}
