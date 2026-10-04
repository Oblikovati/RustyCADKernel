//! BOPTools_AlgoTools2D pcurve utilities.
mod periodic;
pub use periodic::{adjust_pcurve_on_face, adjust_pcurve_on_surface};
mod projection;
pub use projection::{make_pcurve_full_curve, make_pcurve_on_face};
mod edge_pcurve;
pub use edge_pcurve::{
    curve_on_surface, has_curve_on_surface, has_curve_on_surface_simple,
    intermediate_point_on_edge, make_2d,
};
