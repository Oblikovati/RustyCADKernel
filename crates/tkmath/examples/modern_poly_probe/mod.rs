//! Direct and Laguerre polynomial result fields compared with the pinned OCCT headers.
use rustycad_tkmath::math_poly::*;
fn result(r: PolyResult) {
    print!(
        "{} {} {} {} ",
        r.status as i32,
        r.nb_roots,
        r.nb_complex_roots,
        r.nb_real_roots_with_multiplicity()
    );
    for i in 0..MAX_POLY_DEGREE {
        print!(
            "{} {} {} {} {} ",
            r.roots[i].to_bits(),
            r.multiplicities[i],
            r.complex_roots[i].real().to_bits(),
            r.complex_roots[i].imag().to_bits(),
            r.complex_multiplicities[i]
        );
    }
}
pub(super) fn modern_poly_case(v: &[f64]) {
    let mut c: [f64; 5] = v[..5].try_into().unwrap();
    if v[5] == 1.0 {
        c[0] = f64::NAN;
    } else if v[5] == 2.0 {
        c[4] = f64::INFINITY;
    }
    result(linear(c[0], c[1]));
    result(quadratic(c[0], c[1], c[2]));
    result(cubic(c[0], c[1], c[2], c[3]));
    result(quartic(c[0], c[1], c[2], c[3], c[4]));
    println!();
}
/// Mirrors `modern_laguerre_case`: degree, tolerance, iterations, mode, 21 coefficients.
pub(super) fn modern_laguerre_case(v: &[f64]) {
    let (degree, mut tolerance, iterations, mode) = (v[0] as i32, v[1], v[2] as u32, v[3]);
    let mut c: [f64; 21] = v[4..25].try_into().unwrap();
    if mode == 1.0 {
        c[1] = f64::NAN;
    } else if mode == 2.0 {
        c[0] = f64::NEG_INFINITY;
    } else if mode == 3.0 {
        tolerance = f64::NAN;
    }
    result(match mode as i32 {
        4 => laguerre_n(&sized(&c, degree + 1), tolerance, iterations),
        5 => quintic(c[5], c[4], c[3], c[2], c[1], c[0]),
        6 => sextic(c[6], c[5], c[4], c[3], c[2], c[1], c[0]),
        7 => octic(c[..9].try_into().unwrap()),
        _ => laguerre(&c, degree, tolerance, iterations),
    });
    println!();
}
/// The native size argument over a 21-slot array; sizes past the array read zeros.
fn sized(c: &[f64; 21], size: i32) -> Vec<f64> {
    (0..size.max(0) as usize)
        .map(|i| c.get(i).copied().unwrap_or(0.0))
        .collect()
}
fn complex(z: Complex) {
    print!("{} {} ", z.real().to_bits(), z.imag().to_bits());
}
/// Mirrors `modern_complex_case`: `z = (v0, v1)`, `w = (v2, v3)`, mode injects non-finite parts.
pub(super) fn modern_complex_case(v: &[f64]) {
    let mut p: [f64; 4] = v[..4].try_into().unwrap();
    match v[4] as i32 {
        1 => p[0] = f64::INFINITY,
        2 => p[1] = f64::NAN,
        3 => p[3] = f64::NEG_INFINITY,
        4 => (p[2], p[3]) = (0.0, 0.0),
        _ => {}
    }
    let (z, w) = (Complex::new(p[0], p[1]), Complex::new(p[2], p[3]));
    for value in [
        z * w,
        z / w,
        z.sqrt(),
        Complex::new(p[2], 0.0) / w,
        z + p[3],
        z * p[3],
        z.conj(),
        Complex::polar(p[2].abs(), p[3]),
    ] {
        complex(value);
    }
    println!("{} {}", z.abs().to_bits(), z.norm().to_bits());
}
