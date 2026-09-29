//! Empty bounds as native builds them, and named failures where native input is
//! undefined (regressions for the panics these inputs used to reach).
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{
    Bfgs, Crout, EigenValuesSearcher, FunctionSet, FunctionSetRoot, FunctionSetWithDerivatives,
    Gauss, Householder, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient,
    MultipleVarFunctionWithHessian, NewtonMinimum, Uzawa, Vector,
};

fn kind<T>(result: Result<T, Failure>) -> String {
    result
        .err()
        .expect("a failure")
        .exception_type()
        .to_string()
}

#[test]
fn empty_matrix_bounds_build_empty_matrices() {
    let matrix = Matrix::filled(1, 0, 5, 7, 2.0).unwrap();
    assert_eq!((matrix.row_number(), matrix.col_number()), (0, 3));
    assert_eq!((matrix.upper_row(), matrix.upper_col()), (0, 7));
    assert_eq!(matrix.transposed().row_number(), 3);
    assert_eq!(kind(matrix.row(1)), "Standard_OutOfRange");
    let empty = Matrix::with_bounds(1, 0, 1, 0).unwrap();
    assert_eq!(empty.determinant().unwrap(), 1.0);
    assert_eq!(empty.inverse().unwrap().row_number(), 0);
    assert!(Gauss::new(&empty).unwrap().is_done());
    assert!(Crout::new(&empty).unwrap().is_done());
}

#[test]
fn bounds_below_the_empty_range_are_refused_with_their_values() {
    let error = Matrix::with_bounds(1, -1, 1, 2).err().unwrap();
    assert!(error.to_string().contains("[1, -1]"), "{error}");
    let failure = Vector::with_bounds(4, 2).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_RangeError");
    assert!(
        failure.message().contains("[4, 2]"),
        "{}",
        failure.message()
    );
}

#[test]
fn empty_vector_products_raise_where_native_walks_pointers() {
    let empty = Vector::with_bounds(1, 0).unwrap();
    assert_eq!(kind(empty.dot(&empty)), "Standard_OutOfRange");
    let matrix = Matrix::with_bounds(1, 2, 1, 0).unwrap();
    let product = matrix.multiplied_vector(&empty).unwrap();
    assert_eq!((product[1], product[2]), (0.0, 0.0));
    let mut output = Vector::with_bounds(1, 2).unwrap();
    assert_eq!(
        kind(output.set_matrix_vector(&matrix, &empty)),
        "Standard_OutOfRange"
    );
}

#[test]
fn empty_eigenproblem_raises_at_the_subdiagonal_shift() {
    let empty: Array1<'static, f64> = Array1::with_bounds(1, 0).unwrap();
    assert_eq!(
        kind(EigenValuesSearcher::new(&empty, &empty)),
        "Standard_OutOfRange"
    );
}

#[test]
fn householder_with_more_unknowns_than_rows_is_refused_instead_of_panicking() {
    let mut a = Matrix::with_bounds(1, 2, 1, 3).unwrap();
    for (offset, value) in [1.0, 2.0, 0.5, -1.0, 3.0, 2.0].into_iter().enumerate() {
        *a.change_at(offset / 3, offset % 3).unwrap() = value;
    }
    let b = Matrix::filled(1, 2, 1, 1, 1.0).unwrap();
    let error = Householder::from_matrix(&a, &b, 1e-20).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_DimensionError");
    assert!(
        error.message().contains("3 unknowns, 2 rows"),
        "{}",
        error.message()
    );
    // A pivot below the threshold stops the reduction before the missing row.
    let flat = Matrix::filled(1, 2, 1, 3, 0.0).unwrap();
    assert!(
        !Householder::from_matrix(&flat, &b, 1e-20)
            .unwrap()
            .is_done()
    );
    let none = Matrix::with_bounds(1, 2, 1, 0).unwrap();
    assert_eq!(
        kind(Householder::from_matrix(&none, &b, 1e-20)),
        "Standard_DimensionError"
    );
}

#[test]
fn uzawa_without_rows_or_with_shifted_bounds_is_refused_instead_of_panicking() {
    let start = Vector::filled(1, 2, 0.0).unwrap();
    let none = Vector::with_bounds(1, 0).unwrap();
    let no_rows = Matrix::with_bounds(1, 0, 1, 2).unwrap();
    let error = Uzawa::new(&no_rows, &none, &start).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_DimensionError");
    assert!(
        error.message().contains("matrix 0x2"),
        "{}",
        error.message()
    );
    let shifted = Matrix::filled(0, 0, 1, 2, 1.0).unwrap();
    let rhs = Vector::filled(1, 1, 1.0).unwrap();
    let error = Uzawa::new(&shifted, &rhs, &start).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(
        error.message().contains("[0, 1, 1, 1]"),
        "{}",
        error.message()
    );
    let empty = Matrix::with_bounds(1, 0, 1, 0).unwrap();
    assert!(Uzawa::new(&empty, &none, &none).unwrap().is_done());
}

/// An objective without variables, as native math solvers accept it.
struct NoVariable;
impl MultipleVarFunction for NoVariable {
    fn variables(&self) -> i32 {
        0
    }
    fn value(&mut self, _: &Vector<'_>, value: &mut f64) -> bool {
        *value = 1.0;
        true
    }
}
impl MultipleVarFunctionWithGradient for NoVariable {
    fn gradient(&mut self, _: &Vector<'_>, _: &mut Vector<'_>) -> bool {
        true
    }
    fn values(&mut self, _: &Vector<'_>, value: &mut f64, _: &mut Vector<'_>) -> bool {
        *value = 1.0;
        true
    }
}
impl MultipleVarFunctionWithHessian for NoVariable {
    fn values_hessian(
        &mut self,
        _: &Vector<'_>,
        value: &mut f64,
        _: &mut Vector<'_>,
        _: &mut Matrix<'_>,
    ) -> bool {
        *value = 1.0;
        true
    }
}

#[test]
fn solvers_without_variables_run_or_fail_by_name() {
    let start = Vector::with_bounds(1, 0).unwrap();
    let mut bfgs = Bfgs::new(0).unwrap();
    bfgs.perform(&mut NoVariable, &start).unwrap();
    assert!(bfgs.is_done());
    assert_eq!(bfgs.minimum().unwrap(), 1.0);
    // Native reads Values()(0) before the empty eigenvalue vector.
    let mut newton = NewtonMinimum::new(&NoVariable).unwrap();
    let error = newton.perform(&mut NoVariable, &start).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(
        error.message().contains("0 variables"),
        "{}",
        error.message()
    );
}

/// An equation set reporting a negative equation count.
struct NegativeEquations;
impl FunctionSet for NegativeEquations {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        -1
    }
    fn value(&mut self, _: &Vector<'_>, _: &mut Vector<'_>) -> bool {
        true
    }
}
impl FunctionSetWithDerivatives for NegativeEquations {
    fn derivatives(&mut self, _: &Vector<'_>, _: &mut Matrix<'_>) -> bool {
        true
    }
    fn values(&mut self, _: &Vector<'_>, _: &mut Vector<'_>, _: &mut Matrix<'_>) -> bool {
        true
    }
}

#[test]
fn a_negative_equation_count_is_refused_instead_of_panicking() {
    let failure = FunctionSetRoot::without_tolerance(&NegativeEquations, 10)
        .err()
        .unwrap();
    assert_eq!(failure.exception_type(), "Standard_RangeError");
    assert!(
        failure.message().contains("[1, -1]"),
        "{}",
        failure.message()
    );
}
