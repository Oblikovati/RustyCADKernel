use rustycad_tkmath::math::{Function, FunctionRoots, FunctionWithDerivative};
pub(super) struct SampledFunction {
    pub(super) mode: i32,
    pub(super) fail: i32,
    pub(super) calls: i32,
    pub(super) states: i32,
    pub(super) center: f64,
    pub(super) offset: f64,
    pub(super) scale: f64,
}
impl SampledFunction {
    fn val(&self, x: f64) -> f64 {
        let t: f64 = x - self.center;
        match self.mode {
            0 => self.scale * t + self.offset,
            1 => self.scale * t * t + self.offset,
            2 => self.scale * t * t * t + self.offset,
            3 => self.scale * t.sin() + self.offset,
            5 => {
                self.scale
                    * (if t < -1.0 {
                        t + 1.0
                    } else if t > 1.0 {
                        t - 1.0
                    } else {
                        0.0
                    })
                    + self.offset
            }
            6 => {
                self.scale
                    * (if t < -2.0 {
                        t + 2.0
                    } else if t > 2.0 {
                        t - 2.0
                    } else if t.abs() < 1.0 {
                        t * (1.0 - t * t)
                    } else {
                        0.0
                    })
                    + self.offset
            }
            _ => self.offset,
        }
    }
    fn der(&self, x: f64) -> f64 {
        let t: f64 = x - self.center;
        match self.mode {
            0 => self.scale,
            1 => 2.0 * self.scale * t,
            2 => 3.0 * self.scale * t * t,
            3 => self.scale * t.cos(),
            5 => {
                if t.abs() > 1.0 {
                    self.scale
                } else {
                    0.0
                }
            }
            6 => {
                if t.abs() > 2.0 {
                    self.scale
                } else if t.abs() < 1.0 {
                    self.scale * (1.0 - 3.0 * t * t)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
    fn trace(&mut self, kind: char, x: f64) -> bool {
        self.calls += 1;
        let good: bool = self.calls != self.fail;
        print!("{kind} {} {} ", x.to_bits(), i32::from(good));
        good
    }
}
impl Function for SampledFunction {
    fn value(&mut self, x: f64, v: &mut f64) -> bool {
        *v = self.val(x);
        let good: bool = self.trace('v', x);
        print!("{} ", v.to_bits());
        good
    }
    fn state_number(&mut self) -> i32 {
        self.states += 1;
        print!("s {} ", self.states);
        self.states
    }
}
impl FunctionWithDerivative for SampledFunction {
    fn derivative(&mut self, x: f64, d: &mut f64) -> bool {
        *d = self.der(x);
        let good: bool = self.trace('d', x);
        print!("{} ", d.to_bits());
        good
    }
    fn values(&mut self, x: f64, v: &mut f64, d: &mut f64) -> bool {
        *v = self.val(x);
        *d = self.der(x);
        let good: bool = self.trace('c', x);
        print!("{} {} ", v.to_bits(), d.to_bits());
        good
    }
}
pub(super) fn sampled_case(v: &[f64]) {
    let mut f: SampledFunction = SampledFunction {
        mode: v[0] as i32,
        fail: v[1] as i32,
        calls: 0,
        states: 0,
        center: v[2],
        offset: v[3],
        scale: v[4],
    };
    let s: FunctionRoots =
        FunctionRoots::with_options(&mut f, v[5], v[6], v[7] as i32, [v[8], v[9], v[10]], v[11]);
    print!("{} ", i32::from(s.is_done()));
    match s.is_all_null() {
        Ok(v) => print!("{} ", i32::from(v)),
        Err(e) => print!("{} ", e.exception_type()),
    }
    let count: i32 = match s.number_of_solutions() {
        Ok(n) => {
            print!("{n} ");
            n
        }
        Err(e) => {
            print!("{} ", e.exception_type());
            0
        }
    };
    for i in 1..=count {
        print!(
            "{} {} ",
            s.value(i).unwrap().to_bits(),
            s.state_number(i).unwrap()
        );
    }
    for i in [0, count + 1] {
        print_value(&s, i);
        print_state(&s, i);
    }
    println!("{} {} ", f.calls, f.states);
}
fn print_value(s: &FunctionRoots, i: i32) {
    match s.value(i) {
        Ok(v) => print!("{} ", v.to_bits()),
        Err(e) => print!("{} ", e.exception_type()),
    }
}
fn print_state(s: &FunctionRoots, i: i32) {
    match s.state_number(i) {
        Ok(v) => print!("{v} "),
        Err(e) => print!("{} ", e.exception_type()),
    }
}
