// SPDX-License-Identifier: GPL-2.0-only
//! Permanent native declaration fixtures run independently of an OCCT installation.
use martensite_tkmeshvs::selection_policy::*;
use std::mem::size_of;

const NAMED_POLICIES: &[(&str, i32)] = &[
    ("MeshVS_BP_Mesh", MESHVS_BP_MESH),
    ("MeshVS_BP_NodalColor", MESHVS_BP_NODAL_COLOR),
    ("MeshVS_BP_ElemColor", MESHVS_BP_ELEM_COLOR),
    ("MeshVS_BP_Text", MESHVS_BP_TEXT),
    ("MeshVS_BP_Vector", MESHVS_BP_VECTOR),
    ("MeshVS_BP_User", MESHVS_BP_USER),
    ("MeshVS_BP_Default", MESHVS_BP_DEFAULT),
    ("MeshVS_SMF_Mesh", MESHVS_SMF_MESH.0),
    ("MeshVS_SMF_Node", MESHVS_SMF_NODE.0),
    ("MeshVS_SMF_0D", MESHVS_SMF_0D.0),
    ("MeshVS_SMF_Link", MESHVS_SMF_LINK.0),
    ("MeshVS_SMF_Face", MESHVS_SMF_FACE.0),
    ("MeshVS_SMF_Volume", MESHVS_SMF_VOLUME.0),
    ("MeshVS_SMF_Element", MESHVS_SMF_ELEMENT.0),
    ("MeshVS_SMF_All", MESHVS_SMF_ALL.0),
    ("MeshVS_SMF_Group", MESHVS_SMF_GROUP.0),
    (
        "MeshVS_MSM_PRECISE",
        MeshVSMeshSelectionMethod::Precise as i32,
    ),
    ("MeshVS_MSM_NODES", MeshVSMeshSelectionMethod::Nodes as i32),
    ("MeshVS_MSM_BOX", MeshVSMeshSelectionMethod::Box as i32),
    (
        "MeshVS_MSM_unnamed3",
        MeshVSMeshSelectionMethod::Unnamed3 as i32,
    ),
];

#[test]
fn all_named_values_and_native_default_priority_alias_match() {
    let rows = native_policy_rows();
    assert_eq!(NAMED_POLICIES.len(), 20);
    for ((name, value), native) in NAMED_POLICIES.iter().zip(&rows[..20]) {
        assert_eq!(*native, format!("{name} {value}"));
    }
    let negative_priority: MeshVSBuilderPriority = -7;
    assert!(negative_priority < MESHVS_BP_MESH);
    assert_eq!(MESHVS_BP_DEFAULT, MESHVS_BP_USER);
}

#[test]
fn scalar_layout_matches_each_native_declared_type() {
    let rows = native_policy_rows();
    let layouts = [
        ("priority", size_of::<MeshVSBuilderPriority>()),
        ("selection_mask", size_of::<MeshVSSelectionModeFlags>()),
        ("selection_method", size_of::<MeshVSMeshSelectionMethod>()),
    ];
    for ((name, size), native) in layouts.iter().zip(&rows[20..23]) {
        assert_eq!(*native, format!("sizeof_{name} {size}"));
    }
}

#[test]
fn every_native_mask_keeps_group_and_entity_combinations() {
    let rows = native_policy_rows();
    for (value, native) in (0..512).zip(&rows[23..]) {
        let flags = MeshVSSelectionModeFlags(value);
        assert_eq!(*native, format!("mask {value} {}", flags.0));
    }
    assert_eq!(MESHVS_SMF_ALL.0, MESHVS_SMF_ELEMENT.0 | MESHVS_SMF_NODE.0);
    let grouped_faces = MeshVSSelectionModeFlags(MESHVS_SMF_GROUP.0 | MESHVS_SMF_FACE.0);
    assert_eq!(grouped_faces.0, 264);
}

#[test]
fn mesh_selection_conversion_preserves_unnamed_native_value_without_coercion() {
    for value in 0..4 {
        assert_eq!(
            mesh_selection_method_from_native(value).unwrap() as i32,
            value
        );
    }
    for outside_native_range in [-1, 4, i32::MIN, i32::MAX] {
        assert_eq!(
            mesh_selection_method_from_native(outside_native_range),
            None
        );
    }
}

fn native_policy_rows() -> Vec<&'static str> {
    let native = concat!(
        include_str!("fixtures/selection-policy-native.txt"),
        include_str!("fixtures/selection-policy-masks-0-native.txt"),
        include_str!("fixtures/selection-policy-masks-1-native.txt")
    );
    let rows: Vec<&str> = native.lines().collect();
    assert_eq!(rows.len(), 535);
    rows
}

#[test]
fn conversion_recovers_after_rejected_values() {
    for native_value in [0, -1, 1, 4, 2, i32::MAX, 3, i32::MIN, 0] {
        let converted = mesh_selection_method_from_native(native_value);
        assert_eq!(
            converted.map(|method| method as i32),
            (0..4).contains(&native_value).then_some(native_value)
        );
    }
}
