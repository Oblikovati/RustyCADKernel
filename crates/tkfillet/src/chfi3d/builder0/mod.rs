// Rust translation of the free functions of ChFi3d_Builder_0.hxx (ChFi3d_Builder_0.cxx,
// ChFi3d_Builder_C1.cxx, ChFi3d_Builder_CnCrn.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The free functions the fillet builders share. Those that read or write the
//! TopOpeBRepDS data structure wait for TKBool (docs/tkfillet-progress.md).
mod arete;
mod boundaries;
mod bounds;
mod curves;
mod curves_walk;
mod faces;
mod pcurves;
mod smooth;
mod solve;
mod spines;
mod stripes;
mod topology;

pub use arete::{Arete, compute_arete, fil_common_point, trsf_trans};
pub use boundaries::{
    BoundAndPCurve, BoundEnd, build_pcurve, build_pcurve_3d, build_pcurve_on, mkbound,
    mkbound_geom, mkbound_points, mkbound_segment, mkbound_tangents_2d, mkbound_tangents_3d,
};
pub use bounds::{
    boite, boite4, bound_fac, bound_srf, enlarge_box_curve, enlarge_box_edge, enlarge_box_pcurve,
    in_period,
};
pub use curves::{IntersectionCurves, compute_curves};
pub use faces::{conexfaces, edge_state, evalconti, kparticular};
pub use pcurves::{
    check_same_parameter, compute_pcurv, compute_pcurv_geom, compute_pcurv_on, project_pcurv,
    reparam_pcurv, same_parameter, same_parameter_geom,
};
pub use smooth::{approx_by_c2, is_smooth};
pub use solve::{computes_int_pc, int_cs, parameters, search_pivot, trim_curve};
pub use spines::{circular_spine, inter_plane_edge, spine};
pub use stripes::{
    conv_tol2d_to_tol3d, edge_from_v1, eval_tol_reached, has_common_endpoint,
    has_transversal_intersection, index_of_surf_data,
};
pub use topology::{
    angle_edge, cherche_bords_libres, cherche_edge, cherche_element, cherche_face1, cherche_vertex,
    couture, couture_on_vertex, edge_common_faces, is_pseudo_seam, nb_not_degenerated_edges,
    nbface, number_of_edges, number_of_sharp_edges,
};
