//! `BOPAlgo_ArgumentAnalyzer`: checks one or two shapes for faults that make them invalid
//! arguments of a Boolean operation.
#[path = "argument_analyzer/geometry.rs"]
mod geometry;
#[path = "argument_analyzer/merge.rs"]
mod merge;
#[path = "argument_analyzer/topology.rs"]
mod topology;
use super::{CheckResult, CheckStatus, Operation, Options};
use crate::bop_tools::algo_tools3d::is_empty_shape;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;

/// The analyzer of Boolean arguments: each enabled test appends its faults to the check
/// result.
///
/// ```
/// use rustycad_tkbo::bop_algo::{BOPAlgoArgumentAnalyzer, CheckStatus};
/// let mut analyzer = BOPAlgoArgumentAnalyzer::new();
/// *analyzer.argument_type_mode() = true;
/// analyzer.perform();
/// // No argument at all is a bad type.
/// assert_eq!(analyzer.check_result().first().unwrap().check_status(), CheckStatus::BadType);
/// ```
#[derive(Default)]
pub struct BOPAlgoArgumentAnalyzer {
    options: Options,
    shape1: Shape,
    shape2: Shape,
    stop_on_first: bool,
    operation: Operation,
    modes: AnalysisModes,
    empty1: bool,
    empty2: bool,
    result: List<CheckResult>,
}

/// The enabled tests, all off by default.
#[derive(Clone, Copy, Default)]
struct AnalysisModes {
    argument_type: bool,
    self_interference: bool,
    small_edge: bool,
    rebuild_face: bool,
    tangent: bool,
    merge_vertex: bool,
    merge_edge: bool,
    continuity: bool,
    curve_on_surface: bool,
}

/// One test of `Perform`, run when its mode is on and no fault stops the analysis.
type AnalysisTest = fn(&mut BOPAlgoArgumentAnalyzer) -> Result<(), Failure>;

impl BOPAlgoArgumentAnalyzer {
    /// An analyzer with every test off and an unknown operation (`BOPAlgo_ArgumentAnalyzer()`);
    /// `BOPAlgoArgumentAnalyzer::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The algorithm options (`BOPAlgo_Options` base); `analyzer.options().fuzzy_value()`.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Mutable algorithm options; `analyzer.options_mut().set_run_parallel(false)`.
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }

    /// Sets the first argument (`SetShape1`); `analyzer.set_shape1(&shape)`.
    pub fn set_shape1(&mut self, shape: &Shape) {
        self.shape1 = shape.clone();
    }

    /// Sets the second argument (`SetShape2`); `analyzer.set_shape2(&shape)`.
    pub fn set_shape2(&mut self, shape: &Shape) {
        self.shape2 = shape.clone();
    }

    /// The first argument (`GetShape1`); `analyzer.shape1().is_null()`.
    pub fn shape1(&self) -> &Shape {
        &self.shape1
    }

    /// The second argument (`GetShape2`); `analyzer.shape2().is_null()`.
    pub fn shape2(&self) -> &Shape {
        &self.shape2
    }

    /// The operation the arguments are meant for (`OperationType`);
    /// `*analyzer.operation_type() = Operation::Fuse`.
    pub fn operation_type(&mut self) -> &mut Operation {
        &mut self.operation
    }

    /// Whether the tests stop at the first fault (`StopOnFirstFaulty`);
    /// `*analyzer.stop_on_first_faulty() = true`.
    pub fn stop_on_first_faulty(&mut self) -> &mut bool {
        &mut self.stop_on_first
    }

    /// Whether the argument types are tested (`ArgumentTypeMode`);
    /// `*analyzer.argument_type_mode() = true`.
    pub fn argument_type_mode(&mut self) -> &mut bool {
        &mut self.modes.argument_type
    }

    /// Whether the arguments are checked on self-interference (`SelfInterMode`);
    /// `*analyzer.self_inter_mode() = true`.
    pub fn self_inter_mode(&mut self) -> &mut bool {
        &mut self.modes.self_interference
    }

    /// Whether micro edges are searched (`SmallEdgeMode`); `*analyzer.small_edge_mode() = true`.
    pub fn small_edge_mode(&mut self) -> &mut bool {
        &mut self.modes.small_edge
    }

    /// Whether the faces are rebuilt from their edges (`RebuildFaceMode`);
    /// `*analyzer.rebuild_face_mode() = true`.
    pub fn rebuild_face_mode(&mut self) -> &mut bool {
        &mut self.modes.rebuild_face
    }

    /// Whether tangencies are tested; the native test does nothing (`TangentMode`);
    /// `*analyzer.tangent_mode() = true`.
    pub fn tangent_mode(&mut self) -> &mut bool {
        &mut self.modes.tangent
    }

    /// Whether coinciding vertices of the two arguments are searched (`MergeVertexMode`);
    /// `*analyzer.merge_vertex_mode() = true`.
    pub fn merge_vertex_mode(&mut self) -> &mut bool {
        &mut self.modes.merge_vertex
    }

    /// Whether overlapping edges of the two arguments are searched (`MergeEdgeMode`);
    /// `*analyzer.merge_edge_mode() = true`.
    pub fn merge_edge_mode(&mut self) -> &mut bool {
        &mut self.modes.merge_edge
    }

    /// Whether C0 curves and surfaces are searched (`ContinuityMode`);
    /// `*analyzer.continuity_mode() = true`.
    pub fn continuity_mode(&mut self) -> &mut bool {
        &mut self.modes.continuity
    }

    /// Whether edges farther from their faces than their tolerance are searched
    /// (`CurveOnSurfaceMode`); `*analyzer.curve_on_surface_mode() = true`.
    pub fn curve_on_surface_mode(&mut self) -> &mut bool {
        &mut self.modes.curve_on_surface
    }

    /// Runs the enabled tests in native order: types, self-interference, small edges, face
    /// rebuilding, tangency, vertex and edge merging, continuity, curves on surfaces; with
    /// `stop_on_first_faulty` a test after the first fault is skipped. A failing test appends
    /// an unknown fault, as the native catch does (`Perform`); `analyzer.perform()`.
    pub fn perform(&mut self) {
        self.result.clear();
        if self.run_tests().is_err() {
            self.result.append(CheckResult::new());
        }
    }

    /// Whether a fault was found (`HasFaulty`); `analyzer.has_faulty()`.
    pub fn has_faulty(&self) -> bool {
        !self.result.is_empty()
    }

    /// The faults found (`GetCheckResult`); `analyzer.check_result().extent()`.
    pub fn check_result(&self) -> &List<CheckResult> {
        &self.result
    }

    fn run_tests(&mut self) -> Result<(), Failure> {
        self.prepare()?;
        let modes = self.modes;
        if modes.argument_type {
            self.test_types()?;
        }
        if modes.self_interference {
            self.test_self_interferences()?;
        }
        let tests: [(bool, AnalysisTest); 7] = [
            (modes.small_edge, Self::test_small_edge),
            (modes.rebuild_face, Self::test_rebuild_face),
            (modes.tangent, Self::test_tangent),
            (modes.merge_vertex, Self::test_merge_vertex),
            (modes.merge_edge, Self::test_merge_edge),
            (modes.continuity, Self::test_continuity),
            (modes.curve_on_surface, Self::test_curve_on_surface),
        ];
        for (enabled, test) in tests {
            if enabled && (self.result.is_empty() || !self.stop_on_first) {
                test(self)?;
            }
        }
        Ok(())
    }

    /// Notes which arguments are empty (`Prepare`).
    fn prepare(&mut self) -> Result<(), Failure> {
        if !self.shape1.is_null() {
            self.empty1 = is_empty_shape(&self.shape1)?;
        }
        if !self.shape2.is_null() {
            self.empty2 = is_empty_shape(&self.shape2)?;
        }
        Ok(())
    }

    /// Tangency is not tested natively (`TestTangent`).
    fn test_tangent(&mut self) -> Result<(), Failure> {
        Ok(())
    }

    /// The argument of `index` (0 or 1).
    fn argument(&self, index: usize) -> &Shape {
        if index == 0 {
            &self.shape1
        } else {
            &self.shape2
        }
    }

    /// Appends a fault of one argument with its faulty sub-shapes.
    fn add_fault(
        &mut self,
        index: usize,
        faulty: &[&Shape],
        status: CheckStatus,
    ) -> &mut CheckResult {
        let mut result = CheckResult::new();
        if index == 0 {
            result.set_shape1(&self.shape1);
            faulty
                .iter()
                .for_each(|shape| result.add_faulty_shape1(shape));
        } else {
            result.set_shape2(&self.shape2);
            faulty
                .iter()
                .for_each(|shape| result.add_faulty_shape2(shape));
        }
        result.set_check_status(status);
        self.result.append(result)
    }
}
