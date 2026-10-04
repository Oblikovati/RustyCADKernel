//! Boolean topology utilities from the pinned OCCT BOPTools package.
mod connexity;
mod connexity_block;
pub use connexity::{
    make_connexity_blocks, make_connexity_blocks_from_shapes, make_connexity_compounds,
};
pub use connexity_block::ConnexityBlock;
pub mod algo_tools2d;
pub mod algo_tools3d;

mod split_reverse;
pub use split_reverse::is_split_edge_to_reverse;

mod face_orientation;
pub use face_orientation::{face_sense, is_split_face_to_reverse, is_split_shape_to_reverse};

mod orientation_warning;
pub use orientation_warning::is_split_to_reverse_with_warning;

mod shape_dimensions;
pub use shape_dimensions::{dimension, dimensions, treat_compound};
mod open_shell;
pub use open_shell::is_open_shell;

mod construction;
pub use construction::{make_container, point_on_edge};

mod correct_range;
pub use correct_range::{correct_range_edges, correct_range_face};

mod micro_edge;
pub use micro_edge::is_micro_edge;

mod face_relations;
pub use face_relations::{are_faces_same_domain, get_edge_off, get_edge_on_face};

mod vertex_tools;
pub use vertex_tools::{
    compute_vertex_point, compute_vertices, d_tolerance, make_new_vertex_edge_face,
    make_new_vertex_edges, make_new_vertex_pair, make_new_vertex_point, make_vertex,
};

mod curve_tolerance;
pub use curve_tolerance::compute_curve_tolerance;

mod solid_orientation;
pub use solid_orientation::is_inverted_solid;

mod edge_construction;
pub use edge_construction::{copy_edge, make_edge, make_section_edge, make_split_edge};

mod vertex_update;
pub use vertex_update::{update_vertex_curve, update_vertex_edge, update_vertex_pair};

mod make_pcurve;
pub use make_pcurve::make_pcurve;

mod compute_state;
pub use compute_state::{
    compute_state_by_one_point, compute_state_edge, compute_state_face, compute_state_point,
    compute_state_vertex,
};

mod wire_hole;
pub use wire_hole::is_hole;

mod orient_wire;
pub use orient_wire::orient_edges_on_wire;

mod orient_shell;
pub use orient_shell::orient_faces_on_shell;

mod block_face;
pub use block_face::is_block_in_on_face;

mod face_connexity;
pub use face_connexity::make_connexity_block;

mod shape_tolerances;
pub use shape_tolerances::correct_shape_tolerances;

mod point_curve_tolerance;
pub use point_curve_tolerance::correct_point_on_curve;
