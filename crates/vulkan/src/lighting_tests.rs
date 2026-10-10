use super::*;
use crate::{RenderVertex, identity_clip};
fn state(world: [f32; 16], view: [f32; 16], perspective: bool) -> RenderLighting {
    RenderLighting::new(
        vec![[0., 0., 1.]; 3],
        RenderMaterial::default(),
        world,
        view,
        perspective,
        RenderShading::Phong,
    )
    .unwrap()
}
#[test]
fn std430_layout_keeps_matrices_eye_material_and_flags_at_native_offsets() {
    let lighting = state(identity_clip(), identity_clip(), false).with_vertex_colors(true, true);
    let bytes = lighting.encode();
    assert_eq!(bytes.len(), 304);
    assert_eq!(f32::from_ne_bytes(bytes[136..140].try_into().unwrap()), 1.);
    assert_eq!(f32::from_ne_bytes(bytes[140..144].try_into().unwrap()), 0.);
    assert_eq!(u32::from_ne_bytes(bytes[288..292].try_into().unwrap()), 2);
    assert_eq!(u32::from_ne_bytes(bytes[296..300].try_into().unwrap()), 1);
    assert_eq!(u32::from_ne_bytes(bytes[300..304].try_into().unwrap()), 1);
}
#[test]
fn inverse_transpose_and_eye_use_separate_model_and_view_matrices() {
    let mut model = identity_clip();
    model[0] = 2.;
    model[5] = 4.;
    model[12] = 9.;
    let mut view = identity_clip();
    view[14] = -20.;
    let lighting = state(model, view, true);
    assert_eq!(lighting.normal[0], 0.5);
    assert_eq!(lighting.normal[5], 0.25);
    assert_eq!(lighting.eye, [0., 0., 20., 1.]);
    assert_eq!(lighting.normals(), &[[0., 0., 1.]; 3]);
    assert_eq!(lighting.shading(), RenderShading::Phong);
}
#[test]
fn validation_rejects_nonfinite_and_singular_inputs_and_mismatched_normals() {
    let normals = vec![[f32::NAN, 0., 0.]];
    assert!(
        RenderLighting::new(
            normals,
            RenderMaterial::default(),
            identity_clip(),
            identity_clip(),
            false,
            RenderShading::Phong
        )
        .is_err()
    );
    assert!(
        RenderLighting::new(
            vec![],
            RenderMaterial::default(),
            [0.; 16],
            identity_clip(),
            false,
            RenderShading::Phong
        )
        .is_err()
    );
    let draw = PrimitiveDraw::new(
        vec![RenderVertex::new([0.; 3], [1.; 4])],
        vec![],
        identity_clip(),
    )
    .unwrap();
    assert!(
        draw.with_lighting(state(identity_clip(), identity_clip(), false))
            .is_err()
    );
    let draw = PrimitiveDraw::new(
        vec![RenderVertex::new([0.; 3], [1.; 4]); 3],
        vec![0, 1, 2],
        identity_clip(),
    )
    .unwrap();
    assert!(
        draw.with_lighting(state(identity_clip(), identity_clip(), false))
            .unwrap()
            .lighting()
            .is_some()
    );
}
#[test]
fn stencil_masks_force_unlit_and_cannot_reacquire_surface_lighting() {
    let draw = PrimitiveDraw::new(
        vec![RenderVertex::new([0.; 3], [1.; 4]); 3],
        vec![0, 1, 2],
        identity_clip(),
    )
    .unwrap()
    .with_lighting(state(identity_clip(), identity_clip(), false))
    .unwrap();
    let mask = draw
        .with_capping_pass(crate::RenderCappingPass::MaskStart)
        .unwrap();
    assert!(mask.lighting().is_none());
    assert!(
        mask.with_lighting(state(identity_clip(), identity_clip(), false))
            .is_err()
    );
}

#[test]
fn std430_sources_keep_eighty_byte_stride_and_enabled_count() {
    let light = rustycad_tkernel::standard::Handle::new(
        rustycad_tkservice::graphic3d::Graphic3dCLight::new(
            rustycad_tkservice::graphic3d::Graphic3dTypeOfLightSource::Positional,
        ),
    );
    light.set_position_coords(2., 3., 4.).unwrap();
    light.set_attenuation(0.5, 0.25).unwrap();
    let set = rustycad_tkernel::standard::Handle::new(
        rustycad_tkservice::graphic3d::Graphic3dLightSet::default(),
    );
    set.add(&light).unwrap();
    let bytes = state(identity_clip(), identity_clip(), false)
        .with_lights(RenderLightSet::snapshot(&set, identity_clip()).unwrap())
        .encode();
    assert_eq!(bytes.len(), 304 + 80);
    assert_eq!(&bytes[292..296], &1u32.to_ne_bytes());
    assert_eq!(&bytes[304..308], &2u32.to_ne_bytes());
    assert_eq!(
        &bytes[336..348],
        &[2f32, 3., 4.]
            .into_iter()
            .flat_map(f32::to_ne_bytes)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        &bytes[368..376],
        &[0.5f32, 0.25]
            .into_iter()
            .flat_map(f32::to_ne_bytes)
            .collect::<Vec<_>>()
    );
}

#[test]
fn translucent_common_records_cannot_enter_the_opaque_pipeline() {
    for face in 0..2 {
        let mut material = RenderMaterial::default();
        material.common[face].diffuse[3] = 0.5;
        let error = RenderLighting::new(
            vec![],
            material,
            identity_clip(),
            identity_clip(),
            false,
            RenderShading::Phong,
        )
        .unwrap_err();
        assert!(error.to_string().contains("0.5"));
        assert!(
            error
                .to_string()
                .contains("opaque front/back diffuse alpha")
        );
    }
}
