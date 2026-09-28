//! Counterparts of tkmath_bspl_curves_probe.cxx: insertion, removal, degree
//! elevation, unperiodization and trimming on flat, 2D and 3D poles.
use super::{In, fail, oa, oi, oia, opa};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::plib::PolePoint;

/// kind (1 flat, 2 Point2, 3 Point3) dim rational deg periodic knots mults poles weights.
pub struct Curve {
    pub kind: i32,
    pub dim: i32,
    pub rat: bool,
    pub deg: i32,
    pub per: bool,
    pub k: Array1<'static, f64>,
    pub m: Array1<'static, i32>,
    pub f: Array1<'static, f64>,
    pub p2: Array1<'static, Point2>,
    pub p3: Array1<'static, Point3>,
    pub w: Array1<'static, f64>,
}

impl Curve {
    pub fn read(input: &mut In<'_>) -> Self {
        let (kind, dim, rat, deg, per) = (input.i(), input.i(), input.b(), input.i(), input.b());
        let (k, m) = (input.reals(), input.ints());
        let mut curve = Self {
            kind,
            dim,
            rat,
            deg,
            per,
            k,
            m,
            f: Array1::default(),
            p2: Array1::default(),
            p3: Array1::default(),
            w: Array1::default(),
        };
        match kind {
            1 => curve.f = input.reals(),
            2 => curve.p2 = input.points(),
            _ => curve.p3 = input.points(),
        }
        curve.w = input.reals();
        curve
    }
    pub fn w(&self) -> Option<&Array1<'static, f64>> {
        if self.rat { Some(&self.w) } else { None }
    }
}

/// Output arrays of a curve operation, filled with sentinels.
pub struct Out {
    pub f: Array1<'static, f64>,
    pub w: Array1<'static, f64>,
    pub k: Array1<'static, f64>,
    pub m: Array1<'static, i32>,
    pub p2: Array1<'static, Point2>,
    pub p3: Array1<'static, Point3>,
}

fn filled<T: Default + Clone>(count: i32, value: T) -> Array1<'static, T> {
    let mut array: Array1<'static, T> = Array1::with_bounds(1, count).unwrap();
    array.init(value);
    array
}

impl Out {
    pub fn new(c: &Curve, np: i32, nk: i32) -> Self {
        Self {
            f: filled(if c.kind == 1 { np * c.dim } else { 0 }, 91.0),
            w: filled(np, 91.0),
            k: filled(nk, 91.0),
            m: filled(nk, -7),
            p2: filled(if c.kind == 2 { np } else { 0 }, Point2::new(91.0, 91.0)),
            p3: filled(
                if c.kind == 3 { np } else { 0 },
                Point3::new(91.0, 91.0, 91.0),
            ),
        }
    }
    pub fn print(&self, c: &Curve, knots: bool) {
        match c.kind {
            1 => oa(&self.f),
            2 => opa(&self.p2),
            _ => opa(&self.p3),
        }
        if c.rat {
            oa(&self.w);
        }
        if knots {
            oa(&self.k);
            oia(&self.m);
        }
    }
}

fn insert_points<P: PolePoint>(
    c: &Curve,
    poles: &Array1<'_, P>,
    new_poles: &mut Array1<'_, P>,
    r_w: Option<&mut Array1<'static, f64>>,
    knots: (&Array1<'_, f64>, Option<&Array1<'_, i32>>, f64, bool),
    r_k: &mut Array1<'static, f64>,
    r_m: &mut Array1<'static, i32>,
) -> Result<(), Failure> {
    let (add_knots, add_mults, tol, add) = knots;
    bsplclib::insert_knots(
        c.deg,
        c.per,
        poles,
        c.w(),
        &c.k,
        &c.m,
        add_knots,
        add_mults,
        new_poles,
        r_w,
        r_k,
        r_m,
        tol,
        add,
    )
}

// insert: curve addknots useaddmults addmults tol add | single: u umult | raise: index mult
pub fn insert_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (ak, use_am, am, tol, add) = (f.reals(), f.b(), f.ints(), f.r(), f.b());
    let add_mults = if use_am { Some(&am) } else { None };
    let (mut np, mut nk): (i32, i32) = (-7, -7);
    let ok: bool = bsplclib::prepare_insert_knots(
        c.deg, c.per, &c.k, &c.m, &ak, add_mults, &mut np, &mut nk, tol, add,
    )
    .unwrap();
    oi(i32::from(ok));
    oi(np);
    oi(nk);
    if ok {
        let mut r = Out::new(&c, np, nk);
        let knots = (&ak, add_mults, tol, add);
        let Out {
            f: rf,
            w: rw,
            k: rk,
            m: rm,
            p2: rp2,
            p3: rp3,
        } = &mut r;
        let rw = if c.rat { Some(rw) } else { None };
        let result = match c.kind {
            1 => bsplclib::insert_knots_flat(
                c.deg, c.per, c.dim, &c.f, &c.k, &c.m, &ak, add_mults, rf, rk, rm, tol, add,
            ),
            2 => insert_points(&c, &c.p2, rp2, rw, knots, rk, rm),
            _ => insert_points(&c, &c.p3, rp3, rw, knots, rk, rm),
        };
        match result {
            Ok(()) => r.print(&c, true),
            Err(failure) => fail(&failure),
        }
    }
    let (u, umult, index, mult) = (f.r(), f.i(), f.i(), f.i());
    if c.kind != 1 {
        single_insertions(&c, u, umult, index, mult);
    }
    println!();
}

fn single_insertions(c: &Curve, u: f64, umult: i32, index: i32, mult: i32) {
    let one = |value: f64| filled(1, value);
    let (mut np, mut nk): (i32, i32) = (-7, -7);
    let eps_u: f64 = epsilon(u);
    if bsplclib::prepare_insert_knots(
        c.deg,
        c.per,
        &c.k,
        &c.m,
        &one(u),
        Some(&filled(1, umult)),
        &mut np,
        &mut nk,
        eps_u,
        true,
    )
    .unwrap()
    {
        let mut r = Out::new(c, np, 0);
        oi(np);
        let result = match c.kind {
            2 => bsplclib::insert_knot(
                0,
                u,
                umult,
                c.deg,
                c.per,
                &c.p2,
                c.w(),
                &c.k,
                &c.m,
                &mut r.p2,
                if c.rat { Some(&mut r.w) } else { None },
            ),
            _ => bsplclib::insert_knot(
                0,
                u,
                umult,
                c.deg,
                c.per,
                &c.p3,
                c.w(),
                &c.k,
                &c.m,
                &mut r.p3,
                if c.rat { Some(&mut r.w) } else { None },
            ),
        };
        print_result(result, &r, c);
    }
    let knot: f64 = c.k[index];
    let added: i32 = mult - c.m[index];
    if bsplclib::prepare_insert_knots(
        c.deg,
        c.per,
        &c.k,
        &c.m,
        &one(knot),
        Some(&filled(1, added)),
        &mut np,
        &mut nk,
        epsilon(knot),
        true,
    )
    .unwrap()
    {
        let mut r = Out::new(c, np, 0);
        oi(np);
        let result = match c.kind {
            2 => bsplclib::raise_multiplicity(
                index,
                mult,
                c.deg,
                c.per,
                &c.p2,
                c.w(),
                &c.k,
                &c.m,
                &mut r.p2,
                if c.rat { Some(&mut r.w) } else { None },
            ),
            _ => bsplclib::raise_multiplicity(
                index,
                mult,
                c.deg,
                c.per,
                &c.p3,
                c.w(),
                &c.k,
                &c.m,
                &mut r.p3,
                if c.rat { Some(&mut r.w) } else { None },
            ),
        };
        print_result(result, &r, c);
    }
}

fn print_result(result: Result<(), Failure>, r: &Out, c: &Curve) {
    match result {
        Ok(()) => r.print(c, false),
        Err(failure) => fail(&failure),
    }
}

/// Standard_Real Epsilon, as the native probe calls it.
pub fn epsilon(value: f64) -> f64 {
    let magnitude: f64 = value.abs();
    if magnitude == f64::MAX {
        return 0.0;
    }
    if magnitude == f64::INFINITY {
        return f64::NEG_INFINITY;
    }
    magnitude.next_up() - magnitude
}

// remove: curve index mult tol newpoles newknots
pub fn remove_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (index, mult, tol, np, nk) = (f.i(), f.i(), f.r(), f.i(), f.i());
    let mut r = Out::new(&c, np, nk);
    let Out {
        f: rf,
        w: rw,
        k: rk,
        m: rm,
        p2: rp2,
        p3: rp3,
    } = &mut r;
    let rw = if c.rat { Some(rw) } else { None };
    let result = match c.kind {
        1 => bsplclib::remove_knot_flat(
            index, mult, c.deg, c.per, c.dim, &c.f, &c.k, &c.m, rf, rk, rm, tol,
        ),
        2 => bsplclib::remove_knot(
            index,
            mult,
            c.deg,
            c.per,
            &c.p2,
            c.w(),
            &c.k,
            &c.m,
            rp2,
            rw,
            rk,
            rm,
            tol,
        ),
        _ => bsplclib::remove_knot(
            index,
            mult,
            c.deg,
            c.per,
            &c.p3,
            c.w(),
            &c.k,
            &c.m,
            rp3,
            rw,
            rk,
            rm,
            tol,
        ),
    };
    match result {
        Ok(ok) => {
            oi(i32::from(ok));
            r.print(&c, true);
        }
        Err(failure) => fail(&failure),
    }
    println!();
}

// degree: curve newdeg newpoles
pub fn degree_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (nd, np) = (f.i(), f.i());
    let nk: i32 = match bsplclib::increase_degree_count_knots(c.deg, nd, c.per, &c.m) {
        Ok(nk) => nk,
        Err(failure) => {
            fail(&failure);
            println!();
            return;
        }
    };
    oi(nk);
    let mut r = Out::new(&c, np, nk);
    let Out {
        f: rf,
        w: rw,
        k: rk,
        m: rm,
        p2: rp2,
        p3: rp3,
    } = &mut r;
    let rw = if c.rat { Some(rw) } else { None };
    let result = match c.kind {
        1 => bsplclib::increase_degree_flat(c.deg, nd, c.per, c.dim, &c.f, &c.k, &c.m, rf, rk, rm),
        2 => bsplclib::increase_degree(c.deg, nd, c.per, &c.p2, c.w(), &c.k, &c.m, rp2, rw, rk, rm),
        _ => bsplclib::increase_degree(c.deg, nd, c.per, &c.p3, c.w(), &c.k, &c.m, rp3, rw, rk, rm),
    };
    print_with_knots(result, &r, &c);
    println!();
}

fn print_with_knots(result: Result<(), Failure>, r: &Out, c: &Curve) {
    match result {
        Ok(()) => r.print(c, true),
        Err(failure) => fail(&failure),
    }
}

// unperiodize: curve
pub fn unperiodize_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (nk, np) = match bsplclib::prepare_unperiodize(c.deg, &c.m) {
        Ok(counts) => counts,
        Err(failure) => {
            fail(&failure);
            println!();
            return;
        }
    };
    oi(nk);
    oi(np);
    let mut r = Out::new(&c, np, nk);
    let Out {
        f: rf,
        w: rw,
        k: rk,
        m: rm,
        p2: rp2,
        p3: rp3,
    } = &mut r;
    let rw = if c.rat { Some(rw) } else { None };
    let result = match c.kind {
        1 => bsplclib::unperiodize_flat(c.deg, c.dim, &c.m, &c.k, &c.f, rm, rk, rf),
        2 => bsplclib::unperiodize(c.deg, &c.m, &c.k, &c.p2, c.w(), rm, rk, rp2, rw),
        _ => bsplclib::unperiodize(c.deg, &c.m, &c.k, &c.p3, c.w(), rm, rk, rp3, rw),
    };
    print_with_knots(result, &r, &c);
    println!();
}

// trim: curve u1 u2
pub fn trim_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (u1, u2) = (f.r(), f.r());
    let (nk, np) = match bsplclib::prepare_trimming(c.deg, c.per, &c.k, &c.m, u1, u2) {
        Ok(counts) => counts,
        Err(failure) => {
            fail(&failure);
            println!();
            return;
        }
    };
    oi(nk);
    oi(np);
    let mut r = Out::new(&c, np, nk);
    let Out {
        f: rf,
        w: rw,
        k: rk,
        m: rm,
        p2: rp2,
        p3: rp3,
    } = &mut r;
    let rw = if c.rat { Some(rw) } else { None };
    let result = match c.kind {
        1 => bsplclib::trimming_flat(c.deg, c.per, c.dim, &c.k, &c.m, &c.f, u1, u2, rk, rm, rf),
        2 => bsplclib::trimming(
            c.deg,
            c.per,
            &c.k,
            &c.m,
            &c.p2,
            c.w(),
            u1,
            u2,
            rk,
            rm,
            rp2,
            rw,
        ),
        _ => bsplclib::trimming(
            c.deg,
            c.per,
            &c.k,
            &c.m,
            &c.p3,
            c.w(),
            u1,
            u2,
            rk,
            rm,
            rp3,
            rw,
        ),
    };
    print_with_knots(result, &r, &c);
    println!();
}
