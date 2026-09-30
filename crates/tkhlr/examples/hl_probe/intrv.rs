//! Intrv, the Rust twin of tools/oracle/tkhlr_probe_intrv.cxx: fuzzy intervals, their
//! relative positions and the boolean operations of interval sets.
use super::common::{Reader, attempt, end, flag, int, real};
use rustycad_tkernel::standard::Failure;
use rustycad_tkhlr::intrv::{Interval, Intervals};

/// `mode s ts e te` (see `hl_iv_read`); tolerances are narrowed to `f32` as the native
/// `(float)` cast does.
fn read_interval(r: &mut Reader) -> Interval {
    let mode = r.integer();
    let (s, ts, e, te) = (r.real(), r.real() as f32, r.real(), r.real() as f32);
    match mode {
        1 => Interval::from_bounds(s, e),
        2 => Interval::with_tolerances(s, ts, e, te),
        3 => {
            let mut interval = Interval::new();
            interval.set_start(s, ts);
            interval.set_end(e, te);
            interval
        }
        _ => Interval::new(),
    }
}

fn print_interval(interval: &Interval) {
    let (s, ts, e, te) = interval.bounds();
    for value in [s, f64::from(ts), e, f64::from(te)] {
        real(value);
    }
    real(interval.start());
    real(f64::from(interval.tol_start()));
    real(interval.end());
    real(f64::from(interval.tol_end()));
}

/// `n interval...`: the first interval united with the others in turn.
fn read_set(r: &mut Reader) -> Intervals {
    let count = r.integer();
    let mut set = Intervals::new();
    for i in 0..count {
        let interval = read_interval(r);
        if i == 0 {
            set = Intervals::from_interval(interval);
        } else {
            set.unite_interval(&interval);
        }
    }
    set
}

fn print_set(set: &Intervals) -> Result<(), Failure> {
    int(set.nb_intervals());
    for i in 1..=set.nb_intervals() {
        print_interval(set.value(i)?);
    }
    Ok(())
}

/// `interval`: the bounds as read back, IsProbablyEmpty.
pub fn interval_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let interval = read_interval(&mut r);
    print_interval(&interval);
    flag(interval.is_probably_empty());
    end();
}

/// `interval op value tol`: one bound edit, then the bounds.
pub fn edit_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut interval = read_interval(&mut r);
    let (op, v, t) = (r.integer(), r.real(), r.real() as f32);
    match op {
        0 => interval.set_start(v, t),
        1 => interval.fuse_at_start(v, t),
        2 => interval.cut_at_start(v, t),
        3 => interval.set_end(v, t),
        4 => interval.fuse_at_end(v, t),
        _ => interval.cut_at_end(v, t),
    }
    print_interval(&interval);
    flag(interval.is_probably_empty());
    end();
}

/// `a b`: a.Position(b) and the 13 tests of a against b.
pub fn relate_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (read_interval(&mut r), read_interval(&mut r));
    int(a.position(&b) as i32);
    let tests = [
        a.is_before(&b),
        a.is_after(&b),
        a.is_inside(&b),
        a.is_enclosing(&b),
        a.is_just_enclosing_at_start(&b),
        a.is_just_enclosing_at_end(&b),
        a.is_just_before(&b),
        a.is_just_after(&b),
        a.is_overlapping_at_start(&b),
        a.is_overlapping_at_end(&b),
        a.is_just_overlapping_at_start(&b),
        a.is_just_overlapping_at_end(&b),
        a.is_similar(&b),
    ];
    tests.into_iter().for_each(flag);
    end();
}

/// `op set tool`: one boolean operation, then the resulting set.
pub fn set_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let op = r.integer();
    let mut set = read_set(&mut r);
    let tool = read_set(&mut r);
    attempt(|| {
        apply_set_operation(&mut set, &tool, op)?;
        print_set(&set)
    });
    end();
}

fn apply_set_operation(set: &mut Intervals, tool: &Intervals, op: i32) -> Result<(), Failure> {
    if op >= 4 {
        match op {
            4 => set.intersect(tool),
            5 => set.subtract(tool),
            6 => set.unite(tool),
            _ => set.xunite(tool),
        }
        return Ok(());
    }
    let first = tool.value(1)?;
    match op {
        0 => set.intersect_interval(first),
        1 => set.subtract_interval(first),
        2 => set.unite_interval(first),
        _ => set.xunite_interval(first),
    }
    Ok(())
}

/// `set index`: Value(index), or the raised exception.
pub fn value_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let set = read_set(&mut r);
    let index = r.integer();
    int(set.nb_intervals());
    attempt(|| {
        print_interval(set.value(index)?);
        Ok(())
    });
    end();
}
