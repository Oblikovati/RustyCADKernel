//! Concrete BRepMesh algorithms and topology support.
mod shape_tool;
pub use shape_tool::ShapeTool;

mod model_builder;
mod shape_visitor;
pub use model_builder::BRepMeshModelBuilder;
pub use shape_visitor::BRepMeshShapeVisitor;

mod deflection;
pub use deflection::Deflection;

mod post_processor;
pub use post_processor::BRepMeshModelPostProcessor;

mod edge_parameter;
pub use edge_parameter::{EdgeParameterCollection, EdgeParameterProvider};

mod tessellation_extractor;
pub use tessellation_extractor::EdgeTessellationExtractor;

mod curve_tessellator;
pub use curve_tessellator::BRepMeshCurveTessellator;

mod edge_discret;
pub use edge_discret::BRepMeshEdgeDiscret;

mod range_splitter;
pub use range_splitter::{ConeRangeSplitter, DefaultRangeSplitter, RangeSplitter, SurfaceNodes};

mod pre_processor;
pub use pre_processor::BRepMeshModelPreProcessor;

mod geom_tool;
pub use geom_tool::{GeomTool, IntersectionFlag};

mod face_checker;
pub use face_checker::{
    BRepMeshFaceChecker, CurvePointReference, FaceBoundsTree, FaceBoundsTreeArray,
    FaceCheckerParameters, FaceEdgeSet, FaceEdgeSetArray, FaceSegment, FaceSegmentArray,
    FaceSegments,
};

mod model_healer;
pub use model_healer::BRepMeshModelHealer;
