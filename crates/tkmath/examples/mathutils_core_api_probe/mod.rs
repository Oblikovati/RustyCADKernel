//! Rust twin of tools/oracle/tkmath_mathutils_core_api_probe.cxx (MathUtils core API).
use rustycad_tkmath::math::Vector;
use rustycad_tkmath::math_utils::*;
mod deriv;
pub(super) use deriv::mathutils_core_deriv_case;

/// Maps the request sentinels for +/-inf and NaN (the native reader cannot parse them).
pub(super) fn decode(values: &[f64]) -> Vec<f64> {
    let map = |v: f64| {
        if v == 7.77e307 {
            f64::INFINITY
        } else if v == -7.77e307 {
            f64::NEG_INFINITY
        } else if v == 6.66e307 {
            f64::NAN
        } else {
            v
        }
    };
    values.iter().map(|&v| map(v)).collect()
}
pub(super) fn out(values: &[f64]) {
    for value in values {
        print!("{} ", value.to_bits());
    }
}
/// Prints bools without separators, like consecutive `std::cout << bool` then one space.
pub(super) fn flags(values: &[bool]) {
    let text: String = values.iter().map(|&v| if v { '1' } else { '0' }).collect();
    print!("{text} ");
}
fn domain1(d: Domain1D, x: f64, t: f64, r: f64) {
    out(&[
        d.min,
        d.max,
        d.length(),
        d.mid(),
        d.clamp(x),
        d.lerp(x),
        d.normalize(x),
    ]);
    flags(&[
        d.is_valid(),
        d.contains(x, t),
        d.is_large(t),
        d.is_full_period(r, t),
        d.is_finite(r),
        d.is_equal(&Domain1D::new(d.max, d.min), t),
    ]);
}
fn domain2(d: Domain2D, x: f64, y: f64, t: f64, r: f64) {
    out(&[
        d.u_min,
        d.u_max,
        d.v_min,
        d.v_max,
        d.u_length(),
        d.v_length(),
        d.u_mid(),
        d.v_mid(),
    ]);
    domain1(d.u(), x, t, r);
    domain1(d.v(), y, t, r);
    let (mut u, mut v): (f64, f64) = (x, y);
    d.clamp(&mut u, &mut v);
    out(&[u, v]);
    flags(&[
        d.is_valid(),
        d.contains(x, y, t),
        d.is_large(t),
        d.is_u_full_period(r, t),
        d.is_v_full_period(r, t),
        d.is_finite(r),
    ]);
}
fn configs(a: f64, b: f64, t: f64, n: u32) {
    let c: Config = Config::new(t, n);
    let bc: BoundedConfig = BoundedConfig::new(a, b, t, n);
    let nc: NDimConfig = NDimConfig::new(t, n, a < b);
    let ic: IntegConfig = IntegConfig::new(t, n);
    let lc: LinConfig = LinConfig {
        singularity_tolerance: t,
        ..Default::default()
    };
    flags(&[
        c.is_valid(),
        bc.is_valid(),
        nc.is_valid(),
        nc.use_bounds,
        ic.is_valid(),
        lc.is_valid(),
    ]);
    out(&[
        c.step_min,
        bc.base.step_min,
        bc.lower_bound,
        bc.upper_bound,
        nc.base.relative_tolerance,
    ]);
}
fn bracket_options(x: f64, a: f64, b: f64, r: f64, n: u32) {
    let mut o: MinBracketOptions = MinBracketOptions {
        max_iterations: n,
        use_limits: true,
        left_limit: a,
        right_limit: b,
        has_fa: r > 0.0,
        fa: x,
        has_fb: r > 1.0,
        fb: r,
    };
    out(&[utils::limited(x, &o)]);
    flags(&[utils::is_valid(&o)]);
    o.use_limits = false;
    out(&[utils::limited(x, &o)]);
    flags(&[utils::is_valid(&o)]);
}
/// `mathutils_core_scalar x a b t r n`.
pub(super) fn mathutils_core_scalar_case(v: &[f64]) {
    let v: &[f64] = &decode(v);
    let [x, a, b, t, r]: [f64; 5] = [v[0], v[1], v[2], v[3], v[4]];
    let n: u32 = v[5] as i32 as u32;
    out(&[
        clamp(x, a, b),
        safe_div(a, b, x),
        sign_transfer(a, b),
        square(x),
        cube(x),
        cube_root(x),
    ]);
    flags(&[
        is_zero(x, t),
        is_equal(a, b, t),
        is_within_tolerance(a, b, t, r),
        is_finite(x),
    ]);
    print!("{} ", sign(x));
    let c: Config = Config::new(t, n);
    flags(&[
        is_x_converged(a, b, t),
        is_f_converged(x, t),
        is_converged(a, b, x, &c),
        is_min_converged(a, b, x, r, &c),
        utils::opposite_signs(a, b),
    ]);
    let (mut minus, mut plus): (f64, f64) = (-7.0, -7.0);
    flags(&[utils::perturbations(x, t, &mut minus, &mut plus)]);
    out(&[minus, plus]);
    configs(a, b, t, n);
    bracket_options(x, a, b, r, n);
    domain1(Domain1D::new(a, b), x, t, r);
    domain2(Domain2D::new(a, b, x, r), a, x, t, r);
    let swapped: Domain2D = Domain2D::from_domains(Domain1D::new(b, a), Domain1D::new(r, x));
    domain2(swapped, b, r, t, x);
    println!();
}
/// `mathutils_core_defaults`: default members and namespace constants.
pub(super) fn mathutils_core_defaults_case(_: &[f64]) {
    let (c, bc, nc) = (
        Config::default(),
        BoundedConfig::default(),
        NDimConfig::default(),
    );
    let (ic, lc) = (IntegConfig::default(), LinConfig::default());
    let (mo, lr) = (MinBracketOptions::default(), LineSearchResult::default());
    let (d1, d2) = (Domain1D::default(), Domain2D::default());
    print!("{} ", c.max_iterations);
    flags(&[c.is_valid(), bc.is_valid(), nc.is_valid(), nc.use_bounds]);
    print!(
        "{} {} {} ",
        ic.initial_order, ic.max_order, ic.max_iterations
    );
    flags(&[ic.is_valid(), lc.use_pivoting, lc.is_valid()]);
    print!("{} ", mo.max_iterations);
    flags(&[mo.use_limits, mo.has_fa, mo.has_fb]);
    print!(
        "{}{} {} ",
        lr.status as i32,
        i32::from(lr.is_valid),
        lr.evaluations
    );
    print!(
        "{} {} {} ",
        NEWTON_MAX_ITER, NEWTON2D_STAGNATION_COUNT, NEWTON2D_LINE_SEARCH_MAX
    );
    out(&[
        c.tolerance,
        c.x_tolerance,
        c.f_tolerance,
        c.relative_tolerance,
        c.step_min,
        bc.lower_bound,
        bc.upper_bound,
        nc.base.tolerance,
        ic.tolerance,
        lc.singularity_tolerance,
        mo.left_limit,
        mo.right_limit,
        mo.fa,
        mo.fb,
        lr.alpha,
        lr.f_new,
        d1.min,
        d1.max,
        d2.u_min,
        d2.u_max,
        d2.v_min,
        d2.v_max,
    ]);
    out(&[
        EPSILON,
        ZERO_TOL,
        PI,
        TWO_PI,
        EULER_NUMBER,
        GOLDEN_RATIO,
        GOLDEN_SECTION,
        NEWTON_FTOL_SQ,
        NEWTON_STEP_TOL_FACTOR,
        NEWTON2D_SINGULAR_DET,
        NEWTON2D_CRITICAL_GRAD_SQ,
        NEWTON2D_MAX_STEP_RATIO,
        NEWTON2D_DOMAIN_EXT,
        NEWTON2D_STAGNATION_RATIO,
        NEWTON2D_SKIP_LINESEARCH_SQ,
        NEWTON2D_STAGNATION_RELAX,
        NEWTON2D_BACKTRACK_TRIGGER,
        NEWTON2D_BACKTRACK_ACCEPT,
        NEWTON2D_MAXITER_RELAX,
        HESSIAN_DEGENERACY_REL,
        HESSIAN_DEGENERACY_ABS,
        ARMIJO_C1,
        EXACT_LINE_SEARCH_ALPHA_MAX,
    ]);
    println!();
}
fn read_vector(values: &[f64]) -> Vector<'static> {
    let mut vector: Vector<'static> = Vector::with_size(values.len());
    for (index, value) in values.iter().enumerate() {
        *vector.change_at(index).unwrap() = *value;
    }
    vector
}
/// `mathutils_core_vector n m x[n] y[m] tol`.
pub(super) fn mathutils_core_vector_case(v: &[f64]) {
    let v: &[f64] = &decode(v);
    let (n, m): (usize, usize) = (v[0] as usize, v[1] as usize);
    let xs: Vector<'_> = read_vector(&v[2..2 + n]);
    let ys: Vector<'_> = read_vector(&v[2 + n..2 + n + m]);
    let t: f64 = v[2 + n + m];
    flags(&[
        is_vector_converged(&xs, &ys, t),
        is_gradient_converged(&xs, t),
        utils::is_finite_vector(&xs),
    ]);
    out(&[infinity_norm(&xs), euclidean_norm(&xs), norm(&xs)]);
    println!();
}
/// `mathutils_core_poly deg c0..c5 x root iter tol count r0..r3`.
pub(super) fn mathutils_core_poly_case(v: &[f64]) {
    let v: &[f64] = &decode(v);
    let degree: i32 = v[0] as i32;
    let c: &[f64] = &v[1..7];
    let (x, root, iterations, tol, count) = (v[7], v[8], v[9] as i32 as u32, v[10], v[11] as usize);
    let mut roots: [f64; 4] = [v[12], v[13], v[14], v[15]];
    let used: &[f64] = &c[..(degree + 1).max(0) as usize];
    let (value, derivative): (f64, f64) = eval_poly_deriv(used, x);
    out(&[
        eval_poly(used, x),
        value,
        derivative,
        eval_poly_desc(used, x),
        refine_poly_root(used, root, iterations),
        refine_poly_root_desc(used, root, iterations),
    ]);
    sort_roots(&mut roots[..count]);
    out(&roots);
    print!("{} ", remove_duplicate_roots(&mut roots[..count], tol));
    out(&roots);
    let (p, q, s): (f64, f64, f64) = depress_cubic(c[1], c[2], c[3]);
    out(&[p, q, s]);
    let (p, q, r, s): (f64, f64, f64, f64) = depress_quartic(c[1], c[2], c[3], c[4]);
    out(&[p, q, r, s]);
    println!();
}
/// `mathutils_core_gauss order size`.
pub(super) fn mathutils_core_gauss_case(v: &[f64]) {
    let v: &[f64] = &decode(v);
    let (order, size): (i32, usize) = (v[0] as i32, v[1] as usize);
    let mut p: Vector<'_> = Vector::with_size(size);
    let mut w: Vector<'_> = Vector::with_size(size);
    type Rule = fn(i32, &mut Vector<'_>, &mut Vector<'_>) -> bool;
    let rules: [Rule; 3] = [
        get_gauss_points_and_weights,
        get_kronrod_points_and_weights,
        get_ordered_gauss_points_and_weights,
    ];
    for rule in rules {
        p.init(-1.0);
        w.init(-1.0);
        flags(&[rule(order, &mut p, &mut w)]);
        for index in 0..size {
            out(&[*p.at(index).unwrap(), *w.at(index).unwrap()]);
        }
    }
    println!();
}
/// `mathutils_core_random seed reseed`.
pub(super) fn mathutils_core_random_case(v: &[f64]) {
    let v: &[f64] = &decode(v);
    let mut generator: RandomGenerator = RandomGenerator::new(v[0] as u64);
    for _ in 0..3 {
        print!("{} ", generator.next_int());
    }
    out(&[generator.next_real(), generator.next_real()]);
    generator.set_seed(v[1] as u64);
    print!("{} ", generator.next_int());
    out(&[generator.next_real()]);
    println!("{}", RandomGenerator::default().next_int());
}
