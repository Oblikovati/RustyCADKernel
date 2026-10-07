//! `BOPAlgo_ArgumentAnalyzer::TestMergeSubShapes`: sub-shapes of one argument that coincide
//! with several sub-shapes of the other.
use super::BOPAlgoArgumentAnalyzer;
use super::topology::sub_shapes;
use crate::bop_algo::{CheckResult, CheckStatus};
use crate::int_tools::EdgeEdge;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

impl BOPAlgoArgumentAnalyzer {
    /// Vertices of one argument within tolerance of several vertices of the other
    /// (`TestMergeVertex`).
    pub(super) fn test_merge_vertex(&mut self) -> Result<(), Failure> {
        self.test_merge_sub_shapes(ShapeEnum::Vertex)
    }

    /// Edges of one argument overlapping several edges of the other (`TestMergeEdge`).
    pub(super) fn test_merge_edge(&mut self) -> Result<(), Failure> {
        self.test_merge_sub_shapes(ShapeEnum::Edge)
    }

    /// The sub-shapes of `kind` of each argument that coincide with more than one of the other
    /// argument; faces are never compared, as natively (`TestMergeSubShapes`).
    fn test_merge_sub_shapes(&mut self, kind: ShapeEnum) -> Result<(), Failure> {
        let Some(status) = self.merge_status(kind) else {
            return Ok(());
        };
        let first = distinct_sub_shapes(&self.shape1, kind)?;
        let second = distinct_sub_shapes(&self.shape2, kind)?;
        // Each row of flags is computed just before it is reported, as natively, so a stop at
        // the first fault skips the remaining comparisons.
        let mut flags = Vec::with_capacity(first.len());
        for shape1 in &first {
            let row = second
                .iter()
                .map(|shape2| coincide(kind, shape1, shape2))
                .collect::<Result<Vec<bool>, Failure>>()?;
            let matches = selected(&second, &row);
            flags.push(row);
            if matches.len() > 1 && self.add_incompatibility(&[shape1], &matches, status) {
                return Ok(());
            }
        }
        self.report_columns(&first, &second, &flags, status);
        Ok(())
    }

    /// The fault status of merging sub-shapes of `kind`, None when the test does not apply.
    fn merge_status(&self, kind: ShapeEnum) -> Option<CheckStatus> {
        if self.shape1.is_null() || self.shape2.is_null() || self.empty1 || self.empty2 {
            return None;
        }
        match kind {
            ShapeEnum::Vertex => Some(CheckStatus::IncompatibilityOfVertex),
            ShapeEnum::Edge => Some(CheckStatus::IncompatibilityOfEdge),
            ShapeEnum::Face => Some(CheckStatus::IncompatibilityOfFace),
            _ => None,
        }
    }

    /// The sub-shapes of the second argument that coincide with several of the first.
    fn report_columns(
        &mut self,
        first: &[Shape],
        second: &[Shape],
        flags: &[Vec<bool>],
        status: CheckStatus,
    ) {
        for (column, shape2) in second.iter().enumerate() {
            let column_flags: Vec<bool> = flags.iter().map(|row| row[column]).collect();
            let matches = selected(first, &column_flags);
            if matches.len() > 1 && self.add_incompatibility(&matches, &[shape2], status) {
                return;
            }
        }
    }

    /// Appends an incompatibility fault of both arguments; true when the analysis stops at the
    /// first fault.
    fn add_incompatibility(
        &mut self,
        faulty1: &[&Shape],
        faulty2: &[&Shape],
        status: CheckStatus,
    ) -> bool {
        let mut result = CheckResult::new();
        result.set_shape1(&self.shape1);
        result.set_shape2(&self.shape2);
        faulty1
            .iter()
            .for_each(|shape| result.add_faulty_shape1(shape));
        faulty2
            .iter()
            .for_each(|shape| result.add_faulty_shape2(shape));
        result.set_check_status(status);
        self.result.append(result);
        self.stop_on_first
    }
}

/// The shapes whose flag is set.
fn selected<'a>(shapes: &'a [Shape], flags: &[bool]) -> Vec<&'a Shape> {
    shapes
        .iter()
        .zip(flags)
        .filter(|(_, flag)| **flag)
        .map(|(shape, _)| shape)
        .collect()
}

/// The sub-shapes of `kind` in explorer order, each once.
fn distinct_sub_shapes(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut seen = MapOfShape::default();
    Ok(sub_shapes(shape, kind)?
        .into_iter()
        .filter(|sub_shape| seen.add(sub_shape.clone()))
        .collect())
}

/// Vertices within the sum of their tolerances, or edges with a common edge part.
fn coincide(kind: ShapeEnum, first: &Shape, second: &Shape) -> Result<bool, Failure> {
    match kind {
        ShapeEnum::Vertex => {
            let (first, second) = (topods::vertex(first)?, topods::vertex(second)?);
            let distance = BRepTool::pnt(&first)?.distance(&BRepTool::pnt(&second)?);
            Ok(distance
                <= BRepTool::vertex_tolerance(&first)? + BRepTool::vertex_tolerance(&second)?)
        }
        ShapeEnum::Edge => {
            let mut intersector =
                EdgeEdge::with_edges(&topods::edge(first)?, &topods::edge(second)?);
            intersector.perform()?;
            Ok(intersector.is_done()
                && intersector
                    .common_parts()
                    .iter()
                    .any(|part| part.shape_type() == ShapeEnum::Edge))
        }
        // Face coincidence is not implemented natively.
        _ => Ok(false),
    }
}
