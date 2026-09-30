// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_SparseMatrix.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FEmTool_SparseMatrix`: the abstract symmetric sparse matrix of the FEmTool
//! assembly.
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, StandardTransient, Transient};
use rustycad_tkmath::math::Vector;
use std::ops::{Deref, DerefMut};

/// A writable matrix entry (the native `double&` of `ChangeValue`); the matrix
/// stays locked while it lives. `*matrix.change_value(1, 1)? += 2.0`.
pub struct EntryRef<'a>(Box<dyn DerefMut<Target = f64> + 'a>);

impl<'a> EntryRef<'a> {
    /// Wraps a locked entry of an implementation; `EntryRef::new(guard)`.
    pub fn new(entry: impl DerefMut<Target = f64> + 'a) -> Self {
        Self(Box::new(entry))
    }
}

impl Deref for EntryRef<'_> {
    type Target = f64;
    fn deref(&self) -> &f64 {
        &self.0
    }
}

impl DerefMut for EntryRef<'_> {
    fn deref_mut(&mut self) -> &mut f64 {
        &mut self.0
    }
}

/// `FEmTool_SparseMatrix`: implemented by [`super::ProfileMatrix`]; a handle to the
/// trait object is the native base handle.
///
/// ```
/// use rustycad_tkgeombase::fem_tool::{ProfileMatrix, SparseMatrix};
/// use rustycad_tkernel::ncollection::Array1;
/// let matrix = ProfileMatrix::new(&Array1::from_shared(&[1, 1], 1))?;
/// matrix.init(0.0);
/// *matrix.change_value(1, 1)? = 4.0;
/// *matrix.change_value(2, 2)? = 9.0;
/// assert!(matrix.decompose()?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait SparseMatrix: Transient {
    /// `Init(Value)`: sets every stored entry; `matrix.init(0.0)`.
    fn init(&self, value: f64);

    /// `ChangeValue(I, J)`: the stored entry (I, J); `*matrix.change_value(1, 2)? = 1.0`.
    fn change_value(&self, i: i32, j: i32) -> Result<EntryRef<'_>, Failure>;

    /// `Decompose()`: factorizes the matrix; false when it is not positive
    /// definite; `matrix.decompose()?`.
    fn decompose(&self) -> Result<bool, Failure>;

    /// `Solve(B, X)`: the direct solution of A X = B; `matrix.solve(&b, &mut x)?`.
    fn solve(&self, b: &Vector<'_>, x: &mut Vector<'_>) -> Result<(), Failure>;

    /// `Prepare()`: prepares an iterative solution; `matrix.prepare()`.
    fn prepare(&self) -> Result<bool, Failure>;

    /// `Solve(B, Init, X, Residual, Tolerance, NbIterations)`: the iterative
    /// solution (native defaults 1e-8 and 50); `matrix.solve_iterative(&b, &x0, &mut x, &mut r, 1e-8, 50)`.
    fn solve_iterative(
        &self,
        b: &Vector<'_>,
        init: &Vector<'_>,
        x: &mut Vector<'_>,
        residual: &mut Vector<'_>,
        tolerance: f64,
        nb_iterations: i32,
    ) -> Result<(), Failure>;

    /// `Multiplied(X, MX)`: MX = A X; `matrix.multiplied(&x, &mut mx)?`.
    fn multiplied(&self, x: &Vector<'_>, mx: &mut Vector<'_>) -> Result<(), Failure>;

    /// `RowNumber()`; `matrix.row_number()`.
    fn row_number(&self) -> i32;

    /// `ColNumber()`; `matrix.col_number()`.
    fn col_number(&self) -> i32;
}
impl_transient_interface!(dyn SparseMatrix, "FEmTool_SparseMatrix", StandardTransient);
