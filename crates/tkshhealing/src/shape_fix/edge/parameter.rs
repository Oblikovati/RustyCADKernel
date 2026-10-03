use super::{ShapeFixEdge, range::temporary_same_range};
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildEdge;
use crate::shape_extend::ShapeExtendStatus;
use crate::shape_fix::ShapeFixShapeTolerance;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_lib::same_parameter_edge;

struct OriginalEdgeTolerances {
    vertices: [(Vertex, f64); 2],
    edge: f64,
    was_same_parameter: bool,
}

impl OriginalEdgeTolerances {
    fn capture(edge: &Edge) -> Result<Self, Failure> {
        let analyzer = ShapeAnalysisEdge::new();
        let vertices = [analyzer.first_vertex(edge), analyzer.last_vertex(edge)];
        Ok(Self {
            vertices: [
                vertex_tolerance(vertices[0].clone())?,
                vertex_tolerance(vertices[1].clone())?,
            ],
            edge: BRepTool::edge_tolerance(edge)?,
            was_same_parameter: BRepTool::same_parameter(edge)?,
        })
    }

    fn restore_vertices(&self, deviation: f64) -> Result<(), Failure> {
        for (vertex, tolerance) in &self.vertices {
            if !vertex.is_null() {
                let required = if deviation < *tolerance {
                    *tolerance
                } else {
                    deviation
                };
                ShapeFixShapeTolerance::new().set_tolerance(vertex, required, ShapeEnum::Vertex)?;
            }
        }
        Ok(())
    }
}

fn vertex_tolerance(vertex: Vertex) -> Result<(Vertex, f64), Failure> {
    let tolerance = if vertex.is_null() {
        0.0
    } else {
        BRepTool::vertex_tolerance(&vertex)?
    };
    Ok((vertex, tolerance))
}

impl ShapeFixEdge {
    /// Choose native repair or deviation-only tolerance growth; `tool.fix_same_parameter(&edge, 0.0)?`.
    pub fn fix_same_parameter(&mut self, edge: &Edge, tolerance: f64) -> Result<bool, Failure> {
        self.fix_same_parameter_on_face(edge, &Face::default(), tolerance)
    }

    /// Limit deviation checks to one face when already same-parameter;
    /// `tool.fix_same_parameter_on_face(&edge, &face, 0.0)?`.
    pub fn fix_same_parameter_on_face(
        &mut self,
        edge: &Edge,
        face: &Face,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        self.status = 0;
        if BRepTool::degenerated(edge)? {
            return repair_degenerated(edge);
        }
        let original = OriginalEdgeTolerances::capture(edge)?;
        let repaired = self.try_parameter_copy(edge, tolerance, &original);
        BRepBuilder::default().same_parameter(edge, true)?;
        let checked_face = if original.was_same_parameter {
            face.clone()
        } else {
            Face::default()
        };
        let mut analyzer = ShapeAnalysisEdge::new();
        let (_, mut deviation) = analyzer.check_same_parameter_on_face(edge, &checked_face, 23)?;
        self.record_analysis_failure(&analyzer);
        self.choose_parameter_copy(edge, &repaired, &mut analyzer, &mut deviation)?;
        original.restore_vertices(deviation)?;
        self.finish_parameter_repair(edge, &original, repaired.is_some(), deviation)
    }

    fn try_parameter_copy(
        &mut self,
        edge: &Edge,
        tolerance: f64,
        original: &OriginalEdgeTolerances,
    ) -> Option<Edge> {
        match parameter_copy(edge, tolerance, original) {
            Ok(result) => result,
            Err(_) => {
                self.add_status(ShapeExtendStatus::Fail2);
                None
            }
        }
    }

    fn record_analysis_failure(&mut self, analyzer: &ShapeAnalysisEdge) {
        if analyzer.status_flag(ShapeExtendStatus::Fail2) {
            self.add_status(ShapeExtendStatus::Fail1);
        }
    }

    fn choose_parameter_copy(
        &mut self,
        edge: &Edge,
        repaired: &Option<Edge>,
        analyzer: &mut ShapeAnalysisEdge,
        deviation: &mut f64,
    ) -> Result<(), Failure> {
        let Some(copy) = repaired else { return Ok(()) };
        let mut tolerance = BRepTool::edge_tolerance(copy)?;
        let (_, measured) = analyzer.check_same_parameter(copy, 23)?;
        self.add_status(ShapeExtendStatus::Done3);
        if tolerance < measured {
            tolerance = measured;
        }
        if tolerance < *deviation {
            self.record_analysis_failure(analyzer);
            ShapeBuildEdge.copy_pcurves(edge, copy)?;
            *deviation = tolerance;
            ShapeFixShapeTolerance::new().set_tolerance(edge, tolerance, ShapeEnum::Edge)?;
            self.add_status(ShapeExtendStatus::Done5);
        }
        Ok(())
    }

    fn finish_parameter_repair(
        &mut self,
        edge: &Edge,
        original: &OriginalEdgeTolerances,
        repaired: bool,
        deviation: f64,
    ) -> Result<bool, Failure> {
        if deviation > original.edge {
            self.add_status(ShapeExtendStatus::Done1);
            BRepBuilder::default().update_edge_tolerance(edge, deviation)?;
            self.fix_vertex_tolerance(edge)?;
        }
        if !original.was_same_parameter && !repaired {
            self.add_status(ShapeExtendStatus::Done2);
        }
        Ok(self.status_flag(ShapeExtendStatus::Done))
    }
}

fn repair_degenerated(edge: &Edge) -> Result<bool, Failure> {
    if !BRepTool::same_range(edge)? {
        temporary_same_range(edge, precision::p_confusion())?;
    }
    BRepBuilder::default().same_parameter(edge, true)?;
    Ok(false)
}

fn parameter_copy(
    edge: &Edge,
    tolerance: f64,
    original: &OriginalEdgeTolerances,
) -> Result<Option<Edge>, Failure> {
    if !BRepTool::same_range(edge)? {
        temporary_same_range(edge, precision::p_confusion())?;
    }
    if original.was_same_parameter {
        return Ok(None);
    }
    let copy = ShapeBuildEdge.copy(edge, false)?;
    let builder = BRepBuilder::default();
    builder.same_parameter(&copy, false)?;
    let (first, last) = BRepTool::range(edge)?;
    builder.range(&copy, first, last, true)?;
    let tolerance = if tolerance >= precision::confusion() {
        tolerance
    } else {
        original.edge
    };
    same_parameter_edge(&copy, tolerance)?;
    if !BRepTool::same_parameter(&copy)? {
        return Err(Failure::with_message(
            "Standard_Failure",
            "same-parameter copy flag=false; expected true",
        ));
    }
    Ok(Some(copy))
}
