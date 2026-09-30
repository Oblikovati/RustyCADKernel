//! Native throwing callback traversal and retained cell contents.
use super::super::int;
use rustycad_tkernel::{
    ncollection::{CellAction, CellFilter, CellGeometry},
    standard::Failure,
};
struct PlanarIndices;
impl CellGeometry for PlanarIndices {
    type CellTarget = i32;
    type CellPoint = [f64; 2];
    const CELL_DIMENSION: usize = 2;
    fn cell_coordinate(axis: usize, point: &Self::CellPoint) -> f64 {
        point[axis]
    }
    fn targets_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
}
pub fn cell_failure_case(v: &[i32]) {
    let mut filter = CellFilter::<PlanarIndices>::new(1.0);
    for index in 1..=4 {
        filter.add_range(&index, &[0., 0.], &[1., 1.]);
    }
    let inspect = |index: &i32| inspect_target(*index, v[0], v[1]);
    let result = if v[2] == 0 {
        filter.try_inspect_with(&[0., 0.], inspect)
    } else {
        filter.try_inspect_range_with(&[0., 0.], &[1., 1.], inspect)
    };
    match result {
        Ok(()) => print!("OK "),
        Err(error) => print!("{} ", error.exception_type()),
    }
    report_remaining(&mut filter);
    println!();
}

fn inspect_target(index: i32, failure: i32, parity: i32) -> Result<CellAction, Failure> {
    int(index);
    if index == failure {
        return Err(Failure::out_of_range(&format!(
            "target {index}; expected successful callback"
        )));
    }
    Ok(if (index + parity) % 2 == 0 {
        CellAction::Purge
    } else {
        CellAction::Keep
    })
}

fn report_remaining(filter: &mut CellFilter<PlanarIndices>) {
    filter
        .try_inspect_range_with(&[0., 0.], &[1., 1.], |index| {
            int(*index);
            Ok::<_, ()>(CellAction::Keep)
        })
        .unwrap();
}
