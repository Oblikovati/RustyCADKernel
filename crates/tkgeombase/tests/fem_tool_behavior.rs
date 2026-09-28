// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! The exceptions and preserved native quirks of the FEmTool port
//! (docs/tkgeombase-progress.md). The reference-integral cache has its own test
//! binary (fem_tool_reference_cache.rs): it is process-wide state.
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1, HArray2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::fem_tool::{
    Assembly, AssemblyTable, Curve, ElementaryCriterion, ElementsOfRefMatrix, LinearFlexion,
    LinearJerk, LinearTension, ProfileMatrix, SparseMatrix,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::plib::HermitJacobi;

fn kind<T>(result: Result<T, Failure>) -> &'static str {
    result.err().map_or("none", |error| error.exception_type())
}

fn coeff(rl: i32, cl: i32, rows: i32, cols: i32) -> Handle<HArray2<f64>> {
    let mut values = Array2::with_bounds(rl, rl + rows - 1, cl, cl + cols - 1).unwrap();
    values
        .iter_mut()
        .zip(1..)
        .for_each(|(v, i)| *v = f64::from(i) * 0.25);
    Handle::new(HArray2::from_owned(values))
}

#[test]
fn elements_of_ref_matrix_checks_the_order_and_the_output_length() {
    let base = HermitJacobi::new(5, Shape::C1).unwrap();
    assert_eq!(
        kind(ElementsOfRefMatrix::new(&base, 4)),
        "Standard_ConstructionError"
    );
    assert_eq!(
        kind(ElementsOfRefMatrix::new(&base, -1)),
        "Standard_ConstructionError"
    );
    let elements = ElementsOfRefMatrix::new(&base, 2).unwrap();
    let x = Vector::filled(1, 1, 0.5).unwrap();
    let mut short = Vector::filled(1, 20, 0.).unwrap();
    assert_eq!(
        kind(elements.evaluate(&x, &mut short)),
        "Standard_OutOfRange"
    );
    let mut f = Vector::filled(3, 23, 0.).unwrap();
    assert!(elements.evaluate(&x, &mut f).unwrap());
}

#[test]
fn criteria_refuse_shapes_other_than_c0_c1_c2() {
    assert_eq!(
        kind(LinearFlexion::new(5, Shape::G1)),
        "Standard_ConstructionError"
    );
    assert_eq!(
        kind(LinearJerk::new(5, Shape::C3)),
        "Standard_ConstructionError"
    );
}

#[test]
fn criteria_without_coefficients_raise_domain_error_or_null_object() {
    let criterion = LinearJerk::new(5, Shape::C1).unwrap();
    assert_eq!(kind(criterion.dependence_table()), "Standard_DomainError");
    let mut h = Matrix::filled(0, 5, 0, 5, 0.).unwrap();
    assert_eq!(
        kind(criterion.hessian(1, 1, &mut h)),
        "Standard_DomainError"
    );
    // A null dereference natively; the port reports it.
    assert_eq!(kind(criterion.value()), "Standard_NullObject");
}

#[test]
fn tension_dependence_table_uses_one_based_indexes() {
    let tension = LinearTension::new(5, Shape::C1).unwrap();
    tension.set_coeff(&coeff(0, 0, 6, 2));
    assert_eq!(kind(tension.dependence_table()), "Standard_OutOfRange");
    let flexion = LinearFlexion::new(5, Shape::C1).unwrap();
    flexion.set_coeff(&coeff(0, 0, 6, 2));
    let table = flexion.dependence_table().unwrap();
    assert_eq!(
        (
            table.array2().lower_row(),
            table.array2()[(0, 0)],
            table.array2()[(0, 1)]
        ),
        (0, 1, 0)
    );
}

#[test]
fn flexion_hessian_writes_from_index_zero_jerk_from_the_lower_bounds() {
    let (flexion, jerk) = (
        LinearFlexion::new(3, Shape::C0).unwrap(),
        LinearJerk::new(3, Shape::C0).unwrap(),
    );
    flexion.set_coeff(&coeff(0, 1, 4, 1));
    jerk.set_coeff(&coeff(0, 1, 4, 1));
    flexion.set(0., 2.);
    jerk.set(0., 2.);
    let mut h = Matrix::filled(1, 4, 1, 4, 0.).unwrap();
    assert_eq!(kind(flexion.hessian(1, 1, &mut h)), "Standard_OutOfRange");
    assert_eq!(kind(jerk.hessian(1, 1, &mut h)), "none");
    assert_eq!(kind(jerk.hessian(1, 2, &mut h)), "Standard_OutOfRange");
}

#[test]
fn profile_matrix_solves_only_after_decompose_and_has_no_iterative_solver() {
    let matrix = ProfileMatrix::new(&Array1::from_shared(&[1, 1], 1)).unwrap();
    matrix.init(1.);
    let (b, mut x) = (
        Vector::filled(1, 2, 1.).unwrap(),
        Vector::filled(1, 2, 0.).unwrap(),
    );
    assert_eq!(kind(matrix.solve(&b, &mut x)), "StdFail_NotDone");
    assert_eq!(kind(matrix.prepare()), "Standard_NotImplemented");
    let mut residual = x.clone();
    assert_eq!(
        kind(matrix.solve_iterative(&b, &b, &mut x, &mut residual, 1e-8, 50)),
        "Standard_NotImplemented"
    );
    // [[1, 1], [1, 1]] is singular: the second pivot is 0.
    assert!(!matrix.decompose().unwrap());
}

#[test]
fn profile_matrix_entries_outside_the_profile_alias_stored_ones() {
    let matrix = ProfileMatrix::new(&Array1::from_shared(&[1, 2, 2, 4], 1)).unwrap();
    matrix.init(0.);
    *matrix.change_value(2, 2).unwrap() = 3.;
    // Row 3 stores columns 2..=3: (3, 1) falls on the diagonal of row 2.
    assert_eq!(*matrix.change_value(3, 1).unwrap(), 3.);
    assert!(!matrix.is_in_profile(3, 1).unwrap());
    // OutM pads each row with "0 " up to its first stored column.
    assert_eq!(
        matrix.out_text(true).unwrap(),
        "Matrix A\n0 \n0 3 \n0 0 0 \n0 0 0 0 \nNextCoeff\n0 3 0 0 0 \n"
    );
}

fn curve() -> Curve {
    let curve = Curve::new(2, 2, &HermitJacobi::new(5, Shape::C1).unwrap(), 1e-7).unwrap();
    curve
        .knots()
        .iter_mut()
        .zip([0., 1., 3.])
        .for_each(|(k, v)| *k = v);
    curve
}

#[test]
fn curve_checks_degrees_elements_and_length_bounds() {
    let curve = curve();
    assert_eq!(kind(curve.set_degree(1, 6)), "Standard_OutOfRange");
    assert_eq!(kind(curve.degree(3)), "Standard_OutOfRange");
    assert_eq!(kind(curve.length(2., 1.)), "Standard_OutOfRange");
    let values = Array2::with_bounds(0, 5, 1, 2).unwrap();
    assert_eq!(kind(curve.set_element(0, &values)), "Standard_OutOfRange");
    assert_eq!(curve.length(0., 3.).unwrap(), 0.);
}

#[test]
fn curve_get_element_rescales_the_columns_lower_col_to_dimension() {
    let curve = curve();
    let mut values = Array2::with_bounds(0, 5, 1, 2).unwrap();
    values.init(1.);
    // Element 1 has length 1: SetElement halves the derivative rows 1 and 3.
    curve.set_element(1, &values).unwrap();
    // Columns 2..=3: the rescaling loop runs over columns 2..=Dimension (2) only.
    let mut read = Array2::with_bounds(0, 5, 2, 3).unwrap();
    curve.get_element(1, &mut read).unwrap();
    assert_eq!(
        (read[(1, 2)], read[(1, 3)], read[(3, 2)], read[(3, 3)]),
        (1., 0.5, 1., 0.5)
    );
    assert_eq!(read[(0, 3)], 1.);
}

#[test]
fn curve_d1_scales_every_entry_of_the_output() {
    let curve = curve();
    let mut out = Array1::with_bounds(1, 3).unwrap();
    out.init(1.);
    curve.d1(0.5, &mut out).unwrap();
    // The two dimensions are 0 (null element); the extra entry is scaled by 2 / 1.
    assert_eq!(out.as_slice(), &[0., 0., 2.]);
}

fn table(cells: &[&[i32]]) -> AssemblyTable {
    let table: AssemblyTable =
        Handle::new(HArray2::with_bounds(1, 1, 1, cells.len() as i32).unwrap());
    for (cell, j) in cells.iter().zip(1..) {
        table.change_array2()[(1, j)] =
            Handle::new(HArray1::from_array(&Array1::from_shared(cell, 1)));
    }
    table
}

#[test]
fn assembly_checks_the_dependence_and_the_solution_state() {
    let mut dependence = Array2::with_bounds(1, 1, 1, 1).unwrap();
    dependence.init(0);
    let mut assembly = Assembly::new(&dependence, &table(&[&[1, 2]])).unwrap();
    let h = Matrix::filled(1, 2, 1, 2, 1.).unwrap();
    assert_eq!(
        kind(assembly.add_matrix(1, 1, 1, &h)),
        "Standard_DomainError"
    );
    let mut x = Vector::filled(1, 2, 0.).unwrap();
    assert_eq!(kind(assembly.solution(&mut x)), "StdFail_NotDone");
    assert!(!assembly.solve().unwrap());
    assert_eq!(assembly.nb_glob_var(), 2);
}

#[test]
fn assembly_prepends_a_constraint_segment_compared_with_the_last_match() {
    let mut dependence = Array2::with_bounds(1, 1, 1, 1).unwrap();
    dependence.init(1);
    let mut assembly = Assembly::new(&dependence, &table(&[&[1, 2], &[2, 3], &[4, 5]])).unwrap();
    for el in 1..=3 {
        let mut h = Matrix::filled(1, 2, 1, 2, 0.).unwrap();
        (h[(1, 1)], h[(2, 2)]) = (4., 4.);
        assembly.add_matrix(el, 1, 1, &h).unwrap();
    }
    let form = Vector::filled(1, 2, 1.).unwrap();
    for el in [2, 3, 1] {
        assembly.add_constraint(1, el, 1, &form, 1.).unwrap();
    }
    assert!(assembly.solve().unwrap());
    let mut x = Vector::filled(1, 5, 0.).unwrap();
    assembly.solution(&mut x).unwrap();
    assert!(x.array1().iter().all(|value| value.is_finite()));
}
