//! Native surface assembly is tested through actual retained Graphic3d buffers.
#[path = "support/graphic_scene.rs"]
mod scene;
#[path = "support/surface_arrays.rs"]
mod surface_arrays;
use martensite_vulkan::TriangleDraw;
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkservice::graphic3d::*;
use scene::GraphicScene;
fn submit_surface(scene: &GraphicScene, array: &Graphic3dArrayOfPrimitives) {
    scene.group.add_primitive_array_buffers(
        array.primitive_type(),
        &array.indices(),
        &array.attributes(),
        &array.bounds(),
        true,
    );
}
fn draws(scene: &GraphicScene) -> Vec<TriangleDraw> {
    scene.group.triangle_draws(&scene.camera).unwrap()
}
#[test]
fn strips_fans_and_triangles_restart_at_each_colored_bound() {
    for (mode, expected) in [
        ("surface-strip", vec![0, 1, 2, 2, 1, 3]),
        ("surface-strip-indexed", vec![0, 1, 2, 2, 1, 3]),
        ("surface-fan", vec![0, 1, 2, 0, 2, 3]),
        ("surface-fan-indexed", vec![0, 1, 2, 0, 2, 3]),
        ("surface-bound-colors", vec![0, 1, 2]),
    ] {
        check_surface(mode, &expected);
    }
}
fn check_surface(mode: &str, expected: &[u32]) {
    let scene = GraphicScene::new();
    let array = surface_arrays::surface_array(mode, 1);
    submit_surface(&scene, &array);
    let actual = draws(&scene);
    assert_eq!(actual.len(), 2);
    assert_eq!(actual[0].indices(), expected);
    let offset = array.vertex_number() as u32 / 2;
    assert_eq!(
        actual[1].indices(),
        expected.iter().map(|i| i + offset).collect::<Vec<_>>()
    );
    assert_eq!(actual[0].vertices()[0].color, [1., 0., 0., 1.]);
    assert_eq!(actual[1].vertices()[0].color, [0., 1., 0., 1.]);
}
#[test]
fn live_bound_colors_and_vertex_color_precedence() {
    for mode in ["surface-strip-indexed", "surface-vertex-colors"] {
        let scene = GraphicScene::new();
        let array = surface_arrays::surface_array(mode, 1);
        submit_surface(&scene, &array);
        array.set_bound_color_rgb(1, 0., 1., 1.).unwrap();
        let expected = if array.has_vertex_colors() {
            [1., 1., 0., 1.]
        } else {
            [0., 1., 1., 1.]
        };
        assert_eq!(draws(&scene)[0].vertices()[0].color, expected);
        array.bounds().set_color(0, Vec4::new(0., 0., 0., 0.5));
        assert_eq!(
            scene.group.triangle_draws(&scene.camera).is_ok(),
            array.has_vertex_colors()
        );
    }
}
#[test]
fn indexed_multi_bound_performance_fixture_submits_each_range() {
    let scene = GraphicScene::new();
    scene.triangle();
    assert!(scene.structure.is_visible());
    let array = surface_arrays::replace_surface(&*scene.group, "surface-strip-indexed", 128);
    assert_eq!(array.bound_number(), 256);
    let actual = draws(&scene);
    assert_eq!(actual.len(), 256);
    assert!(actual.iter().all(|draw| draw.indices().len() == 6));
}
#[test]
fn bounds_preserve_empty_ranges_ignore_tail_and_reject_overrun() {
    let scene = GraphicScene::new();
    let array = surface_arrays::surface_array("surface-strip-indexed", 1);
    submit_surface(&scene, &array);
    array.bounds().set_bound(0, 0);
    array.bounds().set_bound(1, 2);
    assert!(draws(&scene).iter().all(|draw| draw.indices().is_empty()));
    array.bounds().set_bound(1, 9);
    assert!(scene.group.triangle_draws(&scene.camera).is_err());
    array.bounds().set_bound(1, -1);
    assert!(scene.group.triangle_draws(&scene.camera).is_err());
    array.bounds().set_nb_bounds(-1);
    assert!(scene.group.triangle_draws(&scene.camera).is_err());
}
#[test]
fn unbounded_assembly_handles_short_and_incomplete_primitives() {
    let scene = GraphicScene::new();
    let array = surface_arrays::surface_array("surface-strip-indexed", 1);
    for kind in [
        Graphic3dTypeOfPrimitiveArray::Triangles,
        Graphic3dTypeOfPrimitiveArray::Trianglestrips,
        Graphic3dTypeOfPrimitiveArray::Trianglefans,
    ] {
        scene.group.clear(false);
        scene
            .group
            .set_group_primitives_aspect(&scene::unlit_style([1.; 4]));
        scene.group.add_primitive_array_buffers(
            kind,
            &array.indices(),
            &array.attributes(),
            &Handle::default(),
            false,
        );
        for count in 0..=8 {
            array.indices().set_nb_elements(count);
            let expected = if kind == Graphic3dTypeOfPrimitiveArray::Triangles {
                count / 3 * 3
            } else {
                (count - 2).max(0) * 3
            };
            assert_eq!(draws(&scene)[0].indices().len(), expected as usize);
        }
    }
}
#[test]
fn unsupported_topology_and_invalid_used_index_fail() {
    let scene = GraphicScene::new();
    let array = surface_arrays::surface_array("surface-fan-indexed", 1);
    submit_surface(&scene, &array);
    array.indices().set_index(2, 99);
    assert!(scene.group.triangle_draws(&scene.camera).is_err());
    scene.group.add_primitive_array_buffers(
        Graphic3dTypeOfPrimitiveArray::Points,
        &Handle::default(),
        &array.attributes(),
        &Handle::default(),
        false,
    );
    array.indices().set_index(2, 2);
    assert!(scene.group.triangle_draws(&scene.camera).is_err());
}

#[test]
fn reordered_strip_indices_follow_both_live_index_widths() {
    let scene = GraphicScene::new();
    let array = surface_arrays::surface_array("surface-strip-indexed", 1);
    let indices =
        Handle::new(Graphic3dIndexBuffer::new(Graphic3dBufferBase::default_allocator()).unwrap());
    scene.group.add_primitive_array_buffers(
        array.primitive_type(),
        &indices.cast(),
        &array.attributes(),
        &array.bounds(),
        false,
    );
    for short in [true, false] {
        if short {
            indices.init_indices::<u16>(8).unwrap();
        } else {
            indices.init_int32(8).unwrap();
        }
        for index in 0..8 {
            indices.set_index(index, 7 - index);
        }
        let actual = draws(&scene);
        assert_eq!(actual[0].indices(), [7, 6, 5, 5, 6, 4]);
        assert_eq!(actual[1].indices(), [3, 2, 1, 1, 2, 0]);
    }
}
#[test]
fn degenerate_triangles_preserve_strip_parity_and_ignore_unconsumed_indices() {
    let scene = GraphicScene::new();
    let array = surface_arrays::surface_array("surface-strip-indexed", 1);
    for (index, value) in [0, 1, 1, 2, 3, 99, 99, 99].into_iter().enumerate() {
        array.indices().set_index(index as i32, value);
    }
    array.bounds().set_bound(0, 5);
    array.bounds().set_bound(1, 0);
    submit_surface(&scene, &array);
    let actual = draws(&scene);
    assert_eq!(actual[0].indices(), [0, 1, 1, 1, 1, 2, 1, 2, 3]);
    assert!(actual[1].indices().is_empty());
}
