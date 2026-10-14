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

mod hlr_shape;
mod pole_presentations;
mod shaded_shape_volumes;
mod shape_planarity;
mod wf_restricted_face;
mod wf_surface;
/// `StdPrs_BndBox`: the bounding box presentation, an alias of `Prs3d_BndBox`.
pub type StdPrsBndBox = crate::prs3d::Prs3dBndBox;
pub use hlr_shape::{
    StdPrsHlrPolyShape, StdPrsHlrShape, StdPrsHlrShapeI, StdPrsHlrToolShape, std_prs_hlr_projector,
};
pub use pole_presentations::{
    StdPrsPoleCurve, StdPrsShadedSurface, StdPrsToolVertex, StdPrsWfPoleSurface,
};
pub use shaded_shape_volumes::StdPrsVolume;
pub use shape_planarity::StdPrsShapeTool;
pub use wf_restricted_face::{StdPrsWfDeflectionRestrictedFace, StdPrsWfRestrictedFace};
pub use wf_surface::{StdPrsWfDeflectionSurface, StdPrsWfSurface};

mod shaded_shape_boundaries;
mod shaded_shape_presentation;
mod shaded_shape_wireframe;
mod tool_r_face;
mod wf_shape_presentation;
pub use tool_r_face::StdPrsToolRFace;
mod isoline_outputs;
pub use isoline_outputs::{StdPrsIsolineDestination, StdPrsIsolinePolylines};
mod isoline_segments;
pub use isoline_segments::{StdPrsPntOnIso, StdPrsSegOnIso};

mod isolines;
pub use isolines::StdPrsIsolines;
mod isoline_parameters;
pub use isoline_parameters::StdPrsIsolineUvOutputs;

mod isoline_triangle_refs;
pub use isoline_triangle_refs::StdPrsIsolineTriangulationInputs;
mod isoline_segment_sort;
mod isoline_triangle_crossing;
mod isoline_triangle_projection;

mod isoline_triangle_collect;

mod isoline_triangle_api;
mod isoline_triangle_face;

mod isoline_surface_hatch;
mod isoline_surface_trim;

mod isoline_surface_api;
mod isoline_surface_collect;
mod isoline_surface_curves;

mod isoline_surface_source;
pub use isoline_surface_source::StdPrsIsolineSurfaceSource;

mod isoline_face_api;

#[cfg(test)]
mod isoline_face_fixture;
#[cfg(test)]
mod isoline_surface_tests;

#[cfg(test)]
mod isoline_triangle_fixture;
#[cfg(test)]
mod isoline_triangle_tests;

#[cfg(test)]
mod isoline_rface_tests;

#[cfg(test)]
mod isoline_adaptor_fake;
#[cfg(test)]
mod isoline_adaptor_tests;

#[cfg(test)]
mod isoline_alias_surface_fake;
#[cfg(test)]
mod isoline_face_path_tests;
#[cfg(test)]
mod isoline_fault_tests;
#[cfg(test)]
mod isoline_live_input_tests;
#[cfg(test)]
mod isoline_parameter_edge_tests;
#[cfg(test)]
mod isoline_reference_tests;

mod isoline_surface_reference;
#[cfg(test)]
mod isoline_surface_slot_tests;

mod tool_r_face_edge_ref;
pub use tool_r_face_edge_ref::StdPrsToolRFaceEdgeRef;
#[cfg(test)]
mod tool_r_face_edge_tests;
#[cfg(test)]
mod tool_r_face_ownership_tests;
#[cfg(test)]
mod tool_r_face_source_fake;

mod brep_font;
mod brep_font_cache;
mod brep_font_glyph_cache;
mod brep_font_impl;
pub use brep_font::{StdPrsBRepFont, StdPrsBRepFontGlyphOptions, StdPrsBRepFontTextOptions};
pub use brep_font_cache::StdPrsBRepFontCache;
