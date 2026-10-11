//! CPU replay records native ApplyAspects and actual GL cull state, not copied oracle logic.
use super::*;
use crate::{RasterImage, RenderFaceCulling, RenderVertex, TexturedStyle, identity_clip};

fn fixture_text_draw() -> PrimitiveDraw {
    let vertices = vec![
        RenderVertex::new([0., 0., 0.], [1.; 4]),
        RenderVertex::new([1., 0., 0.], [1.; 4]),
        RenderVertex::new([0., 1., 0.], [1.; 4]),
    ];
    let image = RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
    let style = TexturedStyle::new(image, vec![[0.; 2]; 3], 0.285).unwrap();
    PrimitiveDraw::textured(vertices, vec![0, 1, 2], identity_clip(), style).unwrap()
}

const POLICY_CULLING: [Graphic3dTypeOfBackfacingModel; 4] = [
    Graphic3dTypeOfBackfacingModel::Auto,
    Graphic3dTypeOfBackfacingModel::DoubleSided,
    Graphic3dTypeOfBackfacingModel::BackCulled,
    Graphic3dTypeOfBackfacingModel::FrontCulled,
];
const POLICY_ALPHA: [Graphic3dAlphaMode; 6] = [
    Graphic3dAlphaMode::Opaque,
    Graphic3dAlphaMode::Mask,
    Graphic3dAlphaMode::Blend,
    Graphic3dAlphaMode::MaskBlend,
    Graphic3dAlphaMode::BlendAuto,
    Graphic3dAlphaMode::BlendAuto,
];

fn fixture_text_aspect(fields: &[i32]) -> Graphic3dAspectText3d {
    let aspect = Graphic3dAspectText3d::default();
    aspect.set_face_culling(POLICY_CULLING[fields[1] as usize]);
    aspect.set_alpha_mode(POLICY_ALPHA[fields[3] as usize], 0.285);
    let mut front = aspect.front_material();
    front
        .set_transparency(if fields[3] == 5 { 0.5 } else { 0. })
        .unwrap();
    aspect.set_front_material(front);
    let interior = if fields[4] == 0 {
        AspectInteriorStyle::Solid
    } else {
        AspectInteriorStyle::Hatch
    };
    aspect.set_interior_style(interior);
    aspect
}

fn expected_native_culling(fields: &[i32]) -> RenderFaceCulling {
    assert_eq!(fields[6] != 0, fields[5] != 1);
    match fields[5] {
        1 => RenderFaceCulling::None,
        2 => {
            assert_eq!(fields[7], 1029);
            RenderFaceCulling::Back
        }
        3 => {
            assert_eq!(fields[7], 1028);
            RenderFaceCulling::Front
        }
        mode => panic!("native culling {mode}; expected DoubleSided/BackCulled/FrontCulled"),
    }
}

fn check_native_policy(row: &str) {
    let fields: Vec<i32> = row.split_whitespace().map(|s| s.parse().unwrap()).collect();
    let mut draws = vec![fixture_text_draw(), fixture_text_draw()];
    apply_text_face_culling(&mut draws, &fixture_text_aspect(&fields), fields[2] != 0);
    let culling = if fields[0] == 0 {
        draws[0].face_culling
    } else {
        resolve_culling(POLICY_CULLING[fields[0] as usize], false)
    };
    assert_eq!(culling, expected_native_culling(&fields), "{row}");
    assert_eq!(draws[0].face_culling, draws[1].face_culling);
}

#[test]
fn real_workspace_and_gl_policy_matrix_matches_text_lowering() {
    let native = include_str!("../tests/fixtures/text_culling/native-policy.txt");
    for row in native.lines() {
        check_native_policy(row);
    }
    assert_eq!(native.lines().count(), 384);
}
