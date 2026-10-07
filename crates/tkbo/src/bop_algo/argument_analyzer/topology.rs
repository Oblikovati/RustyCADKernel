//! The topological tests of `BOPAlgo_ArgumentAnalyzer`: argument types, self-interference,
//! micro edges and face rebuilding.
use super::BOPAlgoArgumentAnalyzer;
use crate::bop_algo::{BOPAlgoCheckerSI, BuilderFace, CheckResult, CheckStatus, Operation};
use crate::bop_tools::{dimensions, is_micro_edge};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tktopalgo::brep_extrema::BRepExtremaDistShapeShape;

impl BOPAlgoArgumentAnalyzer {
    /// A bad type for no argument, for a single argument that is empty or meant for an
    /// operation, for an empty argument of two, and for dimensions the operation refuses
    /// (`TestTypes`).
    pub(super) fn test_types(&mut self) -> Result<(), Failure> {
        let (null1, null2) = (self.shape1.is_null(), self.shape2.is_null());
        if null1 && null2 {
            self.bad_type(None, None);
            return Ok(());
        }
        if null1 != null2 {
            let empty = if null1 { self.empty2 } else { self.empty1 };
            if empty || self.operation != Operation::Unknown {
                let shape = if null1 {
                    self.shape2.clone()
                } else {
                    self.shape1.clone()
                };
                self.bad_type(Some(&shape), None);
            }
            return Ok(());
        }
        if self.empty1 || self.empty2 {
            let (first, second) = (self.shape1.clone(), self.shape2.clone());
            let first = self.empty1.then_some(&first);
            let second = self.empty2.then_some(&second);
            self.bad_type(first, second);
            return Ok(());
        }
        if self.operation != Operation::Unknown
            && self.operation != Operation::Common
            && self.bad_dimensions()?
        {
            let (first, second) = (self.shape1.clone(), self.shape2.clone());
            self.bad_type(Some(&first), Some(&second));
        }
        Ok(())
    }

    /// Whether the operation refuses the dimensions of the two arguments.
    fn bad_dimensions(&self) -> Result<bool, Failure> {
        let (mut min1, mut max1, mut min2, mut max2) = (0, 0, 0, 0);
        dimensions(&self.shape1, &mut min1, &mut max1)?;
        dimensions(&self.shape2, &mut min2, &mut max2)?;
        Ok(match self.operation {
            Operation::Fuse => min1 != max1 || min2 != max2 || min1 != min2,
            Operation::Cut => max1 > min2,
            Operation::Cut21 => min1 < max2,
            _ => false,
        })
    }

    /// Appends a bad-type fault naming the given arguments.
    fn bad_type(&mut self, first: Option<&Shape>, second: Option<&Shape>) {
        let mut result = CheckResult::new();
        if let Some(shape) = first {
            result.set_shape1(shape);
        }
        if let Some(shape) = second {
            result.set_shape2(shape);
        }
        result.set_check_status(CheckStatus::BadType);
        self.result.append(result);
    }

    /// The interfering sub-shape pairs of each non-empty argument, plus an aborted operation
    /// when the checker fails (`TestSelfInterferences`).
    pub(super) fn test_self_interferences(&mut self) -> Result<(), Failure> {
        for index in 0..2 {
            let empty = if index == 0 { self.empty1 } else { self.empty2 };
            let shape = self.argument(index).clone();
            if shape.is_null() || empty {
                continue;
            }
            let checker = self.self_interference_checker(&shape);
            self.add_self_interferences(index, &checker)?;
            if checker.options().has_errors() {
                self.add_fault(index, &[&shape], CheckStatus::OperationAborted);
            }
        }
        Ok(())
    }

    /// The performed non-destructive checker of `shape` with the analyzer options.
    fn self_interference_checker(&self, shape: &Shape) -> BOPAlgoCheckerSI {
        let mut checker = BOPAlgoCheckerSI::new();
        checker.set_arguments(&[shape.clone()].into_iter().collect::<List<Shape>>());
        checker.set_non_destructive(true);
        checker
            .options_mut()
            .set_run_parallel(self.options.run_parallel());
        checker
            .options_mut()
            .set_fuzzy_value(self.options.fuzzy_value());
        checker.perform();
        checker
    }

    /// One self-intersection fault per interfering pair of source shapes, in table order.
    fn add_self_interferences(
        &mut self,
        index: usize,
        checker: &BOPAlgoCheckerSI,
    ) -> Result<(), Failure> {
        let structure = checker.data_structure().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "BOPAlgo_ArgumentAnalyzer: the checker has no data structure; expected Perform to create it",
                "",
            )
        })?;
        for pair in structure.interferences().iter() {
            let (first, second) = pair.indices();
            if structure.is_new_shape(first) || structure.is_new_shape(second) {
                continue;
            }
            let (first, second) = (structure.shape(first)?, structure.shape(second)?);
            let faulty: Vec<&Shape> = if first.is_same(second) {
                vec![first]
            } else {
                vec![first, second]
            };
            self.add_fault(index, &faulty, CheckStatus::SelfIntersect);
        }
        Ok(())
    }

    /// The micro edges of each argument; for a section, only the ones with a vertex on the
    /// other argument (`TestSmallEdge`).
    pub(super) fn test_small_edge(&mut self) -> Result<(), Failure> {
        let context = Handle::new(Context::default());
        // One distance tool for the whole test, as natively.
        let mut distance = BRepExtremaDistShapeShape::new();
        for index in 0..2 {
            let shape = self.argument(index).clone();
            if shape.is_null() {
                continue;
            }
            for edge in sub_shapes(&shape, ShapeEnum::Edge)? {
                let edge = topods::edge(&edge)?;
                if BRepTool::degenerated(&edge)? || !is_micro_edge(&edge, &context, true)? {
                    continue;
                }
                if !self.keeps_small_edge(index, &edge, &mut distance)? {
                    continue;
                }
                self.add_fault(index, &[&edge], CheckStatus::TooSmallEdge);
                if self.stop_on_first {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// For a section with two arguments, whether a vertex of the micro edge lies on the other
    /// argument within the summed tolerances.
    fn keeps_small_edge(
        &self,
        index: usize,
        edge: &Edge,
        distance: &mut BRepExtremaDistShapeShape,
    ) -> Result<bool, Failure> {
        let other = self.argument(1 - index);
        if self.operation != Operation::Section || other.is_null() {
            return Ok(true);
        }
        distance.load_s2(other)?;
        for vertex in sub_shapes(edge, ShapeEnum::Vertex)? {
            distance.load_s1(&vertex)?;
            distance.perform(&ProgressRange::default())?;
            if distance.is_done() && vertex_touches(distance, &vertex)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The faces that the face builder cannot rebuild into one face with all their edges;
    /// a section or an unknown operation skips the test (`TestRebuildFace`).
    pub(super) fn test_rebuild_face(&mut self) -> Result<(), Failure> {
        if matches!(self.operation, Operation::Section | Operation::Unknown) {
            return Ok(());
        }
        for index in 0..2 {
            let shape = self.argument(index).clone();
            if shape.is_null() {
                continue;
            }
            for face in sub_shapes(&shape, ShapeEnum::Face)? {
                let face = topods::face(&face)?;
                if is_recoverable_face(&face)? {
                    continue;
                }
                self.add_fault(index, &[&face], CheckStatus::NonRecoverableFace);
                if self.stop_on_first {
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}

/// Whether one solution of the computed distance is closer than the vertex tolerance plus the
/// tolerance of its support on the other shape.
fn vertex_touches(distance: &BRepExtremaDistShapeShape, vertex: &Shape) -> Result<bool, Failure> {
    let vertex_tolerance = BRepTool::vertex_tolerance(&topods::vertex(vertex)?)?;
    for solution in 1..=distance.nb_solution() {
        let support = distance.support_on_shape2(solution)?;
        let tolerance = vertex_tolerance + support_tolerance(&support)?;
        if distance.value()? < tolerance {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The tolerance of a vertex, edge or face support; zero for another shape.
fn support_tolerance(support: &Shape) -> Result<f64, Failure> {
    Ok(match support.shape_type()? {
        ShapeEnum::Vertex => BRepTool::vertex_tolerance(&topods::vertex(support)?)?,
        ShapeEnum::Edge => BRepTool::edge_tolerance(&topods::edge(support)?)?,
        ShapeEnum::Face => BRepTool::face_tolerance(&topods::face(support)?)?,
        _ => 0.0,
    })
}

/// Whether the face builder rebuilds the forward face into exactly one face using every edge
/// (an internal edge given in both orientations).
fn is_recoverable_face(face: &Face) -> Result<bool, Failure> {
    let forward = face.as_ref().oriented(Orientation::Forward);
    let mut edges: List<Shape> = List::default();
    let mut start_edges = 0;
    for edge in sub_shapes(&forward, ShapeEnum::Edge)? {
        if edge.orientation() == Orientation::Internal {
            edges.append(edge.oriented(Orientation::Forward));
            edges.append(edge.oriented(Orientation::Reversed));
        } else {
            edges.append(edge);
        }
        start_edges += 1;
    }
    let mut builder = BuilderFace::new();
    builder.set_face(face);
    builder.set_shapes(&edges);
    builder.perform()?;
    let areas = builder.areas();
    if areas.extent() != 1 {
        return Ok(false);
    }
    let Some(area) = areas.first() else {
        return Ok(false);
    };
    Ok(sub_shapes(area, ShapeEnum::Edge)?.len() == start_edges)
}

/// The sub-shapes of `kind` in explorer order.
pub(super) fn sub_shapes(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    let mut found = Vec::new();
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(found)
}
