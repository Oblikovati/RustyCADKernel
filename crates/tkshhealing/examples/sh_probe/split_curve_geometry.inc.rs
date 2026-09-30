// Shared curve-split protocol; dimension-specific readers and geometry digests live in each module.
use super::common::{Reader, attempt, end, flag, int, real};
use rustycad_tkernel::{
    ncollection::HSequence,
    standard::{Failure, Handle},
};
use rustycad_tkshhealing::{shape_extend::ShapeExtendStatus, shape_upgrade::SplitCurve};
pub fn curve_geometry_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| split_curve_request(&mut reader));
    end();
}
fn split_curve_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let input = read_curve(reader)?;
    let splitter = ProbeSplitter::new();
    let natural = reader.flag();
    let first = reader.real();
    let last = reader.real();
    if natural {
        splitter.init_curve(&input)?;
    } else {
        splitter.init_curve_range(&input, first, last)?;
    }
    print_splitter(&splitter, &ProbeCurves::default())?;
    for _ in 0..reader.integer() {
        let previous = splitter.curves();
        split_curve_operation(&splitter, reader);
        print_splitter(&splitter, &previous)?;
    }
    curve_digest(&input);
    spline_digest(&input);
    Ok(())
}
fn split_curve_operation(splitter: &ProbeSplitter, reader: &mut Reader<'_>) {
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
fn print_splitter(splitter: &ProbeSplitter, previous: &ProbeCurves) -> Result<(), Failure> {
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
pub(super) fn print_curves(curves: &ProbeCurves) -> Result<(), Failure> {
    flag(curves.is_null());
    if curves.is_null() {
        return Ok(());
    }
    int(curves.array1().len() as i32);
    for result in curves.array1().iter() {
        curve_digest(result);
        spline_digest(result);
    }
    Ok(())
}

fn split_change_bounds(splitter: &ProbeSplitter, reader: &mut Reader<'_>) {
    let first = reader.real();
    let last = reader.real();
    attempt(|| {
        splitter.split_values().change_sequence(|s| {
            s.set_value(1, first)?;
            s.set_value(s.len(), last)
        })
    });
}

fn split_add_values(splitter: &ProbeSplitter, reader: &mut Reader<'_>) {
    let values = Handle::new(HSequence::default());
    for _ in 0..reader.integer() {
        values.append(reader.real());
    }
    attempt(|| splitter.set_split_values(&values));
}
