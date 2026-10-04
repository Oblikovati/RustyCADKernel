//! ShapeFix_Wireframe state and topology analysis from OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod gaps;
mod join_3d;
mod merge_faces;
mod merge_free_wires;
mod repair;
mod small_edges;
use super::{FixRoot, FixRootData, ShapeFixRoot};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::{
    topods::{Edge, Shape, Vertex},
    toptools::ShapeMapHasher,
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::{DataMap, List},
    standard::SharedState,
};
/// Face adjacency lists used by wireframe analysis; `ShapeFixWireframeShapeLists::default()`.
pub type ShapeFixWireframeShapeLists = DataMap<Shape, List<Shape>, ShapeMapHasher>;
/// Analyze wireframe defects; `let tool=ShapeFixWireframe::new();`.
#[derive(Default)]
pub struct ShapeFixWireframe {
    parent: ShapeFixRoot,
    state: SharedState<WireframeState>,
}
struct WireframeState {
    shape: Shape,
    drop_small: bool,
    limit_angle: f64,
    gaps: i32,
    small: i32,
    joined_vertices: Vec<(Vertex, Vertex)>,
}
impl Default for WireframeState {
    fn default() -> Self {
        Self {
            shape: Shape::default(),
            drop_small: false,
            limit_angle: -1.,
            gaps: 0,
            small: 0,
            joined_vertices: Vec::new(),
        }
    }
}
impl_transient!(ShapeFixWireframe,"ShapeFix_Wireframe",ShapeFixRoot,parent,interfaces:[dyn FixRoot]);
impl FixRoot for ShapeFixWireframe {
    fn fix_root_data(&self) -> &FixRootData {
        self.parent.fix_root_data()
    }
}
impl ShapeFixWireframe {
    /// Construct an unloaded tool; `ShapeFixWireframe::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Construct around a shape without changing root configuration; `ShapeFixWireframe::from_shape(&shape)`.
    pub fn from_shape(shape: &Shape) -> Self {
        let tool = Self::new();
        tool.load(shape);
        tool
    }
    /// Load a shape and clear both statuses; `tool.load(&shape)`.
    pub fn load(&self, shape: &Shape) {
        self.clear_statuses();
        self.state.write().shape = shape.clone();
    }
    /// Clear both repair statuses; `tool.clear_statuses()`.
    pub fn clear_statuses(&self) {
        let mut state = self.state.write();
        state.gaps = 0;
        state.small = 0;
    }
    /// Return the current wireframe shape; `tool.shape()`.
    pub fn shape(&self) -> Shape {
        self.state.read().shape.clone()
    }
    /// Decode gap repair status; `tool.status_wire_gaps(ShapeExtendStatus::Ok)`.
    pub fn status_wire_gaps(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().gaps, status)
    }
    /// Decode small-edge repair status; `tool.status_small_edges(ShapeExtendStatus::Ok)`.
    pub fn status_small_edges(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().small, status)
    }
    /// Read whether unmergeable small edges may be dropped; `tool.mode_drop_small_edges()`.
    pub fn mode_drop_small_edges(&self) -> bool {
        self.state.read().drop_small
    }
    /// Set whether unmergeable small edges may be dropped; `tool.set_mode_drop_small_edges(true)`.
    pub fn set_mode_drop_small_edges(&self, enabled: bool) {
        self.state.write().drop_small = enabled;
    }
    /// Set the native unconstrained angle value; `tool.set_limit_angle(0.1)`.
    pub fn set_limit_angle(&self, angle: f64) {
        self.state.write().limit_angle = angle;
    }
    /// Read the merge angle limit; `tool.limit_angle()`.
    pub fn limit_angle(&self) -> f64 {
        self.state.read().limit_angle
    }

    pub(super) fn reset_joined_vertices(&self) {
        self.state.write().joined_vertices.clear();
    }

    pub(super) fn record_joined_endpoints(
        &self,
        edge: &Edge,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        let analyzer = crate::shape_analysis::ShapeAnalysisEdge::new();
        let endpoints = (analyzer.first_vertex(edge), analyzer.last_vertex(edge));
        let copies = (
            joined_vertex_copy(&endpoints.0)?,
            joined_vertex_copy(&endpoints.1)?,
        );
        let mut state = self.state.write();
        record_joined_vertex(&mut state.joined_vertices, &endpoints.0, copies.0);
        record_joined_vertex(&mut state.joined_vertices, &endpoints.1, copies.1);
        Ok(())
    }

    pub(super) fn apply_joined_vertex_replacements(
        &self,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        let replacements = std::mem::take(&mut self.state.write().joined_vertices);
        let context = self.context();
        for (source, target) in replacements {
            rustycad_tkbrep::brep_tools::ReShape::replace(
                &*context,
                &source,
                &target.into_shape(),
            )?;
        }
        Ok(())
    }
}

fn record_joined_vertex(replacements: &mut Vec<(Vertex, Vertex)>, source: &Vertex, copy: Vertex) {
    if replacements
        .iter()
        .any(|(original, _)| original.is_same(source))
    {
        return;
    }
    replacements.push((source.clone(), copy));
}

fn joined_vertex_copy(source: &Vertex) -> Result<Vertex, rustycad_tkernel::standard::Failure> {
    rustycad_tkbrep::topods::vertex(&source.empty_copied()?)
}
