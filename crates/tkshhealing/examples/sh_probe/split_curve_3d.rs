//! Three-dimensional split geometry, shared result arrays and native statuses.
use super::{
    brep_lib_common::geom_curve,
    common::{Reader, attempt, end, flag, int, real},
    sh_common::curve,
    shape_construct::construct_spline_3d,
};
use rustycad_tkernel::{
    ncollection::HSequence,
    standard::{Failure, Handle},
};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{ShapeUpgradeSplitCurve3d, SplitCurve, SplitCurves3d},
};
pub fn split_curve_3d_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| split_curve_request(&mut reader));
    end();
}
fn split_curve_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let input = geom_curve(reader)?;
    let splitter = ShapeUpgradeSplitCurve3d::new();
    let natural = reader.flag();
    let first = reader.real();
    let last = reader.real();
    if natural {
        splitter.init_curve(&input)?;
    } else {
        splitter.init_curve_range(&input, first, last)?;
    }
    print_splitter(&splitter, &SplitCurves3d::default())?;
    for _ in 0..reader.integer() {
        let previous = splitter.curves();
        split_curve_operation(&splitter, reader);
        print_splitter(&splitter, &previous)?;
    }
    curve(&input);
    construct_spline_3d(&input);
    Ok(())
}
fn split_curve_operation(splitter: &ShapeUpgradeSplitCurve3d, reader: &mut Reader<'_>) {
    let operation = reader.integer();
    if operation == 0 {
        split_add_values(splitter, reader);
    } else if operation == 1 || operation == 2 {
        let segment = reader.flag();
        attempt(|| {
            if operation == 1 {
                splitter.build(segment)
            } else {
                splitter.perform(segment)
            }
        });
    } else if operation == 3 {
        split_change_bounds(splitter, reader);
    } else {
        let status = reader.integer();
        splitter.split_curve_data().with_mut(|f| f.status = status);
    }
}
fn print_splitter(
    splitter: &ShapeUpgradeSplitCurve3d,
    previous: &SplitCurves3d,
) -> Result<(), Failure> {
    let values = splitter.split_values();
    let values = values.sequence(|s| s.iter().copied().collect::<Vec<_>>());
    int(values.len() as i32);
    for value in values {
        real(value);
    }
    for status in [
        ShapeExtendStatus::Ok,
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Done3,
        ShapeExtendStatus::Fail,
    ] {
        flag(splitter.status_flag(status));
    }
    let curves = splitter.curves();
    flag(curves.object_address() == previous.object_address());
    print_curves(&curves)?;
    print_curves(previous)
}
fn print_curves(curves: &SplitCurves3d) -> Result<(), Failure> {
    flag(curves.is_null());
    if curves.is_null() {
        return Ok(());
    }
    int(curves.array1().len() as i32);
    for result in curves.array1().iter() {
        curve(result);
        construct_spline_3d(result);
    }
    Ok(())
}

fn split_change_bounds(splitter: &ShapeUpgradeSplitCurve3d, reader: &mut Reader<'_>) {
    let first = reader.real();
    let last = reader.real();
    attempt(|| {
        splitter.split_values().change_sequence(|s| {
            s.set_value(1, first)?;
            s.set_value(s.len(), last)
        })
    });
}

fn split_add_values(splitter: &ShapeUpgradeSplitCurve3d, reader: &mut Reader<'_>) {
    let values = Handle::new(HSequence::default());
    for _ in 0..reader.integer() {
        values.append(reader.real());
    }
    attempt(|| splitter.set_split_values(&values));
}
