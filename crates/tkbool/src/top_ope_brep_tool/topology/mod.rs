// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_TOPOLOGY.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The topological `FUN_tool_*` helpers: UV tolerances and bounds of faces, points on
//! edges and faces, the orientation of sub-shapes, normals and tangents at edge
//! parameters, the parameter of a point of an edge on another edge or a face, and the
//! `FUN_ds_*` edge helpers of the build tool. The overloads take distinct names; the
//! output arguments written natively even when the function fails stay `&mut`.
mod bounds;
mod edges;
mod matter;
mod params;

pub use bounds::{
    fun_tool_bounds_face, fun_tool_closed_s, fun_tool_closed_s_flag, fun_tool_closed_s_in_u,
    fun_tool_comparebndkole, fun_tool_direct, fun_tool_geombounds, fun_tool_isobounds,
    fun_tool_maxtol, fun_tool_maxtol_of_type, fun_tool_mk_bnd2d, fun_tool_outbounds,
    fun_tool_tol_uv,
};
pub use edges::{
    fun_ds_copy_edge, fun_ds_parameter, fun_tool_closing_e, fun_tool_curves_so,
    fun_tool_curves_so_at, fun_tool_curves_so_at_both, fun_tool_ebound_f, fun_tool_eshared,
    fun_tool_find_ancestor, fun_tool_haspc, fun_tool_in_s, fun_tool_is_closing_e,
    fun_tool_make_wire, fun_tool_nbshapes, fun_tool_orient_e_in_f, fun_tool_orient_e_in_f_forward,
    fun_tool_orient_v_in_e, fun_tool_pcurve_on_f, fun_tool_pcurve_on_f_faulty, fun_tool_same_ori,
    fun_tool_shapes,
};
pub use matter::{
    fun_tool_getdxx, fun_tool_getgeomxx, fun_tool_getgeomxx_ng, fun_tool_getxx, fun_tool_getxx_ng,
};
pub use params::{
    fun_nearest_iso, fun_tool_cylinder_face, fun_tool_dir_c_on_curve, fun_tool_eitangentto_fe,
    fun_tool_etg_f, fun_tool_etg_ooe, fun_tool_find_par_in_bac, fun_tool_find_par_in_e,
    fun_tool_find_pin_bac, fun_tool_find_pin_e, fun_tool_line_bac, fun_tool_line_edge,
    fun_tool_nggeom_f, fun_tool_nggeom_f_on_edge, fun_tool_nggeom_f_on_edge_tol, fun_tool_par_e,
    fun_tool_par_e_tol, fun_tool_par_f, fun_tool_par_f_tol, fun_tool_par_v_on_e, fun_tool_paron_ef,
    fun_tool_paron_ef_tol, fun_tool_pin_c, fun_tool_pin_c_range, fun_tool_plane_face,
    fun_tool_quad_bac, fun_tool_quad_edge, fun_tool_quad_face, fun_tool_sta_pin_e,
    fun_tool_sta_pin_e_tol, fun_tool_tggeom_e_vec, fun_tool_typ, fun_tool_value_on_edge,
    fun_tool_value_on_face,
};
