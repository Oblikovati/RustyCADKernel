use super::sampled_probe::SampledFunction;
use rustycad_tkmath::math::{FunctionAllRoots, FunctionSample};
pub(super) fn all_roots_case(v: &[f64]) {
    let mut f: SampledFunction = SampledFunction {
        mode: v[0] as i32,
        fail: v[1] as i32,
        center: v[2],
        offset: v[3],
        scale: v[4],
        calls: 0,
        states: 0,
    };
    let sample: FunctionSample = FunctionSample::new(v[5], v[6], v[7] as i32);
    let [a, b]: [f64; 2] = sample.bounds();
    let count: i32 = sample.number_of_points();
    print!("{} {} {count} ", a.to_bits(), b.to_bits());
    for index in [0, 1, count, count + 1] {
        print!("{} ", sample.parameter(index).to_bits());
    }
    match FunctionAllRoots::new(&mut f, &sample, [v[8], v[9], v[10]]) {
        Ok(s) => results(&s),
        Err(e) => print!("{} ", e.exception_type()),
    }
    println!("{} {} ", f.calls, f.states);
}
fn results(s: &FunctionAllRoots) {
    print!(
        "{} {} {} ",
        i32::from(s.is_done()),
        s.number_of_intervals().unwrap(),
        s.number_of_points().unwrap()
    );
    for i in 1..=s.number_of_intervals().unwrap() {
        let [a, b]: [f64; 2] = s.interval(i).unwrap();
        let [first, last]: [i32; 2] = s.interval_state(i).unwrap();
        print!("{} {} {first} {last} ", a.to_bits(), b.to_bits());
    }
    for i in 1..=s.number_of_points().unwrap() {
        print!(
            "{} {} ",
            s.point(i).unwrap().to_bits(),
            s.point_state(i).unwrap()
        );
    }
    for i in [0, s.number_of_intervals().unwrap() + 1] {
        print!(
            "{} {} ",
            s.interval(i).unwrap_err().exception_type(),
            s.interval_state(i).unwrap_err().exception_type()
        );
    }
    for i in [0, s.number_of_points().unwrap() + 1] {
        print!(
            "{} {} ",
            s.point(i).unwrap_err().exception_type(),
            s.point_state(i).unwrap_err().exception_type()
        );
    }
}
