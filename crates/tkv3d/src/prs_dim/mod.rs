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

mod angle_dimension;
mod angle_dimension_draw;
mod angle_dimension_init;
mod angle_dimension_layout;
pub use angle_dimension::{PrsDimAngleDimension, PrsDimAngleDimensionFields};
pub use angle_dimension_layout::PrsDimAngleAdjust;

mod diameter_dimension;
mod radius_dimension;
pub use diameter_dimension::PrsDimDiameterDimension;
pub use radius_dimension::{PrsDimCircularFields, PrsDimRadiusDimension};

mod chamf_dimension;
mod concentric_relation;
mod ellipse_radius_dimension;
mod equal_distance_length;
mod equal_distance_relation;
mod equal_radius_relation;
pub use equal_distance_length::{PrsDimEqualDistanceContext, PrsDimEqualDistancePoints};
pub use equal_distance_relation::{PrsDimEqualDistanceFields, PrsDimEqualDistanceRelation};
mod fix_relation;
mod max_min_radius_dimension;
pub use fix_relation::{PrsDimFixFields, PrsDimFixRelation};
mod midpoint_points;
mod midpoint_relation;
mod offset_dimension;
pub use midpoint_points::{
    PrsDimMidPointAttach, points_on_conic, points_on_line, points_on_segment,
};
pub use midpoint_relation::{PrsDimMidPointFields, PrsDimMidPointRelation};
mod symmetric_relation;
mod symmetric_selection;
mod tangent_relation;
pub use ellipse_radius_dimension::{PrsDimEllipseRadiusDimension, PrsDimEllipseRadiusFields};
pub use max_min_radius_dimension::{PrsDimMaxRadiusDimension, PrsDimMinRadiusDimension};
pub use offset_dimension::{PrsDimOffsetDimension, PrsDimOffsetFields};
pub use symmetric_relation::{PrsDimSymmetricFields, PrsDimSymmetricRelation};
pub use tangent_relation::{PrsDimTangentFields, PrsDimTangentRelation};
mod parallel_relation;
mod perpendicular_relation;
pub use concentric_relation::{PrsDimConcentricFields, PrsDimConcentricRelation};
pub use equal_radius_relation::{PrsDimEqualRadiusFields, PrsDimEqualRadiusRelation};
pub use parallel_relation::{PrsDimParallelFields, PrsDimParallelRelation};
pub use perpendicular_relation::{PrsDimPerpendicularFields, PrsDimPerpendicularRelation};
mod relation;
mod relation_support;
pub use chamf_dimension::{PrsDimChamf2dDimension, PrsDimChamf3dDimension, PrsDimChamfFields};
pub use relation::{
    PrsDimRelationBase, PrsDimRelationFields, PrsDimRelationMembers, PrsDimRelationSource,
    prs_dim_relation_set_color, prs_dim_relation_unset_color,
};

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
