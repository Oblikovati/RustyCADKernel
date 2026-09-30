//! Native reuse of existing polygon nodes and their face parameters.
use super::{EdgeParameterProvider, ShapeTool};
use crate::{
    imesh_data::{EdgeHandle, FaceHandle},
    imesh_tools::CurveTessellator,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, StandardTransient, WeakHandle},
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{
    gp::Point3,
    poly::{PolygonOnTriangulation, Triangulation, mesh_purpose},
    toploc::Location,
};
/// Existing edge polygon as a curve tessellator; `EdgeTessellationExtractor::new(&edge, &face)?`.
pub struct EdgeTessellationExtractor {
    transient: StandardTransient,
    provider: EdgeParameterProvider,
    triangulation: WeakHandle<Triangulation>,
    polygon: WeakHandle<PolygonOnTriangulation>,
    location: Location,
}
impl_transient!(EdgeTessellationExtractor,"BRepMesh_EdgeTessellationExtractor",dyn CurveTessellator,transient,interfaces:[dyn CurveTessellator]);
impl EdgeTessellationExtractor {
    /// Retains native non-owning mesh/node associations; `EdgeTessellationExtractor::new(&edge, &face)?`.
    pub fn new(edge: &EdgeHandle, face: &FaceHandle) -> Result<Self, Failure> {
        let (triangulation, location) =
            BRepTool::triangulation(&face.get_face()?, mesh_purpose::NONE)?;
        let polygon =
            BRepTool::polygon_on_triangulation(&edge.get_edge()?, &triangulation, &location)?;
        let parameters = polygon.get().ok_or_else(missing_polygon)?.parameters();
        Ok(Self {
            transient: StandardTransient::default(),
            provider: EdgeParameterProvider::with_edge(
                edge,
                Orientation::Forward,
                face,
                parameters,
            )?,
            triangulation: triangulation.downgrade(),
            polygon: polygon.downgrade(),
            location,
        })
    }
}
impl CurveTessellator for EdgeTessellationExtractor {
    fn points_nb(&self) -> i32 {
        self.polygon
            .upgrade()
            .get()
            .expect("expired polygon; expected live edge polygon")
            .nb_nodes()
    }
    fn value(&self, index: i32, point: &mut Point3, parameter: &mut f64) -> Result<bool, Failure> {
        let polygon = self.polygon.upgrade();
        let node = polygon.get().ok_or_else(missing_polygon)?.node(index)?;
        let triangulation = self.triangulation.upgrade();
        let reference = triangulation
            .get()
            .ok_or_else(missing_polygon)?
            .node(node)?;
        *point = ShapeTool::use_location(reference, &self.location);
        *parameter = self.provider.parameter(index, *point)?;
        Ok(true)
    }
}
fn missing_polygon() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "null or expired polygon/triangulation; expected stored edge tessellation",
    )
}
