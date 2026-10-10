//! Real Graphic3d buffer layouts exercise the adapter independently of Vulkan availability.
#[path = "support/graphic_scene.rs"]
mod scene;
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkservice::graphic3d::*;
use scene::GraphicScene;
fn vertex_buffer() -> Handle<Graphic3dBuffer> {
    let buffer =
        Handle::new(Graphic3dBuffer::new(Graphic3dBufferBase::default_allocator()).unwrap());
    let attributes = [
        Graphic3dAttribute {
            id: Graphic3dTypeOfAttribute::Pos,
            data_type: Graphic3dTypeOfData::Vec2,
        },
        Graphic3dAttribute {
            id: Graphic3dTypeOfAttribute::Color,
            data_type: Graphic3dTypeOfData::Vec4ub,
        },
    ];
    assert!(buffer.init(3, &attributes).unwrap());
    for i in 0..3 {
        assert!(buffer.write_at(i * 12, i as f32));
        assert!(buffer.write_at(i * 12 + 4, 0.5f32));
        assert!(buffer.write_at(i * 12 + 8, Vec4::new(0u8, 255, 0, 255)));
    }
    buffer
}
fn submit(
    scene: &GraphicScene,
    buffer: &Handle<Graphic3dBuffer>,
    indices: &Handle<dyn Graphic3dIndexBufferSource>,
) {
    scene.group.add_primitive_array_buffers(
        Graphic3dTypeOfPrimitiveArray::Triangles,
        indices,
        &buffer.cast(),
        &Handle::default(),
        true,
    );
}
#[test]
fn packed_colors_vec2_positions_and_both_index_widths() {
    let scene = GraphicScene::new();
    let buffer = vertex_buffer();
    let indices =
        Handle::new(Graphic3dIndexBuffer::new(Graphic3dBufferBase::default_allocator()).unwrap());
    assert!(indices.init_indices::<u16>(3).unwrap());
    for i in 0..3 {
        assert!(indices.set_index(i, 2 - i));
    }
    submit(&scene, &buffer, &indices.cast());
    assert_eq!(
        scene.group.triangle_draws(&scene.camera).unwrap()[0].indices(),
        [2, 1, 0]
    );
    check_reinitialized_indices(&scene, &indices);
}
fn check_reinitialized_indices(scene: &GraphicScene, indices: &Graphic3dIndexBuffer) {
    assert!(indices.init_int32(3).unwrap());
    for i in 0..3 {
        assert!(indices.set_index(i, i));
    }
    let draw = scene.group.triangle_draws(&scene.camera).unwrap().remove(0);
    assert_eq!(draw.indices(), [0, 1, 2]);
    assert_eq!(draw.vertices()[2].position, [2., 0.5, 0.]);
    assert_eq!(draw.vertices()[0].color, [0., 1., 0., 1.]);
}
#[test]
fn unindexed_buffers_and_truncated_attributes() {
    let scene = GraphicScene::new();
    let buffer = vertex_buffer();
    submit(&scene, &buffer, &Handle::default());
    assert_eq!(
        scene.group.triangle_draws(&scene.camera).unwrap()[0].indices(),
        [0, 1, 2]
    );
    buffer.set_nb_elements(4);
    assert!(scene.group.triangle_draws(&scene.camera).is_err());
    buffer.set_nb_elements(-1);
    assert!(scene.group.triangle_draws(&scene.camera).is_err());
}

#[test]
fn multiple_geometry_sources_share_a_live_parent() {
    let scene = GraphicScene::new();
    scene.triangle();
    submit(&scene, &vertex_buffer(), &Handle::default());
    assert!(scene.structure.is_visible());
    assert_eq!(scene.group.triangle_draws(&scene.camera).unwrap().len(), 2);
}
#[test]
fn homogeneous_positions_require_a_finite_unit_w() {
    let scene = GraphicScene::new();
    let buffer = homogeneous_buffer();
    submit(&scene, &buffer, &Handle::default());
    assert_eq!(scene.group.triangle_draws(&scene.camera).unwrap().len(), 1);
    for w in [0., 2., f32::NAN, f32::INFINITY] {
        assert!(buffer.write_at(12, w));
        assert!(
            scene
                .group
                .triangle_draws(&scene.camera)
                .unwrap_err()
                .to_string()
                .contains("unit homogeneous w")
        );
    }
}

fn homogeneous_buffer() -> Handle<Graphic3dBuffer> {
    let buffer =
        Handle::new(Graphic3dBuffer::new(Graphic3dBufferBase::default_allocator()).unwrap());
    let attributes = [Graphic3dAttribute {
        id: Graphic3dTypeOfAttribute::Pos,
        data_type: Graphic3dTypeOfData::Vec4,
    }];
    assert!(buffer.init(3, &attributes).unwrap());
    for index in 0..3 {
        assert!(buffer.write_at(index * 16, Vec4::new(0f32, 0., 0., 1.)));
    }
    buffer
}
