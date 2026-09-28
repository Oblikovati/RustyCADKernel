//! Trigonometric solutions and periodic count states against OCCT.
use rustycad_tkmath::math_root::*;
fn result(r: TrigResult) {
    print!(
        "{} {} {} ",
        r.status as i32,
        r.nb_roots,
        i32::from(r.infinite_roots)
    );
    for root in r.roots {
        print!("{} ", root.to_bits());
    }
}
pub(super) fn modern_trig_case(v: &[f64]) {
    let mut c: [f64; 5] = v[..5].try_into().unwrap();
    let mut lower: f64 = v[5];
    let mut upper: f64 = v[6];
    let mut eps: f64 = v[7];
    match v[8] as i32 {
        1 => c[0] = f64::NAN,
        2 => upper = f64::INFINITY,
        3 => lower = f64::NAN,
        4 => eps = f64::INFINITY,
        _ => {}
    }
    result(trigonometric(c, lower, upper, eps));
    result(trigonometric_linear(c[3], c[4], lower, upper));
    result(trigonometric_cde(c[2], c[3], c[4], lower, upper));
    for exclude in [false, true] {
        let mut count: usize = 99;
        let ok: bool =
            utils::compute_periodic_root_count(lower, upper, v[9], v[10], exclude, &mut count);
        print!("{} {} ", i32::from(ok), count);
    }
    println!();
}
