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

mod tangent;
pub use tangent::edge_tangent;

mod edge_utilities;
pub use edge_utilities::{
    adjust_pcurve_for_curve, build_pcurve_for_edge_on_face, curve_on_surface_simple,
    intermediate_point, is_edge_isoline, point_on_surface,
};
