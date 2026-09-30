use super::{flag, integer, range};
use rustycad_tkbo::int_tools::{MarkedRangeSet, Range};
use rustycad_tkernel::standard::Failure;
pub fn case(v: &[f64]) {
    let count = v[0] as usize;
    let mut set = MarkedRangeSet::from_ranges(&v[1..=count], v[count + 1] as i32);
    emit(&set);
    for step in v[count + 2..].as_chunks::<5>().0 {
        let result = insert(&mut set, step);
        show(result);
        emit(&set);
    }
    if set.length() > 0 {
        set.set_flag(1, -7).unwrap();
        emit(&set);
    }
    for value in [-1.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0] {
        query(&mut set, value);
    }
    set.set_boundaries(-2.0, 3.0, 9);
    emit(&set);
    set.set_ranges(&[0.0, 0.0, 1.0], 4);
    emit(&set);
}
fn show(result: Result<bool, Failure>) {
    match result {
        Ok(value) => flag(value),
        Err(error) => print!("{} ", error.exception_type()),
    }
}
fn emit(set: &MarkedRangeSet) {
    integer(set.length());
    for index in 1..=set.length() {
        range(set.range(index).unwrap());
        integer(set.flag(index).unwrap());
    }
}
fn insert(set: &mut MarkedRangeSet, v: &[f64]) -> Result<bool, Failure> {
    match v[0] as i32 {
        0 => set.insert_range(v[1], v[2], v[3] as i32),
        1 => set.insert(&Range::new(v[1], v[2]), v[3] as i32),
        2 => set.insert_range_at(v[1], v[2], v[3] as i32, v[4] as i32),
        _ => set.insert_at(&Range::new(v[1], v[2]), v[3] as i32, v[4] as i32),
    }
}
fn query(set: &mut MarkedRangeSet, value: f64) {
    integer(set.get_index(value).unwrap());
    integer(set.get_index_with_side(value, true).unwrap());
    integer(set.get_index_with_side(value, false).unwrap());
    let indices = set.get_indices(value).unwrap();
    integer(indices.len() as i32);
    for index in indices {
        integer(*index);
    }
}
