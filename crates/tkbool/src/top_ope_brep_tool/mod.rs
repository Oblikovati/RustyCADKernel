//! `TopOpeBRepTool`: the tools of the topological operations (surface normals for the
//! face connectivity analysis, the curve options and the section curve tool, shape
//! queries, projections, boxes, tolerances and the p-curves of edges on faces).
mod c2df;
mod classi;
mod connexity;
mod corriso;
mod curve_tool;
mod face;
mod fuse_edges;
mod geom_tool;
mod geometry;
mod hbox_tool;
mod make_transition;
mod mk_tond_g_e;
mod normal;
mod package;
mod project;
mod purge_internal_edges;
mod regus;
mod reguw;
mod shape_classifier;
mod shape_explorer;
mod shape_tool;
mod solid_classifier;
mod tol;
mod tool;
mod tool_iso;
mod tool_tangent;
mod topology;
mod two_d;

pub use c2df::TopOpeBRepToolC2DF;
pub use classi::{
    CLASSI_DIFF, CLASSI_ONE_IN_TWO, CLASSI_SAME, CLASSI_TWO_IN_ONE, CLASSI_UNKNOWN,
    TopOpeBRepToolCLASSI, fun_add_owlw,
};
pub use connexity::TopOpeBRepToolConnexity;
pub use corriso::{
    CORRISO_DECREASE, CORRISO_INCREASE, CORRISO_SPLITEDGE, EdgeFlagMap, TopOpeBRepToolCORRISO,
};
pub use curve_tool::{MadeCurves, SectionCurves, TopOpeBRepToolCurveTool, make_pcurve};
pub use face::TopOpeBRepToolFace;
pub use fuse_edges::TopOpeBRepToolFuseEdges;
pub use geom_tool::{TopOpeBRepToolGeomTool, TopOpeBRepToolOutCurveType};
pub use geometry::{
    SurfaceClosure, basis_curve2d, fun_quad_ct, fun_tool_closed, fun_tool_dir_c, fun_tool_line,
    fun_tool_line2d, fun_tool_nc_inside_s, fun_tool_nc2d_inside_s, fun_tool_ng_s, fun_tool_onapex,
    fun_tool_quad, fun_tool_quad_surface, fun_tool_quad2d, fun_tool_update_bnd2d,
};
pub use hbox_tool::{TopOpeBRepToolHBoxTool, fbox_box, fbox_get_hbox_tool, fbox_prepare};
pub use make_transition::TopOpeBRepToolMakeTransition;
pub use mk_tond_g_e::{
    DgTransition, MKT_MKI1, MKT_MKI2, MKT_MKI12, MKT_NOI, TopOpeBRepToolMkTondgE,
};
pub use normal::{min_duv, nggeom_f, nt};
pub use package::{
    correct_on_uviso, fun_tool_classif_w, fun_tool_closed_w, make_faces, print_out_curve_type,
    purge_closing_edges, purge_closing_edges_face, regularize, regularize_face, regularize_shells,
    regularize_wires,
};
pub use project::{
    ParameterRange, fun_tool_bounds, fun_tool_getindex, fun_tool_getindex2d,
    fun_tool_proj_ponbounded_f, fun_tool_proj_ponc, fun_tool_proj_ponc_range,
    fun_tool_proj_ponc_tol, fun_tool_proj_ponc2d, fun_tool_proj_ponc2d_range,
    fun_tool_proj_ponc2d_tol, fun_tool_proj_pone, fun_tool_proj_pone_tol, fun_tool_proj_ponf,
    fun_tool_proj_pons,
};
pub use purge_internal_edges::TopOpeBRepToolPurgeInternalEdges;
pub use regus::TopOpeBRepToolREGUS;
pub use reguw::{TopOpeBRepToolREGUW, fun_tool_add};
pub use shape_classifier::{
    TopOpeBRepToolShapeClassifier, fsc_get_psc, fsc_get_psc_with_reference, fsc_state_eonface,
    fsc_state_ponface,
};
pub use shape_explorer::TopOpeBRepToolShapeExplorer;
pub use shape_tool::{TopOpeBRepToolShapeTool, UvBounds};
pub use solid_classifier::TopOpeBRepToolSolidClassifier;
pub use tol::{
    FaceTolerances, ftol_face_tolerances, ftol_face_tolerances2d, ftol_face_tolerances3d,
    ftol_face_tolerances3d_on_boxes,
};
pub use tool::{
    ShapeListMap, TOOL_CLOSING, TOOL_EXTERNAL, TOOL_FORWARD, TOOL_INFFIRST, TOOL_INTERNAL,
    TOOL_ONFIRST, TOOL_ONLAST, TOOL_REVERSED, TOOL_SUPLAST, TopOpeBRepToolTOOL,
};
pub use tool_iso::{UvIsoFlags, uviso, uviso_c2df, uviso_edge};
pub use tool_tangent::{tggeom_e, tggeom_e_on_curve};
pub use topology::*;
pub use two_d::{
    PCurveOnFace, fc2d_add_new_curve_on_surface, fc2d_curve_on_surface,
    fc2d_curve_on_surface_on_edge, fc2d_editable_curve_on_surface, fc2d_has_c3d,
    fc2d_has_curve_on_surface, fc2d_has_new_curve_on_surface, fc2d_has_old_curve_on_surface,
    fc2d_make_curve_on_surface, fc2d_prepare,
};
