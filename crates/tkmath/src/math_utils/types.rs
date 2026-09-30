use crate::math::{Matrix, Vector};
/// Native solver outcome; `Status::Ok` denotes a successful computation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    /// Native outcome Ok; `Status::Ok`.
    Ok = 0,
    /// Native outcome NotConverged; `Status::NotConverged`.
    #[default]
    NotConverged = 1,
    /// Native outcome MaxIterations; `Status::MaxIterations`.
    MaxIterations = 2,
    /// Native outcome NumericalError; `Status::NumericalError`.
    NumericalError = 3,
    /// Native outcome InvalidInput; `Status::InvalidInput`.
    InvalidInput = 4,
    /// Native outcome InfiniteSolutions; `Status::InfiniteSolutions`.
    InfiniteSolutions = 5,
    /// Native outcome NoSolution; `Status::NoSolution`.
    NoSolution = 6,
    /// Native outcome NotPositiveDefinite; `Status::NotPositiveDefinite`.
    NotPositiveDefinite = 7,
    /// Native outcome Singular; `Status::Singular`.
    Singular = 8,
    /// Native outcome NonDescentDirection; `Status::NonDescentDirection`.
    NonDescentDirection = 9,
    /// Native outcome CallbackError; `Status::CallbackError`.
    CallbackError = 10,
}
/// Native result with optional computed fields; `ScalarResult::default()`.
#[derive(Default)]
pub struct ScalarResult {
    /// Native status field; `result.status`.
    pub status: Status,
    /// Native iterations field; `result.iterations`.
    pub iterations: u32,
    /// Native root field; `result.root`.
    pub root: Option<f64>,
    /// Native value field; `result.value`.
    pub value: Option<f64>,
    /// Native derivative field; `result.derivative`.
    pub derivative: Option<f64>,
}
impl ScalarResult {
    /// Reports Status::Ok; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&ScalarResult> for bool {
    fn from(result: &ScalarResult) -> Self {
        result.is_done()
    }
}
/// Native result with optional computed fields; `PolyResult::default()`.
#[derive(Default)]
pub struct PolyResult {
    /// Native status field; `result.status`.
    pub status: Status,
    /// Native number of roots field; `result.number_of_roots`.
    pub number_of_roots: usize,
    /// Native roots field; `result.roots`.
    pub roots: [f64; 4],
}
impl PolyResult {
    /// Reports Status::Ok; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&PolyResult> for bool {
    fn from(result: &PolyResult) -> Self {
        result.is_done()
    }
}
/// Native result with optional computed fields; `VectorResult::default()`.
#[derive(Default)]
pub struct VectorResult {
    /// Native status field; `result.status`.
    pub status: Status,
    /// Native iterations field; `result.iterations`.
    pub iterations: u32,
    /// Native solution field; `result.solution`.
    pub solution: Option<Vector<'static>>,
    /// Native value field; `result.value`.
    pub value: Option<f64>,
    /// Native gradient field; `result.gradient`.
    pub gradient: Option<Vector<'static>>,
    /// Native jacobian field; `result.jacobian`.
    pub jacobian: Option<Matrix<'static>>,
}
impl VectorResult {
    /// Reports Status::Ok; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&VectorResult> for bool {
    fn from(result: &VectorResult) -> Self {
        result.is_done()
    }
}
/// Native result with optional computed fields; `LinearResult::default()`.
#[derive(Default)]
pub struct LinearResult {
    /// Native status field; `result.status`.
    pub status: Status,
    /// Native solution field; `result.solution`.
    pub solution: Option<Vector<'static>>,
    /// Native determinant field; `result.determinant`.
    pub determinant: Option<f64>,
}
impl LinearResult {
    /// Reports Status::Ok; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&LinearResult> for bool {
    fn from(result: &LinearResult) -> Self {
        result.is_done()
    }
}
/// Native result with optional computed fields; `LinearMultipleResult::default()`.
#[derive(Default)]
pub struct LinearMultipleResult {
    /// Native status field; `result.status`.
    pub status: Status,
    /// Native solutions field; `result.solutions`.
    pub solutions: Option<Matrix<'static>>,
    /// Native determinant field; `result.determinant`.
    pub determinant: Option<f64>,
}
impl LinearMultipleResult {
    /// Reports Status::Ok; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&LinearMultipleResult> for bool {
    fn from(result: &LinearMultipleResult) -> Self {
        result.is_done()
    }
}
/// Native result with optional computed fields; `EigenResult::default()`.
#[derive(Default)]
pub struct EigenResult {
    /// Native status field; `result.status`.
    pub status: Status,
    /// Native iterations field; `result.iterations`.
    pub iterations: u32,
    /// Native eigen values field; `result.eigen_values`.
    pub eigen_values: Option<Vector<'static>>,
    /// Native eigen vectors field; `result.eigen_vectors`.
    pub eigen_vectors: Option<Matrix<'static>>,
}
impl EigenResult {
    /// Reports Status::Ok; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&EigenResult> for bool {
    fn from(result: &EigenResult) -> Self {
        result.is_done()
    }
}
/// Native result with optional computed fields; `DecompResult::default()`.
#[derive(Default)]
pub struct DecompResult {
    /// Native status field; `result.status`.
    pub status: Status,
    /// Native lower field; `result.lower`.
    pub lower: Option<Matrix<'static>>,
    /// Native upper field; `result.upper`.
    pub upper: Option<Matrix<'static>>,
    /// Native diagonal field; `result.diagonal`.
    pub diagonal: Option<Vector<'static>>,
    /// Native determinant field; `result.determinant`.
    pub determinant: Option<f64>,
}
impl DecompResult {
    /// Reports Status::Ok; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&DecompResult> for bool {
    fn from(result: &DecompResult) -> Self {
        result.is_done()
    }
}
/// Native result with optional computed fields; `IntegResult::default()`.
#[derive(Default)]
pub struct IntegResult {
    /// Native status field; `result.status`.
    pub status: Status,
    /// Native iterations field; `result.iterations`.
    pub iterations: u32,
    /// Native number of points field; `result.number_of_points`.
    pub number_of_points: usize,
    /// Native value field; `result.value`.
    pub value: Option<f64>,
    /// Native absolute error field; `result.absolute_error`.
    pub absolute_error: Option<f64>,
    /// Native relative error field; `result.relative_error`.
    pub relative_error: Option<f64>,
}
impl IntegResult {
    /// Reports Status::Ok; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&IntegResult> for bool {
    fn from(result: &IntegResult) -> Self {
        result.is_done()
    }
}
/// Native result with optional computed fields; `InverseResult::default()`.
#[derive(Default)]
pub struct InverseResult {
    /// Native status field; `result.status`.
    pub status: Status,
    /// Native inverse field; `result.inverse`.
    pub inverse: Option<Matrix<'static>>,
    /// Native determinant field; `result.determinant`.
    pub determinant: Option<f64>,
}
impl InverseResult {
    /// Reports Status::Ok; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&InverseResult> for bool {
    fn from(result: &InverseResult) -> Self {
        result.is_done()
    }
}
impl std::ops::Index<usize> for PolyResult {
    type Output = f64;
    fn index(&self, index: usize) -> &f64 {
        &self.roots[index]
    }
}
