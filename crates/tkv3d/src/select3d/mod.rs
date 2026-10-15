//! The Select3D package: the sensitive entities that the selecting volumes test (v3d-design.md
//! section 6).
mod bvh_index_buffer;
mod interior_point_set;
mod point_data;
mod primitive_array;
mod primitive_array_bounds;
mod primitive_array_entity;
mod primitive_array_init;
mod primitive_array_match;
mod primitive_array_overloads;
mod primitive_array_patches;
mod quadric_selection;
mod sensitive_box;
mod sensitive_circle;
mod sensitive_curve;
mod sensitive_cylinder;
mod sensitive_entity;
mod sensitive_face;
mod sensitive_group;
mod sensitive_point;
mod sensitive_poly;
mod sensitive_segment;
mod sensitive_set;
mod sensitive_set_traversal;
mod sensitive_sphere;
mod sensitive_triangle;
mod sensitive_triangulation;
mod sensitive_wire;

pub use bvh_index_buffer::Select3DBvhIndexBuffer;
pub use interior_point_set::Select3DInteriorSensitivePointSet;
pub use point_data::{Select3DPnt, Select3DPointData};
pub use primitive_array::{Select3DPrimArrayInit, Select3DSensitivePrimitiveArray};
pub use sensitive_box::Select3DSensitiveBox;
pub use sensitive_circle::Select3DSensitiveCircle;
pub use sensitive_curve::Select3DSensitiveCurve;
pub use sensitive_cylinder::Select3DSensitiveCylinder;
pub use sensitive_entity::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityFields, Select3DSensitiveEntityMembers,
    select3d_sensitive_entity_dump_json,
};
pub use sensitive_face::Select3DSensitiveFace;
pub use sensitive_group::Select3DSensitiveGroup;
pub use sensitive_point::Select3DSensitivePoint;
pub use sensitive_poly::{Select3DPolyData, Select3DSensitivePoly, Select3DSensitivePolyCore};
pub use sensitive_segment::Select3DSensitiveSegment;
pub use sensitive_set::{
    Select3DBvhBuilder3d, Select3DBvhPrimitiveSet, Select3DSensitiveSet, Select3DSensitiveSetBase,
    select3d_default_bvh_builder, select3d_set_bounding_box, select3d_set_build_bvh,
    select3d_set_center_of_geometry, select3d_set_default_bvh_builder, select3d_set_dump_json,
};
pub use sensitive_set_traversal::{
    select3d_set_clear, select3d_set_matches, select3d_set_process_elements,
};
pub use sensitive_sphere::Select3DSensitiveSphere;
pub use sensitive_triangle::{Select3DSensitiveTriangle, Select3DTypeOfSensitivity};
pub use sensitive_triangulation::Select3DSensitiveTriangulation;
pub use sensitive_wire::Select3DSensitiveWire;

mod sensitive_poly_source;
pub use sensitive_poly_source::Select3DSensitivePolySource;

#[cfg(test)]
mod primitive_array_init_tests;

#[cfg(test)]
mod primitive_array_matching_tests;

#[cfg(test)]
mod primitive_array_lifecycle_tests;

#[cfg(test)]
mod primitive_array_matching_fixtures;

mod triangulation_bounds;
mod triangulation_construction;
mod triangulation_entity;
mod triangulation_geometry;
mod triangulation_primitives;

#[cfg(test)]
mod triangulation_oracle_fixtures;
#[cfg(test)]
mod triangulation_oracle_geometry;
#[cfg(test)]
mod triangulation_oracle_snapshot;
#[cfg(test)]
mod triangulation_oracle_tests;

mod triangulation_failure;

#[cfg(test)]
mod triangulation_fault_tests;

mod triangulation_detection;

#[cfg(test)]
mod quadric_oracle_fixtures;
#[cfg(test)]
mod quadric_oracle_geometry;
#[cfg(test)]
mod quadric_oracle_snapshot;
#[cfg(test)]
mod quadric_oracle_tests;
#[cfg(test)]
mod quadric_oracle_volume;

#[cfg(test)]
mod quadric_fault_tests;
#[cfg(test)]
mod quadric_fault_volume_fake;
#[cfg(test)]
mod selector_volume_delegation;

mod sphere_detection_reference;
pub use sphere_detection_reference::Select3DSphereDetectionRef;
#[cfg(test)]
mod quadric_reference_tests;

mod group_entity;
mod group_membership;
mod group_mutation;
mod group_primitives;
mod group_references;
mod group_selection;
pub use group_references::{Select3DGroupEntitiesRef, Select3DGroupSubEntityRef};

mod group_dump;

#[cfg(test)]
mod group_oracle_geometry;

#[cfg(test)]
mod group_oracle_snapshot;

#[cfg(test)]
mod group_oracle_fixtures;

#[cfg(test)]
mod group_oracle_tests;

#[cfg(test)]
mod group_fault_child_fake;

#[cfg(test)]
mod group_reference_tests;

#[cfg(test)]
mod group_lifecycle_tests;

#[cfg(test)]
mod group_fault_tests;

#[cfg(test)]
mod group_policy_volume_fake;

#[cfg(test)]
mod group_policy_tests;

#[cfg(test)]
mod group_policy_fixtures;

#[cfg(test)]
mod group_primitive_tests;

mod group_source;
pub use group_source::{
    Select3DSensitiveGroupBase, Select3DSensitiveGroupSource, select3d_group_matches,
};

#[cfg(test)]
mod group_json_tests;

pub use group_dump::select3d_group_dump_json;
pub use group_source::{
    select3d_group_distance_to_cog, select3d_group_element_is_inside,
    select3d_group_overlaps_element,
};

#[cfg(test)]
mod group_hook_fake;

#[cfg(test)]
mod group_hook_tests;

#[cfg(test)]
mod group_hook_fixtures;

#[cfg(test)]
mod group_dump_child_fake;

#[cfg(test)]
mod group_json_failure_tests;

#[cfg(test)]
mod group_state_tests;

pub use group_source::select3d_group_downcast;

#[cfg(test)]
mod group_subclass_json_tests;

#[cfg(test)]
pub(crate) use triangulation_oracle_snapshot::compare_oracle_snapshot;

#[cfg(test)]
pub(crate) use primitive_array_matching_tests::matching_volume;
