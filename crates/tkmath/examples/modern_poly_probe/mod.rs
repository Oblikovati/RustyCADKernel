//! Direct-polynomial result fields compared with the pinned OCCT headers.
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
