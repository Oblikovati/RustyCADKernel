//! `TopOpeBRepTool`: the tools of the topological operations (surface normals for the
//! face connectivity analysis, the curve options and the section curve tool).
mod curve_tool;
mod geom_tool;
mod normal;
pub use curve_tool::{MadeCurves, SectionCurves, TopOpeBRepToolCurveTool, make_pcurve};
pub use geom_tool::{TopOpeBRepToolGeomTool, TopOpeBRepToolOutCurveType};
pub use normal::{min_duv, nggeom_f, nt};
