//! Native reconstruction of wires branching through repeated vertices.
mod graph;
mod join;
use super::{ShapeExtend, ShapeExtendStatus, ShapeFixFace};
use crate::{
    shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire},
    shape_extend::ShapeExtendWireData,
    shape_fix::ShapeFixWire,
};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{self, Shape, Wire},
};
use rustycad_tkernel::{ncollection::Sequence, precision, standard::Failure};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::Orientation};
use rustycad_tkmath::geom_abs::SurfaceType;
impl ShapeFixFace {
    /// Append reconstructed loop wires in OCCT order; `tool.fix_loop_wire(&mut wires)?`.
    pub fn fix_loop_wire(&self, output: &mut Sequence<Shape>) -> Result<bool, Failure> {
        let Some(mut graph) = graph::LoopGraph::analyze(self)? else {
            return Ok(false);
        };
        let mut open = graph.collect_paths(output)?;
        join::join_open_paths(&mut open, output, self)?;
        Ok(!output.is_empty() && self.loop_outputs_closed_2d(output)?)
    }
    fn loop_outputs_closed_2d(&self, output: &Sequence<Shape>) -> Result<bool, Failure> {
        let analysis = self.analysis_surface.read().clone();
        if analysis.is_null() || analysis.adaptor3d().get_type()? == SurfaceType::Plane {
            return Ok(true);
        }
        let mut face = topods::face(&self.face()?.empty_copied()?)?;
        face.set_orientation(Orientation::Forward);
        for shape in output {
            if !closed_in_pcurve(&face, &topods::wire(shape)?)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
fn reorder_wire(wire: &Wire) -> Result<Wire, Failure> {
    let fixer = ShapeFixWire::new();
    fixer.load(wire)?;
    fixer.fix_reorder()?;
    fixer.wire()
}
fn closed_in_pcurve(face: &topods::Face, wire: &Wire) -> Result<bool, Failure> {
    let analysis = ShapeAnalysisWire::from_wire(wire, face, precision::confusion())?;
    for index in 1..=analysis.nb_edges() {
        let edge = analysis.wire_data().edge(index)?;
        let vertex = ShapeAnalysisEdge::new().first_vertex(&edge);
        analysis.set_precision(BRepTool::vertex_tolerance(&vertex)?);
        analysis.check_gap_2d_at(index)?;
        if !analysis.last_check_status(ShapeExtendStatus::Ok) {
            return Ok(false);
        }
    }
    Ok(true)
}
