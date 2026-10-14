use super::*;
#[test]
fn sampled_polygon_alpha_constants_preserve_surface_layout_before_line_parameters() {
    let mut draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
    draw.polygon_edge_style = Some(crate::LineStyle::new(3., 0xff00, 2).unwrap());
    draw.surface_cutoff = Some(0.285);
    let bytes = encode_transform(&draw, fixture_extent());
    assert_eq!(bytes.len(), 80);
    assert_eq!(&bytes[64..68], &0.285_f32.to_ne_bytes());
    assert_eq!(&bytes[68..72], &8_u32.to_ne_bytes());
    draw.surface_texture_binding_only = true;
    let line = encode_transform(&draw, fixture_extent());
    assert_eq!(&line[72..76], &0xff00_u32.to_ne_bytes());
    assert_eq!(&line[76..80], &2_u32.to_ne_bytes());
}
#[test]
fn fixed_font_alpha_replace_keeps_source_color_and_ordinary_alpha_unchanged() {
    let vertex = crate::RenderVertex::new([0.; 3], [0.25, 0.5, 0.75, 0.5]);
    let ordinary = encoded_vertex_header(&vertex, false, false);
    let compatibility = encoded_vertex_header(&vertex, false, true);
    assert_eq!(packed_vertex_rgba(&ordinary), vertex.color);
    assert_eq!(packed_vertex_rgba(&compatibility), [0.25, 0.5, 0.75, 1.]);
    assert_eq!(vertex.color[3], 0.5);
}
#[test]
fn xor_pack_encodes_only_upload_rgb_and_preserves_linear_submission_and_alpha() {
    let color = [0.18, 0.4, 0.7, 0.4];
    let vertices = vec![crate::RenderVertex::new([0.; 3], color); 3];
    let mut draw = PrimitiveDraw::new(vertices, vec![0, 1, 2], crate::identity_clip()).unwrap();
    for xor in [false, true] {
        draw.text_logic_xor = xor;
        let encoded = encode_vertices(&draw);
        let packed = packed_vertex_rgba(&encoded);
        assert_eq!(packed, crate::text_logic::text_target_rgba(xor, color));
        assert_eq!(draw.vertices()[0].color, color);
        assert_eq!(packed[3], 0.4);
    }
}
fn packed_vertex_rgba(bytes: &[u8]) -> [f32; 4] {
    std::array::from_fn(|index| {
        let slice = &bytes[16 + index * 4..20 + index * 4];
        f32::from_ne_bytes(slice.try_into().unwrap())
    })
}
#[test]
fn unavailable_surface_and_alpha_only_pack_projective_default_coordinates() {
    let mut draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
    let mut bytes = vec![];
    append_surface_coordinates(&mut bytes, &draw, 0);
    assert!(bytes.is_empty());
    draw.surface_cutoff = Some(0.5);
    append_surface_coordinates(&mut bytes, &draw, 0);
    let expected: Vec<u8> = [0_f32, 0., 0., 1.]
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect();
    assert_eq!(bytes, expected);
}
#[test]
fn unlit_back_rgba_owns_existing_auxiliary_vertex_slot_and_transform_flag() {
    let mut draw = side_pack_fixture();
    draw.unlit_back_color = Some([0.1, 0.2, 0.3, 0.25]);
    let bytes = encode_vertices(&draw);
    assert_eq!(bytes.len(), 3 * 48);
    let expected: Vec<u8> = draw
        .unlit_back_color
        .unwrap()
        .into_iter()
        .flat_map(f32::to_ne_bytes)
        .collect();
    assert_eq!(&bytes[32..48], expected.as_slice());
    verify_side_transform(&draw, 80);
    draw.mesh_edges = Some(crate::RenderMeshEdges::new([1.; 4], 1., false).unwrap());
    verify_side_transform(&draw, 128);
    draw.mesh_edges = None;
    draw.unlit_back_color = None;
    assert_eq!(encode_vertices(&draw).len(), 3 * 32);
}
fn verify_side_transform(draw: &PrimitiveDraw, expected_size: usize) {
    let transform = encode_transform(draw, fixture_extent());
    assert_eq!(transform.len(), expected_size);
    assert_eq!(
        u32::from_ne_bytes(transform[68..72].try_into().unwrap()),
        4096
    );
}

fn side_pack_fixture() -> PrimitiveDraw {
    PrimitiveDraw::new(
        vec![crate::RenderVertex::new([0.; 3], [1.; 4]); 3],
        vec![0, 1, 2],
        crate::identity_clip(),
    )
    .unwrap()
}

fn fixture_extent() -> vk::Extent2D {
    vk::Extent2D {
        width: 64,
        height: 64,
    }
}
