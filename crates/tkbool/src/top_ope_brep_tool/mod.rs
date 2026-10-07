//! `TopOpeBRepTool`: the tools of the topological operations (surface normals for the
//! face connectivity analysis, the curve options and the section curve tool, shape
//! queries, projections, boxes, tolerances and the p-curves of edges on faces).
mod c2df;
mod curve_tool;
mod geom_tool;
mod geometry;
mod hbox_tool;
mod normal;
mod project;
mod shape_tool;
mod tol;
mod tool_iso;
mod two_d;

pub use c2df::TopOpeBRepToolC2DF;
pub use curve_tool::{MadeCurves, SectionCurves, TopOpeBRepToolCurveTool, make_pcurve};
pub use geom_tool::{TopOpeBRepToolGeomTool, TopOpeBRepToolOutCurveType};
pub use geometry::{
    SurfaceClosure, basis_curve2d, fun_quad_ct, fun_tool_closed, fun_tool_dir_c, fun_tool_line,
    fun_tool_line2d, fun_tool_nc_inside_s, fun_tool_nc2d_inside_s, fun_tool_ng_s, fun_tool_onapex,
    fun_tool_quad, fun_tool_quad_surface, fun_tool_quad2d, fun_tool_update_bnd2d,
};
pub use hbox_tool::{TopOpeBRepToolHBoxTool, fbox_box, fbox_get_hbox_tool, fbox_prepare};
pub use normal::{min_duv, nggeom_f, nt};
pub use project::{
    ParameterRange, fun_tool_bounds, fun_tool_getindex, fun_tool_getindex2d,
    fun_tool_proj_ponbounded_f, fun_tool_proj_ponc, fun_tool_proj_ponc_range,
    fun_tool_proj_ponc_tol, fun_tool_proj_ponc2d, fun_tool_proj_ponc2d_range,
    fun_tool_proj_ponc2d_tol, fun_tool_proj_pone, fun_tool_proj_pone_tol, fun_tool_proj_ponf,
    fun_tool_proj_pons,
};
pub use shape_tool::{TopOpeBRepToolShapeTool, UvBounds};
pub use tol::{
    FaceTolerances, ftol_face_tolerances, ftol_face_tolerances2d, ftol_face_tolerances3d,
    ftol_face_tolerances3d_on_boxes,
};
pub use tool_iso::{UvIsoFlags, uviso, uviso_c2df, uviso_edge};
pub use two_d::{
    PCurveOnFace, fc2d_add_new_curve_on_surface, fc2d_curve_on_surface,
    fc2d_curve_on_surface_on_edge, fc2d_editable_curve_on_surface, fc2d_has_c3d,
    fc2d_has_curve_on_surface, fc2d_has_new_curve_on_surface, fc2d_has_old_curve_on_surface,
    fc2d_make_curve_on_surface, fc2d_prepare,
};
