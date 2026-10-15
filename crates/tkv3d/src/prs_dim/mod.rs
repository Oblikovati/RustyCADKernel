//! Martensite port of OCCT PrsDim (3d097a0): dimension and relation interactive objects.
//!
//! The package class [`PrsDim`] holds the geometry extraction helpers (native output
//! arguments are returned values). The abstract `PrsDim_Dimension` and `PrsDim_Relation`
//! are traits over embedded base state, as `AIS_InteractiveObject`.
mod enums;
pub use enums::*;

mod dimension;
mod dimension_circle;
mod dimension_draw;
mod dimension_impls;
mod dimension_linear;
mod dimension_linear_draw;
mod dimension_owner;
mod dimension_selection;
pub use dimension::*;
pub use dimension_circle::{
    PrsDimCircleArc, circle_from_edge, circle_from_planar_face, init_circular_dimension,
};
pub use dimension_draw::{PrsDimDimensionDraw, PrsDimLabel, THE_3D_TEXT_MARGIN, points_for_arrow};
pub use dimension_linear::{
    PrsDimDimensionLinear, PrsDimLinearAdjust, PrsDimLinearFit,
    prs_dim_dimension_flyout_line_points,
};
pub use dimension_owner::PrsDimDimensionOwner;
pub use dimension_selection::prs_dim_dimension_compute_selection;

mod length_dimension;
mod length_dimension_init;
pub use length_dimension::{PrsDimLengthDimension, PrsDimLengthDimensionFields};

mod diameter_dimension;
mod radius_dimension;
pub use diameter_dimension::PrsDimDiameterDimension;
pub use radius_dimension::{PrsDimCircularFields, PrsDimRadiusDimension};

mod tool_edges;
mod tool_faces;
mod tool_geometry;
mod tool_misc;
mod tool_presentation;
pub use tool_edges::PrsDimTwoEdges;
pub use tool_faces::PrsDimFaceSurface;
pub use tool_geometry::{PrsDimCurveEnds, PrsDimCurveInfinite, PrsDimEnds, PrsDimProjectedEdge};
pub use tool_presentation::PrsDimProjectionStyle;

/// `PrsDim`: the package functions of the dimension presentations.
pub struct PrsDim;
