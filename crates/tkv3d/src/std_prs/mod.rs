//! Martensite standard presentations, translated from OCCT 3d097a0.
mod curve;
mod curve_draw;
mod curve_limits;
mod curve_match;
pub use curve::StdPrsCurve;
mod point;
pub use point::{StdPrsPoint, StdPrsToolPoint};
mod deflection_curve;
mod deflection_match;
mod deflection_queries;
mod deflection_sampling;
pub use deflection_curve::StdPrsDeflectionCurve;

mod plane;
mod plane_arrows;
mod plane_geometry;
pub use plane::StdPrsPlane;

mod shaded_shape_indices;
mod shaded_shape_triangles;
mod shaded_shape_vertices;
mod triangulated_shape;
pub use shaded_shape_triangles::StdPrsShadedShape;
pub use triangulated_shape::{
    StdPrsNativeShapeMesher, StdPrsShapeMesher, StdPrsToolTriangulatedShape,
};

mod wf_shape_edges;
mod wf_shape_vertices;
pub use wf_shape_edges::StdPrsWfShape;

mod shaded_shape_volumes;
mod shape_planarity;
pub use shaded_shape_volumes::StdPrsVolume;
pub use shape_planarity::StdPrsShapeTool;

mod shaded_shape_boundaries;
