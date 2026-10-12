//! The native const sequence-object contract observes public add/remove without new view ownership.
use super::*;
use rustycad_tkernel::standard::{Handle, HandleReference};
use rustycad_tkservice::null_driver::NullGraphicDriver;
struct SubviewFieldScene {
    _viewer: Handle<V3dViewer>,
    parent: Handle<dyn V3dViewSource>,
    children: Vec<Handle<dyn V3dViewSource>>,
}
fn subview_field_scene() -> SubviewFieldScene {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let parent = viewer.create_view().unwrap();
    let children = (0..3).map(|_| viewer.create_view().unwrap()).collect();
    SubviewFieldScene {
        _viewer: viewer,
        parent,
        children,
    }
}
fn apply_subview_reference_phase(scene: &SubviewFieldScene, phase: i32) {
    if phase == 1 {
        for child in &scene.children {
            scene.parent.add_subview(child);
        }
        return;
    }
    let removed = match phase {
        2 => Some(1),
        3 => Some(0),
        4 => Some(2),
        _ => None,
    };
    if let Some(index) = removed {
        scene
            .parent
            .remove_subview(scene.children[index].view_base());
    }
}
fn retained_subview_reference_row(
    scene: &SubviewFieldScene,
    reference: &V3dSubviewSequenceRef,
    phase: i32,
) -> Vec<i32> {
    let mut row = subview_sequence_metadata(scene, reference, phase);
    row.extend([
        reference_endpoint_identity(scene, reference, true),
        reference_endpoint_identity(scene, reference, false),
        reference.value_ref(2).map_or(-1, |node| {
            subview_field_identity(scene, node.object_address())
        }),
    ]);
    row.extend(scene.children.iter().map(Handle::ref_count));
    row
}
fn subview_sequence_metadata(
    scene: &SubviewFieldScene,
    reference: &V3dSubviewSequenceRef,
    phase: i32,
) -> Vec<i32> {
    vec![
        phase,
        i32::from(reference.reference_address() == scene.parent.subviews_ref().reference_address()),
        reference.lower() as i32,
        reference.upper() as i32,
        reference.len() as i32,
        i32::from(reference.is_empty()),
    ]
}
fn reference_endpoint_identity(
    scene: &SubviewFieldScene,
    reference: &V3dSubviewSequenceRef,
    first: bool,
) -> i32 {
    if reference.is_empty() {
        return 0;
    }
    let node = if first {
        reference.first_ref()
    } else {
        reference.last_ref()
    }
    .unwrap();
    subview_field_identity(scene, node.object_address())
}
fn subview_field_identity(scene: &SubviewFieldScene, address: *const ()) -> i32 {
    scene
        .children
        .iter()
        .position(|child| child.object_address() == address)
        .map_or(0, |offset| offset as i32 + 1)
}
#[test]
fn retained_native_sequence_object_and_readonly_values_match_public_add_remove_rows() {
    let scene = subview_field_scene();
    let reference = scene.parent.subviews_ref();
    let fixture = include_str!("../../tests/fixtures/v3d-subview-reference-native.txt");
    for (phase, line) in fixture.lines().enumerate() {
        apply_subview_reference_phase(&scene, phase as i32);
        let expected: Vec<i32> = line
            .split_whitespace()
            .map(|word| word.parse().unwrap())
            .collect();
        assert_eq!(
            retained_subview_reference_row(&scene, &reference, phase as i32),
            expected
        );
    }
}
fn assert_empty_subview_exceptions(reference: &V3dSubviewSequenceRef) {
    for endpoint in [reference.first_ref(), reference.last_ref()] {
        assert_eq!(
            endpoint.err().unwrap().exception_type(),
            "Standard_NoSuchObject"
        );
    }
    for index in [-1, 0, 1] {
        assert_eq!(
            reference.value_ref(index).err().unwrap().exception_type(),
            "Standard_OutOfRange"
        );
    }
}
#[test]
fn readonly_sequence_endpoints_and_indices_keep_native_exception_types() {
    let scene = subview_field_scene();
    let reference = scene.parent.subviews_ref();
    assert_empty_subview_exceptions(&reference);
    scene.parent.add_subview(&scene.children[0]);
    let first = reference.first_ref().unwrap();
    assert_eq!(
        first.reference_address(),
        reference.last_ref().unwrap().reference_address()
    );
    assert_eq!(
        first.reference_address(),
        reference.value_ref(1).unwrap().reference_address()
    );
}
#[test]
fn explicit_sequence_copy_keeps_native_getter_and_ownership_distinct() {
    let scene = subview_field_scene();
    let reference = scene.parent.subviews_ref();
    scene.parent.add_subview(&scene.children[0]);
    let address = reference.reference_address();
    let copied = reference.snapshot();
    assert_eq!(scene.children[0].ref_count(), 4);
    drop(copied);
    assert_eq!(scene.children[0].ref_count(), 3);
    assert_eq!(reference.reference_address(), address);
    assert_eq!(
        reference.cursor().value_ref().object_address(),
        scene.children[0].object_address()
    );
}
