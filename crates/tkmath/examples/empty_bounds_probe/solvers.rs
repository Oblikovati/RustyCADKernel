//! The solvers and decompositions over no variable at all (`empty_solvers`).
use super::super::vector_base_probe::{cells, guarded, reals};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{
    Bfgs, Frpr, FunctionSet, FunctionSetRoot, FunctionSetWithDerivatives, GaussLeastSquares,
    Householder, Jacobi, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient,
    NewtonFunctionSetRoot, Powell, Svd, Uzawa, Vector,
};

/// Objective without any variable: returns `value`, callbacks report `ok`.
struct NoVariable {
    value: f64,
    ok: bool,
}
impl MultipleVarFunction for NoVariable {
    fn variables(&self) -> i32 {
        0
    }
    fn value(&mut self, _: &Vector<'_>, value: &mut f64) -> bool {
        *value = self.value;
        self.ok
    }
}
impl MultipleVarFunctionWithGradient for NoVariable {
    fn gradient(&mut self, _: &Vector<'_>, _: &mut Vector<'_>) -> bool {
        self.ok
    }
    fn values(&mut self, _: &Vector<'_>, value: &mut f64, _: &mut Vector<'_>) -> bool {
        *value = self.value;
        self.ok
    }
}
/// Equation set without variable or equation; callbacks report `ok`.
struct NoEquation {
    ok: bool,
}
impl FunctionSet for NoEquation {
    fn variables(&self) -> i32 {
        0
    }
    fn equations(&self) -> i32 {
        0
    }
    fn value(&mut self, _: &Vector<'_>, _: &mut Vector<'_>) -> bool {
        self.ok
    }
}
impl FunctionSetWithDerivatives for NoEquation {
    fn derivatives(&mut self, _: &Vector<'_>, _: &mut Matrix<'_>) -> bool {
        self.ok
    }
    fn values(&mut self, _: &Vector<'_>, _: &mut Vector<'_>, _: &mut Matrix<'_>) -> bool {
        self.ok
    }
}
/// `IsDone`, then the guarded `Minimum` and `NbIterations` of one minimizer.
fn minimizer(done: bool, minimum: Result<f64, Failure>, iterations: Result<i32, Failure>) {
    print!("{} ", u8::from(done));
    guarded(|| shown!(print!("{} ", minimum?.to_bits())));
    guarded(|| shown!(print!("{} ", iterations?)));
}
fn empty_minimizers(value: f64, ok: bool) {
    let start = Vector::with_bounds(1, 0).unwrap();
    guarded(|| {
        let mut f = NoVariable { value, ok };
        let mut s = Bfgs::new(0)?;
        s.perform(&mut f, &start)?;
        shown!(minimizer(s.is_done(), s.minimum(), s.iterations()))
    });
    guarded(|| {
        let mut f = NoVariable { value, ok };
        let mut s = Frpr::new(&f, 1e-8)?;
        s.perform(&mut f, &start)?;
        shown!(minimizer(s.is_done(), s.minimum(), s.iterations()))
    });
    guarded(|| {
        let mut f = NoVariable { value, ok };
        let mut s = Powell::new(&f, 1e-8)?;
        s.perform(&mut f, &start, &Matrix::with_bounds(1, 0, 1, 0).unwrap())?;
        shown!(minimizer(s.is_done(), s.minimum(), s.iterations()))
    });
    empty_roots(ok, &start);
}
fn empty_roots(ok: bool, start: &Vector<'_>) {
    let tolerance = Vector::with_bounds(1, 0).unwrap();
    guarded(|| {
        let mut f = NoEquation { ok };
        let mut s = FunctionSetRoot::new(&f, &tolerance, 100)?;
        s.perform(&mut f, start, false)?;
        shown!(print!("{} ", u8::from(s.is_done())))
    });
    guarded(|| {
        let mut f = NoEquation { ok };
        let mut s = NewtonFunctionSetRoot::new(&f, &tolerance, 1e-8, 100)?;
        s.perform(&mut f, start)?;
        shown!(print!("{} ", u8::from(s.is_done())))
    });
}
fn empty_decompositions() {
    let e00 = Matrix::with_bounds(1, 0, 1, 0).unwrap();
    let e02 = Matrix::with_bounds(1, 0, 1, 2).unwrap();
    let none = Vector::with_bounds(1, 0).unwrap();
    guarded(|| {
        let s = Jacobi::new(&e00)?;
        print!("{} ", u8::from(s.is_done()));
        shown!(reals(s.values()?))
    });
    guarded(|| shown!(print!("{} ", u8::from(Svd::new(&e00)?.is_done()))));
    guarded(|| shown!(print!("{} ", u8::from(Svd::new(&e02)?.is_done()))));
    guarded(|| {
        shown!(print!(
            "{} ",
            u8::from(GaussLeastSquares::new(&e00)?.is_done())
        ))
    });
    guarded(|| {
        shown!(print!(
            "{} ",
            u8::from(GaussLeastSquares::new(&e02)?.is_done())
        ))
    });
    guarded(|| {
        let s = Householder::from_matrix(&e00, &e00, 1e-20)?;
        print!("{} ", u8::from(s.is_done()));
        shown!(cells(s.all_values()?))
    });
    guarded(|| {
        let s = Uzawa::new(&e00, &none, &none)?;
        print!("{} ", u8::from(s.is_done()));
        shown!(reals(s.value()?))
    });
}
/// `value ok`.
pub(in super::super) fn empty_solvers_case(v: &[f64]) {
    empty_minimizers(v[0], v[1] != 0.0);
    empty_decompositions();
    println!();
}
