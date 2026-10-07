// Rust translation of the free functions of ChFi3d_Builder_0.hxx (ChFi3d_Builder_0.cxx,
// ChFi3d_Builder_C1.cxx, ChFi3d_Builder_CnCrn.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The free functions the fillet builders share. Those that read or write the
//! TopOpeBRepDS data structure wait for TKBool (docs/tkfillet-progress.md).
mod boundaries;
mod bounds;
mod faces;
mod pcurves;
mod spines;

pub use boundaries::{
    BoundAndPCurve, BoundEnd, build_pcurve, build_pcurve_3d, build_pcurve_on, mkbound,
    mkbound_geom, mkbound_points, mkbound_segment, mkbound_tangents_2d, mkbound_tangents_3d,
};
pub use bounds::{
    boite, boite4, bound_fac, bound_srf, enlarge_box_curve, enlarge_box_edge, enlarge_box_pcurve,
    in_period,
};
pub use faces::{conexfaces, edge_state, evalconti, kparticular};
pub use pcurves::{
    check_same_parameter, compute_pcurv, compute_pcurv_geom, compute_pcurv_on, project_pcurv,
    reparam_pcurv, same_parameter, same_parameter_geom,
};
pub use spines::{circular_spine, inter_plane_edge, spine};
