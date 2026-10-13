//! Real retained groups mirror martensite_oit_fixture.hxx, including independent alpha fields.
use super::scene::ViewerScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::{aspect::AspectInteriorStyle, graphic3d::*};
#[path = "oit_interaction.rs"]
mod interaction;
#[path = "oit_layers.rs"]
mod layered;
#[path = "oit_overlay.rs"]
mod overlay;

pub struct OitScene {
    pub scene: ViewerScene,
    layer_structure: Option<Handle<Graphic3dStructure>>,
    cap_inputs: Option<(Handle<Graphic3dClipPlane>, Handle<Graphic3dTexture2D>)>,
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
            let cap = super::capping_texture_scene::CappingTextureScene::new(driver, fixture);
            return Self {
                scene: cap.scene,
                layer_structure: None,
                cap_inputs: Some((cap.plane, cap.texture)),
            };
        }
        assert!(
            OIT_FIXTURES.contains(&fixture),
            "received fixture={fixture:?}; expected a retained OIT fixture from {OIT_FIXTURES:?}"
        );
        let scene = ViewerScene::new(driver);
        scene.group.clear(true);
        scene.structure.set_z_layer(GRAPHIC3D_ZLAYER_ID_DEFAULT);
        let layer_structure = if fixture.starts_with("layer-") {
            Some(layered::install_oit_layers(&scene, fixture))
        } else {
            populate_oit_groups(&scene, reverse, fixture);
            overlay::install_overlay(&scene, fixture);
            None
        };
        Self {
            scene,
            layer_structure,
            cap_inputs: None,
        }
    }
    /// Release this fixture's actual viewer-owned layer before another fixture; `fixture.finish()`.
    pub fn finish(&self) {
        self.scene.view.remove();
        let Some(structure) = &self.layer_structure else {
            return;
        };
        let layer = structure.get_z_layer();
        structure.erase();
        assert!(
            !structure.is_displayed(),
            "received displayed layer={layer}; expected erased fixture structure"
        );
        // OCCT3d097a0 Viewer::DelView does not remove viewer-owned driver-global layers.
        let removed = self.scene.viewer.remove_z_layer(layer).unwrap();
        assert!(
            removed,
            "received layer={layer}; expected this fixture's owned custom layer"
        );
    }
    /// Replace live groups while retaining the same context; `fixture.replace_groups("two")`.
    pub fn replace_groups(&self, fixture: &str) {
        self.scene.structure.clear(true);
        populate_oit_groups(&self.scene, false, fixture);
        overlay::install_overlay(&self.scene, fixture);
    }
    /// Change the actual retained cap inputs after initial rendering; `fixture.mutate_inputs("cap-alpha-updated")`.
    pub fn mutate_inputs(&self, mode: &str) {
        if let Some((plane, texture)) = &self.cap_inputs {
            super::capping_texture_scene::mutate_cap_inputs(&self.scene, plane, texture, mode);
        }
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
    "layer-clear",
    "layer-shared",
    "tied",
    "occluded",
    "near-boundary",
    "far-boundary",
    "far-boundary-opaque",
    "far-interior",
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
    "text-colored",
    "text-shadow",
    "text-subtitle",
    "text-decal",
    "text-xor",
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
    } else if fixture == "occluded" {
        3
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
        interaction::configure_interaction(scene, &aspects, rank, fixture);
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
    style.set_alpha_mode(
        if fixture == "occluded" && rank == 2 {
            Graphic3dAlphaMode::Opaque
        } else {
            oit_alpha_mode(fixture)
        },
        0.5,
    );
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
                [x, y, oit_quad_depth(rank, fixture)],
                [0., 0., 1.],
                [(x + 6.) / 12., (y + 6.) / 12.],
            )
            .unwrap();
    }
    vertices
}
fn oit_quad_depth(rank: usize, fixture: &str) -> f32 {
    match fixture {
        "tied" => 2.,
        "near-boundary" => 19.,
        "far-boundary" | "far-boundary-opaque" => -80.,
        "far-interior" => -79.,
        "occluded" if rank == 2 => 4.,
        "five" => 6. - 3. * rank as f32,
        _ if rank == 0 => 2.,
        _ => -2.,
    }
}
fn oit_palette(rank: usize, fixture: &str) -> [f32; 4] {
    if fixture == "occluded" && rank == 2 {
        return [1.; 4];
    }
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
        "opaque" | "far-boundary-opaque" => Graphic3dAlphaMode::Opaque,
        "mask" | "one-mask" => Graphic3dAlphaMode::Mask,
        "mask-blend" => Graphic3dAlphaMode::MaskBlend,
        "auto" => Graphic3dAlphaMode::BlendAuto,
        _ => Graphic3dAlphaMode::Blend,
    }
}
