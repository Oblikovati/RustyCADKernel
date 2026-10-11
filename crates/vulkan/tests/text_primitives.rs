//! Text layouts preserve live Graphic3d handles and view state through an injected font source.
#[path = "support/fixed_glyph_source.rs"]
mod fixed_glyph_source;
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use fixed_glyph_source::FixedGlyphSource;
use martensite_vulkan::*;
use recording_device::RecordingDevice;
use rustycad_tkernel::{
    ncollection::{Mat4, Vec4},
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkservice::{aspect::*, graphic3d::*};
use std::sync::Arc;
struct TextFixture {
    scene: scene::ViewerScene,
    device: RecordingDevice,
    font: Arc<FixedGlyphSource>,
    text: Handle<Graphic3dText>,
    style: Handle<Graphic3dAspectText3d>,
}
impl TextFixture {
    fn new() -> Self {
        let device = RecordingDevice::default();
        let font = Arc::new(FixedGlyphSource::default());
        let driver =
            Graphic3dRenderDriver::with_backend_and_fonts(device.clone(), 256, font.clone())
                .unwrap();
        let scene = scene::ViewerScene::new(&driver);
        scene.group.clear(true);
        let style = Handle::new(Graphic3dAspectText3d::default());
        scene.group.set_group_primitives_aspect(&style.cast());
        let text = Handle::new(Graphic3dText::new(1.));
        text.set_text_str("AV");
        scene.group.add_text(&text, true);
        Self {
            scene,
            device,
            font,
            text,
            style,
        }
    }
    fn redraw(&self) -> RenderFrame {
        self.scene.render_view().try_redraw().unwrap();
        self.device.last_frame()
    }
}
fn screen_width(draw: &PrimitiveDraw) -> f32 {
    let matrix = Mat4::from_data(*draw.clip_transform());
    let project = |i: usize| {
        let p = draw.vertices()[i].position;
        let c = matrix * Vec4::new(p[0], p[1], p[2], 1.);
        c.x / c.w
    };
    (project(1) - project(0)) * 32.
}
#[test]
fn retained_text_defaults_height_and_forwards_live_view_and_string() {
    let fixture = TextFixture::new();
    assert_eq!(fixture.text.height(), 16.);
    let first = fixture.redraw();
    assert_eq!(first.draws().len(), 1);
    assert_eq!(
        first.draws()[0].textured_style().unwrap().image().extent(),
        [2, 2]
    );
    fixture.text.set_text_str("changed");
    fixture
        .scene
        .view
        .change_rendering_params(&mut |p| p.resolution = 144);
    fixture.redraw();
    assert_eq!(fixture.font.requests.read().last().unwrap().1, 144);
    assert!(
        fixture
            .font
            .requests
            .read()
            .last()
            .unwrap()
            .0
            .starts_with(b"changed")
    );
    fixture.text.set_text_str("");
    assert!(fixture.redraw().draws().is_empty());
}
#[test]
fn billboard_size_stays_fixed_and_zoomable_text_follows_camera_scale() {
    let fixture = TextFixture::new();
    assert!((screen_width(&fixture.redraw().draws()[0]) - 4.).abs() < 1e-5);
    fixture.scene.view.camera().set_scale(40.);
    assert!((screen_width(&fixture.redraw().draws()[0]) - 4.).abs() < 1e-5);
    fixture.style.set_text_zoomable(true);
    let width = screen_width(&fixture.redraw().draws()[0]);
    fixture.scene.view.camera().set_scale(80.);
    assert!((screen_width(&fixture.redraw().draws()[0]) * 2. - width).abs() < 1e-5);
}
#[test]
fn text_planes_anchor_modes_and_annotation_retain_valid_geometry() {
    let fixture = TextFixture::new();
    fixture.text.set_orientation(
        &Axis2::new(
            Point3::new(1., 2., 0.),
            Direction3::new(0., 1., 1.).unwrap(),
        )
        .unwrap(),
    );
    for own in [true, false] {
        fixture.text.set_own_anchor_point(own);
        fixture
            .style
            .set_text_style(AspectTypeOfStyleText::Annotation);
        assert_eq!(fixture.redraw().draws()[0].indices().len(), 6);
    }
    fixture.scene.group.clear(true);
    assert!(fixture.redraw().draws().is_empty());
}
#[test]
fn textures_validate_uv_shape_and_alpha_threshold() {
    let image = RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
    assert!(TexturedStyle::new(image.clone(), vec![[f32::NAN, 0.]], 0.1).is_err());
    assert!(TexturedStyle::new(image.clone(), vec![], 2.).is_err());
    let style = TexturedStyle::new(image, vec![], 0.1).unwrap();
    assert!(
        PrimitiveDraw::textured(
            vec![RenderVertex::new([0.; 3], [1.; 4])],
            vec![],
            identity_clip(),
            style
        )
        .is_err()
    );
}
#[test]
fn unsupported_text_display_states_fail_explicitly() {
    let fixture = TextFixture::new();
    fixture
        .style
        .set_text_display_type(AspectTypeOfDisplayText::Blend);
    assert!(fixture.scene.render_view().try_redraw().is_err());
}

#[test]
fn text_bookkeeping_and_ordered_live_styles_match_group_contract() {
    let fixture = TextFixture::new();
    assert!(
        fixture
            .scene
            .structure
            .cstructure()
            .cstructure_base()
            .fields
            .read()
            .is_2d_text
    );
    assert!(!fixture.scene.group.is_empty());
    let second = Handle::new(Graphic3dText::new(16.));
    second.set_text_str("second");
    let style = Handle::new(Graphic3dAspectText3d::default());
    style.set_interior_color(rustycad_tkernel::quantity::Color::named(
        rustycad_tkernel::quantity::NamedColor::GREEN,
    ));
    fixture.scene.group.set_primitives_aspect(&style.cast());
    fixture.scene.group.add_text(&second, true);
    let frame = fixture.redraw();
    assert_eq!(frame.draws().len(), 2);
    assert_eq!(frame.draws()[1].vertices()[0].color, [0., 1., 0., 1.]);
    assert_ne!(
        frame.draws()[0].vertices()[0].color,
        frame.draws()[1].vertices()[0].color
    );
    fixture.scene.group.clear(true);
    assert!(fixture.scene.group.is_empty());
    assert!(fixture.redraw().draws().is_empty());
}

#[test]
fn font_cutoff_is_fixed_and_only_mask_blend_enables_coverage_blending() {
    let fixture = TextFixture::new();
    let initial = fixture.redraw().draws()[0]
        .textured_style()
        .unwrap()
        .clone();
    fixture
        .style
        .set_alpha_mode(Graphic3dAlphaMode::MaskBlend, 0.99);
    assert_eq!(
        fixture.redraw().draws()[0].textured_style().unwrap(),
        &initial
    );
    for mode in [
        Graphic3dAlphaMode::Opaque,
        Graphic3dAlphaMode::Mask,
        Graphic3dAlphaMode::BlendAuto,
        Graphic3dAlphaMode::Blend,
    ] {
        fixture.style.set_alpha_mode(mode, 0.01);
        assert_eq!(
            fixture.redraw().draws()[0].textured_style().unwrap(),
            &initial.clone().with_alpha_blending(false)
        );
    }
}

#[test]
fn shadow_precedes_foreground_with_one_pixel_offset_and_live_color() {
    let fixture = TextFixture::new();
    fixture
        .style
        .set_text_display_type(AspectTypeOfDisplayText::Shadow);
    fixture.style.set_edge_color(Color::named(NamedColor::RED));
    let frame = fixture.redraw();
    let draws = frame.draws();
    assert_eq!(draws.len(), 2);
    assert_eq!(draws[0].vertices()[0].color, [1., 0., 0., 1.]);
    assert_eq!(draws[0].textured_style(), draws[1].textured_style());
    let first = screen_anchor(&draws[0]);
    let second = screen_anchor(&draws[1]);
    assert!((first[0] - second[0] - 1.).abs() < 1e-5);
    assert!((first[1] - second[1] + 1.).abs() < 1e-5);
    fixture
        .style
        .set_text_display_type(AspectTypeOfDisplayText::Normal);
    assert_eq!(fixture.redraw().draws().len(), 1);
}
fn screen_anchor(draw: &PrimitiveDraw) -> [f32; 2] {
    let c = Mat4::from_data(*draw.clip_transform()) * Vec4::new(0., 0., 0., 1.);
    [c.x / c.w * 32., c.y / c.w * 32.]
}
const SHADOW_NATIVE_CULLING_POLICIES: [(Graphic3dTypeOfBackfacingModel, RenderFaceCulling); 3] = [
    (
        Graphic3dTypeOfBackfacingModel::BackCulled,
        RenderFaceCulling::Back,
    ),
    (
        Graphic3dTypeOfBackfacingModel::FrontCulled,
        RenderFaceCulling::Front,
    ),
    (
        Graphic3dTypeOfBackfacingModel::DoubleSided,
        RenderFaceCulling::None,
    ),
];
#[test]
fn shadow_and_foreground_keep_the_same_live_native_face_culling() {
    let fixture = TextFixture::new();
    fixture
        .style
        .set_text_display_type(AspectTypeOfDisplayText::Shadow);
    for (native, expected) in SHADOW_NATIVE_CULLING_POLICIES {
        fixture.style.set_face_culling(native);
        let frame = fixture.redraw();
        assert_eq!(frame.draws().len(), 2);
        for glyph_pass in frame.draws() {
            assert_eq!(glyph_pass.face_culling(), expected);
        }
    }
}
#[test]
fn local_plane_shadow_compounds_current_native_world_view() {
    let fixture = TextFixture::new();
    fixture
        .style
        .set_text_display_type(AspectTypeOfDisplayText::Shadow);
    let axis = Axis2::new(Point3::new(1., 2., 0.), Direction3::Z).unwrap();
    fixture.text.set_orientation(&axis);
    fixture.text.set_own_anchor_point(false);
    // Actual native draw traces at 3d097a0 show local-plane setupMatrix consumes the previous pass.
    assert_local_shadow_passes_differ(&fixture);
    fixture.text.set_own_anchor_point(true);
    assert_local_shadow_passes_differ(&fixture);
}
fn assert_local_shadow_passes_differ(fixture: &TextFixture) {
    let frame = fixture.redraw();
    assert_ne!(
        frame.draws()[0].clip_transform(),
        frame.draws()[1].clip_transform()
    );
}
#[test]
fn translucent_glyph_colors_keep_native_order_and_depth_policy() {
    let image = RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
    let style = TexturedStyle::new(image, vec![[0.; 2]; 3], 0.285).unwrap();
    let draw = PrimitiveDraw::textured(
        vec![RenderVertex::new([0.; 3], [1., 0., 0., 0.5]); 3],
        vec![0, 1, 2],
        identity_clip(),
        style,
    )
    .unwrap();
    assert!(!draw.is_surface_blended());
    assert_eq!(draw.vertices()[0].color[3], 0.5);
}
