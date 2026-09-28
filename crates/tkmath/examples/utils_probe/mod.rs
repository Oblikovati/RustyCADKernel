//! Differential MathUtils counterpart for the pinned OCCT revision.
use rustycad_tkmath::{math::Vector, math_utils::*};
pub(super) fn core_case(v: &[f64]) {
    let n: usize = v[0] as usize;
    let mut a: Vector<'_> = Vector::with_size(n);
    let mut b: Vector<'_> = Vector::with_size(n);
    for i in 0..n {
        *a.change_at(i).unwrap() = v[1 + i];
        *b.change_at(i).unwrap() = v[1 + n + i];
    }
    for value in [
        dot_product(&a, &b),
        vector_norm(&a),
        norm2(&a),
        vector_inf_norm(&a),
        compute_scale_factor(&v[1..1 + n]),
    ] {
        print!("{} ", value.to_bits());
    }
    println!();
}
pub(super) fn bracket_case(v: &[f64]) {
    let mode: i32 = v[0] as i32;
    let center: f64 = v[1];
    let mut calls: Vec<f64> = Vec::new();
    let mut function = |x: f64, value: &mut f64| {
        calls.push(x);
        *value = if mode == 0 {
            x - center
        } else {
            (x - center) * (x - center)
        };
        if mode == 2 && x > center {
            return false;
        }
        if mode == 3 && x > center {
            *value = f64::NAN;
        }
        if mode == 4 {
            *value = 1.0;
        }
        true
    };
    let root: BracketResult = bracket_root(&mut function, v[2], v[3], v[4] as u32);
    print!(
        "{} {} {} {} {} ",
        root.status as i32,
        i32::from(root.is_valid),
        root.iterations,
        i32::from(root.has_fa),
        i32::from(root.has_fb)
    );
    for value in [root.a, root.b, root.fa, root.fb] {
        print!("{} ", value.to_bits());
    }
    let options: MinBracketOptions = MinBracketOptions {
        max_iterations: v[4] as u32,
        use_limits: v[5] != v[6],
        left_limit: v[5],
        right_limit: v[6],
        ..Default::default()
    };
    let minimum: MinBracketResult = bracket_minimum(&mut function, v[2], v[3], &options);
    print!(
        "{} {} {} ",
        minimum.status as i32,
        i32::from(minimum.is_valid),
        minimum.iterations
    );
    for value in [
        minimum.a, minimum.b, minimum.c, minimum.fa, minimum.fb, minimum.fc,
    ] {
        print!("{} ", value.to_bits());
    }
    print!("{} ", calls.len());
    for x in calls {
        print!("{} ", x.to_bits());
    }
    println!();
}

mod line;
pub(super) use line::line_case;
