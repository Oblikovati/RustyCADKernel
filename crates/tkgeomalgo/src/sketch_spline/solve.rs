// SPDX-License-Identifier: GPL-2.0-only
use super::{SketchSplineDefinition, SketchSplineIntent, decline, knots};
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
use rustycad_tkmath::{
    bsplclib,
    gp::Point2,
    math::{Gauss, Matrix, Vector},
};

pub(super) struct Collocation<'a> {
    points: &'a [Point2],
    degree: i32,
    flat: Vec<f64>,
    stations: &'a [f64],
    closed: bool,
    intent: SketchSplineIntent,
}
impl<'a> Collocation<'a> {
    pub(super) fn new(
        points: &'a [Point2],
        closed: bool,
        stations: &'a [f64],
        intent: SketchSplineIntent,
    ) -> Self {
        let degree = if closed {
            3
        } else {
            (points.len() - 1).min(3) as i32
        };
        let flat = knots::for_intent(points.len(), degree, intent, closed, stations);
        Self {
            points,
            degree,
            flat,
            stations,
            closed,
            intent,
        }
    }
    pub(super) fn construct(self, minimum: f64) -> Result<SketchSplineDefinition, Failure> {
        let mut poles = match self.intent {
            SketchSplineIntent::Control => self.points.to_vec(),
            SketchSplineIntent::Fit(_) => fit(&self, minimum)?,
        };
        if self.closed {
            poles.extend_from_within(..3);
        }
        Ok(SketchSplineDefinition::new(self.degree, poles, self.flat))
    }
}
fn fit(system: &Collocation<'_>, minimum: f64) -> Result<Vec<Point2>, Failure> {
    let matrix = collocation(system)?;
    let solver = Gauss::with_min_pivot(&matrix, minimum)?;
    if !solver.is_done() {
        return Err(decline(format!(
            "collocation for {} points, pivot limit {minimum}: expected a completed nonsingular factorization",
            system.points.len()
        )));
    }
    let mut x = coordinate_rhs(system.points, Point2::x)?;
    let mut y = coordinate_rhs(system.points, Point2::y)?;
    solver.solve(&mut x)?;
    solver.solve(&mut y)?;
    Ok((1..=system.points.len() as i32)
        .map(|index| Point2::new(x[index], y[index]))
        .collect())
}
fn coordinate_rhs(
    points: &[Point2],
    coordinate: fn(&Point2) -> f64,
) -> Result<Vector<'static>, Failure> {
    let mut rhs = Vector::with_bounds(1, points.len() as i32)?;
    for (index, point) in points.iter().enumerate() {
        rhs[index as i32 + 1] = coordinate(point);
    }
    Ok(rhs)
}
fn collocation(system: &Collocation<'_>) -> Result<Matrix<'static>, Failure> {
    let count = system.points.len() as i32;
    let mut matrix = zero_matrix(count, count)?;
    for (index, &parameter) in system.stations.iter().take(system.points.len()).enumerate() {
        let (basis, first) = evaluate_basis(system, parameter)?;
        adopt_basis(&mut matrix, &basis, index as i32 + 1, first, system.closed)?;
    }
    Ok(matrix)
}
fn evaluate_basis(
    system: &Collocation<'_>,
    parameter: f64,
) -> Result<(Matrix<'static>, i32), Failure> {
    let mut basis = zero_matrix(1, system.degree + 1)?;
    let mut first = 0;
    let status = bsplclib::eval_bspline_basis(
        0,
        system.degree + 1,
        &Array1::from_shared(&system.flat, 1),
        parameter,
        &mut first,
        &mut basis,
        false,
    )?;
    if status != 0 {
        return Err(decline(format!(
            "basis at parameter {parameter}, status {status}: expected successful basis evaluation"
        )));
    }
    Ok((basis, first))
}
fn adopt_basis(
    matrix: &mut Matrix<'_>,
    basis: &Matrix<'_>,
    row: i32,
    first: i32,
    closed: bool,
) -> Result<(), Failure> {
    let count = matrix.col_number();
    for offset in 0..basis.col_number() {
        let index = first + offset - 1;
        let column = if closed {
            index.rem_euclid(count)
        } else {
            index
        };
        if column < 0 || column >= count {
            return Err(decline(format!(
                "basis column {column}, first {first}, offset {offset}: expected 0..{count}"
            )));
        }
        matrix[(row, column + 1)] += basis[(1, offset + 1)];
    }
    Ok(())
}
fn zero_matrix(rows: i32, columns: i32) -> Result<Matrix<'static>, Failure> {
    Matrix::filled(1, rows, 1, columns, 0.0).map_err(|cause| {
        decline(format!(
            "matrix dimensions {rows} x {columns}: expected valid bounds: {cause}"
        ))
    })
}
