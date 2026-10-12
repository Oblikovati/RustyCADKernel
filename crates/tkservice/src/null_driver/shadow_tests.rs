//! Native CPU shadows retain parent geometry and exact parent counts after destruction.
use super::*;
use crate::graphic3d::*;
use rustycad_tkernel::{ncollection::Vec3, standard::Handle};
type ShadowScene = (
    Handle<Graphic3dStructureManager>,
    Handle<Graphic3dStructure>,
);
fn shadow_lifecycle_scene() -> ShadowScene {
    let manager =
        Handle::new(Graphic3dStructureManager::new(&NullGraphicDriver::new_handle()).unwrap());
    let structure = Graphic3dStructure::new(&manager, &Handle::default()).unwrap();
    let group = structure.current_group().unwrap();
    let points = Handle::new(Graphic3dArrayOfPoints::new(1, 0).unwrap());
    points.add_vertex_vec3(Vec3::new(0., 0., 0.)).unwrap();
    group.add_primitive_array(&points.cast(), true);
    structure.calculate_bound_box();
    (manager, structure)
}
#[test]
fn shadow_lifecycle_matches_native_root_counts_and_group_preservation() {
    let (manager, structure) = shadow_lifecycle_scene();
    let root = structure.cstructure();
    let mut actual = vec![shadow_root_row(0, &root)];
    let first = root.shadow_link(&manager).unwrap();
    actual.push(shadow_root_row(1, &root));
    let second = first.shadow_link(&manager).unwrap();
    actual.push(shadow_root_row(2, &root));
    actual.push(shadow_pair_row(&first, &second));
    drop(first);
    actual.push(shadow_root_row(4, &root));
    drop(second);
    actual.push(shadow_root_row(5, &root));
    assert_shadow_native_rows(actual);
}
fn assert_shadow_native_rows(actual: Vec<String>) {
    assert_eq!(
        actual.join("\n") + "\n",
        include_str!("../../tests/fixtures/null-shadow-lifecycle-native.txt")
    );
}
fn shadow_root_row(phase: i32, root: &Handle<dyn Graphic3dCStructure>) -> String {
    let fields = root.cstructure_base().fields.read();
    format!(
        "{phase} {} {} {} {}",
        root.ref_count(),
        fields.groups.len(),
        i32::from(fields.bnd_box.is_valid()),
        i32::from(fields.groups[0].bounding_box().is_valid())
    )
}
fn shadow_pair_row(
    first: &Handle<dyn Graphic3dCStructure>,
    second: &Handle<dyn Graphic3dCStructure>,
) -> String {
    format!(
        "3 {} {} {} {}",
        first.cstructure_base().fields.read().groups.len(),
        second.cstructure_base().fields.read().groups.len(),
        i32::from(first.bounding_box().is_valid()),
        i32::from(second.bounding_box().is_valid())
    )
}

#[test]
fn shadow_copies_infinite_and_placement_without_clearing_parent_group_nodes() {
    let (manager, structure) = shadow_lifecycle_scene();
    let root = structure.cstructure();
    root.cstructure_base().fields.write().is_infinite = true;
    let first = root.shadow_link(&manager).unwrap();
    assert!(first.cstructure_base().fields.read().is_infinite);
    first.clear();
    assert_eq!(root.cstructure_base().fields.read().groups.len(), 1);
    assert!(root.bounding_box().is_valid());
    assert!(
        root.cstructure_base().fields.read().groups[0]
            .bounding_box()
            .is_valid()
    );
}
#[test]
fn shadow_inherits_genuine_transform_and_persistence_handles() {
    use rustycad_tkmath::{gp::Transform3, toploc::Datum3};
    let (manager, structure) = shadow_lifecycle_scene();
    let root = structure.cstructure();
    let transform = Handle::new(Datum3::new(Transform3::translation([4., 5., 6.])));
    let persistence =
        Handle::new(Graphic3dTransformPers::new(Graphic3dTransModeFlags::TMF_2D).unwrap());
    root.cstructure_base().fields.write().trsf = transform.clone();
    root.cstructure_base().fields.write().trsf_pers = persistence.cast();
    let shadow = root.shadow_link(&manager).unwrap();
    assert_eq!(shadow.transformation(), transform);
    assert_eq!(shadow.transform_persistence(), root.transform_persistence());
}
