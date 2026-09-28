//! Specialized Newton callback traces, including symmetric value-only evaluations.
use rustycad_tkmath::math_sys::*;
struct Fixture<const N: usize> {
    mode: i32,
    fail: usize,
    scale: f64,
    calls: Vec<(i32, [f64; N])>,
}
impl<const N: usize> Fixture<N> {
    fn record(&mut self, kind: i32, x: &[f64; N]) -> bool {
        self.calls.push((kind, *x));
        self.calls.len() != self.fail
    }
    fn entry(&self, r: usize, c: usize) -> f64 {
        match self.mode {
            2 | 9 => self.scale * if self.mode == 9 { -1.0 } else { 1.0 },
            4 => 0.0,
            7 | 8 => {
                if r == c {
                    if r == 0 { 0.01 } else { 2.0 }
                } else if (r + c).is_multiple_of(2) {
                    1.5
                } else {
                    -1.0
                }
            }
            _ => {
                if r == c {
                    if r == 0 { self.scale } else { 1.0 }
                } else {
                    0.0
                }
            }
        }
    }
    fn fill(&self, x: &[f64; N], f: &mut [f64; N], j: &mut [[f64; N]; N]) {
        for r in 0..N {
            j[r] = std::array::from_fn(|c| self.entry(r, c));
            f[r] = (0..N).fold(0.0, |v, c| v + j[r][c] * (x[c] - (c + 1) as f64));
            match self.mode {
                1 => {
                    f[r] = x[r].atan();
                    j[r][r] = 1.0 / (1.0 + x[r] * x[r]);
                }
                3 => {
                    f[r] = 1.0;
                    j[r][r] = 1e20;
                }
                4 => f[r] = 1.0,
                5 => f[r] = f64::NAN,
                6 => {
                    f[r] = j[r][r] * (x[r] * x[r] - (r + 1) as f64);
                    j[r][r] = j[r][r] * 2.0 * x[r];
                }
                _ => {}
            }
        }
    }
}
impl<const N: usize> SmallSystem<N> for Fixture<N> {
    fn evaluate(&mut self, x: &[f64; N], f: &mut [f64; N], j: &mut [[f64; N]; N]) -> bool {
        if !self.record(1, x) {
            return false;
        }
        self.fill(x, f, j);
        true
    }
}
impl SymmetricFunction2D for Fixture<2> {
    fn value(&mut self, x: &[f64; 2], f: &mut [f64; 2]) -> bool {
        if !self.record(0, x) {
            return false;
        }
        self.fill(x, f, &mut [[0.0; 2]; 2]);
        true
    }
    fn value_and_jacobian(&mut self, x: &[f64; 2], f: &mut [f64; 2], j: &mut [f64; 3]) -> bool {
        if !self.record(1, x) {
            return false;
        }
        let mut matrix: [[f64; 2]; 2] = [[0.0; 2]; 2];
        self.fill(x, f, &mut matrix);
        *j = [matrix[0][0], matrix[0][1], matrix[1][1]];
        true
    }
}
fn result<const N: usize>(r: NewtonResultN<N>, f: &Fixture<N>) {
    print!(
        "{} {} {} {} {} {} ",
        r.status as i32,
        r.iterations,
        u8::from(r.is_done()),
        u8::from(bool::from(&r)),
        r.residual_norm.to_bits(),
        r.step_norm.to_bits()
    );
    for v in r.x {
        print!("{} ", v.to_bits());
    }
    print!("{} ", f.calls.len());
    for (kind, x) in &f.calls {
        print!("{kind} ");
        for v in x {
            print!("{} ", v.to_bits());
        }
    }
}
fn setup<const N: usize>(v: &[f64]) -> (Fixture<N>, [f64; N], NewtonBoundsN<N>) {
    (
        Fixture {
            mode: v[0] as i32,
            fail: v[1] as usize,
            scale: v[2],
            calls: Vec::new(),
        },
        std::array::from_fn(|i| v[3 + i % 2]),
        NewtonBoundsN {
            min: [v[13]; N],
            max: [v[14]; N],
            has_bounds: v[10] != 0.0,
        },
    )
}
pub fn modern_small_case(v: &[f64]) {
    let cfg: NewtonOptions = NewtonOptions {
        solver: rustycad_tkmath::math_utils::Config {
            f_tolerance: v[5],
            x_tolerance: v[6],
            max_iterations: v[7] as u32,
            ..Default::default()
        },
        max_step_ratio: v[8],
        enable_line_search: v[9] != 0.0,
        allow_soft_bounds: v[11] != 0.0,
        soft_bounds_extension: v[12],
    };
    let (mut f, x, b) = setup::<2>(v);
    result(solve_2d(&mut f, x, &b, &cfg), &f);
    f.calls.clear();
    result(solve_2d_symmetric(&mut f, x, &b, &cfg), &f);
    let (mut f, x, b) = setup::<3>(v);
    result(solve_3d(&mut f, x, &b, &cfg), &f);
    let (mut f, x, b) = setup::<4>(v);
    result(solve_4d(&mut f, x, &b, &cfg), &f);
    println!();
}
