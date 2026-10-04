//! Source-pinned protected Jacobi derivative-order behavior.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::plib::{BasisRows, HermitJacobi, JacobiPolynomial};
use std::fmt::Write;

const INITIAL: f64 = 777.0;

fn output_rows(count: usize) -> [Array1<'static, f64>; 4] {
    std::array::from_fn(|row| {
        let lower: i32 = [-2, 3, 8, 13][row];
        let upper: i32 = lower + count as i32 - 1;
        let mut output: Array1<'static, f64> = Array1::with_bounds(lower, upper).unwrap();
        output.as_mut_slice().fill(INITIAL);
        output
    })
}

fn emit_rows(label: &str, order: i32, outputs: &[Array1<'_, f64>; 4]) -> String {
    let mut line: String = format!("{label} {order} ");
    append_rows(&mut line, outputs);
    line
}

fn append_rows(line: &mut String, outputs: &[Array1<'_, f64>; 4]) {
    for output in outputs {
        for value in output.as_slice() {
            write!(line, "{} ", value.to_bits()).unwrap();
        }
    }
}

fn row_slices<'a>(outputs: &'a mut [Array1<'static, f64>; 4]) -> BasisRows<'a> {
    let [value, d1, d2, d3] = outputs;
    BasisRows {
        rows: [
            value.as_mut_slice(),
            d1.as_mut_slice(),
            d2.as_mut_slice(),
            d3.as_mut_slice(),
        ],
    }
}

fn jacobi_case(label: &str, order: i32, work: i32, shape: Shape) -> String {
    let basis: JacobiPolynomial = JacobiPolynomial::new(work, shape).unwrap();
    let mut outputs: [Array1<'static, f64>; 4] = output_rows(basis_degree(&basis));
    basis.d0123(order, 0.37, row_slices(&mut outputs)).unwrap();
    emit_rows(label, order, &outputs)
}

fn basis_degree(basis: &JacobiPolynomial) -> usize {
    (basis.work_degree() - 2 * (basis.niv_constr() + 1) + 1) as usize
}

fn hermit_case(order: i32) -> Result<String, rustycad_tkernel::standard::Failure> {
    let basis: HermitJacobi = HermitJacobi::new(8, Shape::C1).unwrap();
    let mut outputs: [Array1<'static, f64>; 4] = output_rows(basis.work_degree() as usize + 1);
    basis.d0123(order, 0.37, row_slices(&mut outputs))?;
    Ok(emit_rows("H", order, &outputs))
}

#[test]
fn protected_basis_evaluators_match_native_rows_and_untouched_buffers() {
    let expected: Vec<String> = include_str!("../../../tools/oracle/tkmath_plib_d0123.expected")
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(native_rows(), expected);
}

fn native_rows() -> Vec<String> {
    [
        jacobi_case("J0", -1, 6, Shape::C2),
        jacobi_case("J0", 0, 6, Shape::C2),
        jacobi_case("J", -1, 8, Shape::C1),
        jacobi_case("J", 0, 8, Shape::C1),
        jacobi_case("J", 1, 8, Shape::C1),
        jacobi_case("J", 2, 8, Shape::C1),
        jacobi_case("J", 3, 8, Shape::C1),
        jacobi_case("J", 4, 8, Shape::C1),
        hermit_case(0).unwrap(),
        hermit_case(1).unwrap(),
        hermit_case(2).unwrap(),
        hermit_case(3).unwrap(),
    ]
    .into_iter()
    .collect()
}

#[test]
fn hermit_rejects_orders_outside_native_valid_domain() {
    let error = hermit_case(4).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(error.message().contains("NDeriv = 4"));
}
