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
mod sensitive_box;
mod sensitive_curve;
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
pub use sensitive_curve::{Select3DSensitiveCircle, Select3DSensitiveCurve};
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
    Select3DBvhBuilder3d, Select3DSensitiveSet, Select3DSensitiveSetBase,
    select3d_default_bvh_builder, select3d_set_bounding_box, select3d_set_build_bvh,
    select3d_set_center_of_geometry, select3d_set_default_bvh_builder, select3d_set_dump_json,
};
pub use sensitive_set_traversal::select3d_set_matches;
pub use sensitive_sphere::{Select3DSensitiveCylinder, Select3DSensitiveSphere};
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
