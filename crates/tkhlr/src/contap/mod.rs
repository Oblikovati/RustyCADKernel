// Rust translation of ModelingAlgorithms/TKHLR/Contap at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Contour records; the contour algorithms are being ported separately.
mod line;
mod point;
mod types;
pub use line::ContapLine;
pub use point::ContapPoint;
pub use types::{ContapIType, ContapTFunction};

mod cont_ana;
mod cont_ana_cone;
mod geometry;
pub use cont_ana::ContapContAna;

mod cont_ana_cylinder;
mod cont_ana_sphere;

mod surf_props;
pub use surf_props::{ContapDerivAndNorm, ContapNormAndDn, ContapSurfProps};

mod h_curve2d_tool;
pub use h_curve2d_tool::ContapHCurve2dTool;

mod h_cont_tool;
pub use h_cont_tool::ContapHContTool;

mod surf_function;
mod surf_function_traits;
pub use surf_function::ContapSurfFunction;

mod arc_function;
mod equation;
mod normal_sampling;
pub use arc_function::ContapArcFunction;

mod iwline;
pub use iwline::ContapTheIWLineOfTheIWalking;

mod search_records;
pub use search_records::{ContapThePathPointOfTheSearch, ContapTheSegmentOfTheSearch};

mod search_inside;
pub use search_inside::{ContapTheSearchInside, ContapTheSearchInsideTypes};

mod search_boundary;
pub use search_boundary::{ContapTheSearch, ContapTheSearchTypes};

mod walking;
pub use walking::{ContapTheIWalking, ContapTheIWalkingTypes};
