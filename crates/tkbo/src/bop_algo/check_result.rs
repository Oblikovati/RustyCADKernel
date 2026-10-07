//! `BOPAlgo_CheckStatus` and `BOPAlgo_CheckResult`: one fault found by the argument analyzer.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;

/// `BOPAlgo_CheckStatus`: the kind of fault of a Boolean operation argument.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum CheckStatus {
    /// `BOPAlgo_CheckUnknown` (the native default; also an analysis that failed).
    #[default]
    CheckUnknown = 0,
    /// `BOPAlgo_BadType`: an empty argument or argument dimensions the operation refuses.
    BadType = 1,
    /// `BOPAlgo_SelfIntersect`: sub-shapes of one argument interfere.
    SelfIntersect = 2,
    /// `BOPAlgo_TooSmallEdge`: a micro edge.
    TooSmallEdge = 3,
    /// `BOPAlgo_NonRecoverableFace`: a face that cannot be rebuilt from its edges.
    NonRecoverableFace = 4,
    /// `BOPAlgo_IncompatibilityOfVertex`: a vertex coincides with several of the other argument.
    IncompatibilityOfVertex = 5,
    /// `BOPAlgo_IncompatibilityOfEdge`: an edge overlaps several of the other argument.
    IncompatibilityOfEdge = 6,
    /// `BOPAlgo_IncompatibilityOfFace` (never reported: the face test is not implemented).
    IncompatibilityOfFace = 7,
    /// `BOPAlgo_OperationAborted`: the self-interference check failed.
    OperationAborted = 8,
    /// `BOPAlgo_GeomAbs_C0`: a C0 curve or surface.
    GeomAbsC0 = 9,
    /// `BOPAlgo_InvalidCurveOnSurface`: an edge farther from its face than its tolerance.
    InvalidCurveOnSurface = 10,
    /// `BOPAlgo_NotValid`.
    NotValid = 11,
}

/// One fault: the checked arguments, their faulty sub-shapes, the status and, for an invalid
/// curve on surface, the maximal distance and its parameter.
///
/// ```
/// use rustycad_tkbo::bop_algo::{CheckResult, CheckStatus};
/// let mut result = CheckResult::new();
/// result.set_check_status(CheckStatus::BadType);
/// assert_eq!(result.check_status(), CheckStatus::BadType);
/// assert!(result.faulty_shapes1().is_empty());
/// ```
#[derive(Clone, Default)]
pub struct CheckResult {
    shape1: Shape,
    shape2: Shape,
    status: CheckStatus,
    faulty1: List<Shape>,
    faulty2: List<Shape>,
    max_distance1: f64,
    max_distance2: f64,
    max_parameter1: f64,
    max_parameter2: f64,
}

impl CheckResult {
    /// An unknown fault without shapes and with zero distances (`BOPAlgo_CheckResult()`);
    /// `CheckResult::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the first checked argument (`SetShape1`); `result.set_shape1(&shape)`.
    pub fn set_shape1(&mut self, shape: &Shape) {
        self.shape1 = shape.clone();
    }

    /// Appends a faulty sub-shape of the first argument (`AddFaultyShape1`);
    /// `result.add_faulty_shape1(&edge)`.
    pub fn add_faulty_shape1(&mut self, shape: &Shape) {
        self.faulty1.append(shape.clone());
    }

    /// Sets the second checked argument (`SetShape2`); `result.set_shape2(&shape)`.
    pub fn set_shape2(&mut self, shape: &Shape) {
        self.shape2 = shape.clone();
    }

    /// Appends a faulty sub-shape of the second argument (`AddFaultyShape2`);
    /// `result.add_faulty_shape2(&edge)`.
    pub fn add_faulty_shape2(&mut self, shape: &Shape) {
        self.faulty2.append(shape.clone());
    }

    /// The first checked argument (`GetShape1`); `result.shape1().is_null()`.
    pub fn shape1(&self) -> &Shape {
        &self.shape1
    }

    /// The second checked argument (`GetShape2`); `result.shape2().is_null()`.
    pub fn shape2(&self) -> &Shape {
        &self.shape2
    }

    /// The faulty sub-shapes of the first argument (`GetFaultyShapes1`);
    /// `result.faulty_shapes1().extent()`.
    pub fn faulty_shapes1(&self) -> &List<Shape> {
        &self.faulty1
    }

    /// The faulty sub-shapes of the second argument (`GetFaultyShapes2`);
    /// `result.faulty_shapes2().extent()`.
    pub fn faulty_shapes2(&self) -> &List<Shape> {
        &self.faulty2
    }

    /// Sets the kind of fault (`SetCheckStatus`); `result.set_check_status(CheckStatus::BadType)`.
    pub fn set_check_status(&mut self, status: CheckStatus) {
        self.status = status;
    }

    /// The kind of fault (`GetCheckStatus`); `result.check_status()`.
    pub fn check_status(&self) -> CheckStatus {
        self.status
    }

    /// Sets the maximal edge/face distance of the first argument (`SetMaxDistance1`);
    /// `result.set_max_distance1(0.1)`.
    pub fn set_max_distance1(&mut self, distance: f64) {
        self.max_distance1 = distance;
    }

    /// Sets the maximal edge/face distance of the second argument (`SetMaxDistance2`);
    /// `result.set_max_distance2(0.1)`.
    pub fn set_max_distance2(&mut self, distance: f64) {
        self.max_distance2 = distance;
    }

    /// Sets the edge parameter of the first maximal distance (`SetMaxParameter1`);
    /// `result.set_max_parameter1(0.5)`.
    pub fn set_max_parameter1(&mut self, parameter: f64) {
        self.max_parameter1 = parameter;
    }

    /// Sets the edge parameter of the second maximal distance (`SetMaxParameter2`);
    /// `result.set_max_parameter2(0.5)`.
    pub fn set_max_parameter2(&mut self, parameter: f64) {
        self.max_parameter2 = parameter;
    }

    /// The maximal edge/face distance of the first argument (`GetMaxDistance1`);
    /// `result.max_distance1()`.
    pub fn max_distance1(&self) -> f64 {
        self.max_distance1
    }

    /// The maximal edge/face distance of the second argument (`GetMaxDistance2`);
    /// `result.max_distance2()`.
    pub fn max_distance2(&self) -> f64 {
        self.max_distance2
    }

    /// The edge parameter of the first maximal distance (`GetMaxParameter1`);
    /// `result.max_parameter1()`.
    pub fn max_parameter1(&self) -> f64 {
        self.max_parameter1
    }

    /// The edge parameter of the second maximal distance (`GetMaxParameter2`);
    /// `result.max_parameter2()`.
    pub fn max_parameter2(&self) -> f64 {
        self.max_parameter2
    }
}
