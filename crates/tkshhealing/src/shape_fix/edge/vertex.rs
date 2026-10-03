use super::ShapeFixEdge;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

impl ShapeFixEdge {
    /// Repair tolerances against all stored curves; `tool.fix_vertex_tolerance(&edge)?`.
    pub fn fix_vertex_tolerance(&self, edge: &Edge) -> Result<bool, Failure> {
        self.repair_vertex_tolerances(edge, None)
    }

    /// Repair tolerances using one face; `tool.fix_vertex_tolerance_on_face(&edge, &face)?`.
    pub fn fix_vertex_tolerance_on_face(&self, edge: &Edge, face: &Face) -> Result<bool, Failure> {
        self.repair_vertex_tolerances(edge, Some(face))
    }

    fn repair_vertex_tolerances(&self, edge: &Edge, face: Option<&Face>) -> Result<bool, Failure> {
        self.reset_status();
        let Some(edge) = self.context_edge(edge)? else {
            return Ok(false);
        };
        let mut analyzer = ShapeAnalysisEdge::new();
        let (changed, first, last) = match face {
            Some(face) => analyzer.check_vertex_tolerance_on_face(&edge, face)?,
            None => analyzer.check_vertex_tolerance(&edge)?,
        };
        if !changed {
            return Ok(false);
        }
        self.vertex_status(&analyzer);
        self.update_vertex(&analyzer.first_vertex(&edge), first)?;
        self.update_vertex(&analyzer.last_vertex(&edge), last)?;
        Ok(true)
    }

    fn vertex_status(&self, analyzer: &ShapeAnalysisEdge) {
        for flag in [ShapeExtendStatus::Done1, ShapeExtendStatus::Done2] {
            if analyzer.status_flag(flag) {
                // Native assignment lets DONE2 replace DONE1 when both vertices need repair.
                self.set_status(ShapeExtend::encode_status(flag));
            }
        }
    }

    fn context_edge(&self, edge: &Edge) -> Result<Option<Edge>, Failure> {
        let context_handle = self.context();
        let Some(context) = context_handle.get() else {
            return Ok(Some(edge.clone()));
        };
        let applied = context.apply(edge, ShapeEnum::Shape)?;
        if applied.is_null() || applied.shape_type()? != ShapeEnum::Edge {
            return Ok(None);
        }
        Ok(Some(Edge::unchecked(applied)))
    }

    fn update_vertex(&self, vertex: &Vertex, tolerance: f64) -> Result<(), Failure> {
        let context_handle = self.context();
        if let Some(context) = context_handle.get() {
            context.copy_vertex(vertex, tolerance)?;
            return Ok(());
        }
        BRepBuilder::default().update_vertex_tolerance(vertex, tolerance)
    }
}
