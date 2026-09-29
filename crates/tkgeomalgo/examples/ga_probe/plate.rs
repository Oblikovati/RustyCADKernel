//! Plate probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_plate.cxx.
use super::common::{Reader, end, flag, int, real, show, xy, xyz};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::plate::*;
use rustycad_tkmath::gp::{Line3, Plane3, Xy, Xyz};

fn ppc(r: &mut Reader) -> PlatePinpointConstraint {
    let (u, v) = (r.real(), r.real());
    let p = r.xyz();
    let (iu, iv) = (r.integer(), r.integer());
    PlatePinpointConstraint::new(Xy::new(u, v), p, iu, iv)
}

fn print_ppc(c: &PlatePinpointConstraint) {
    xy(c.pnt2d());
    xyz(c.value());
    int(c.idu());
    int(c.idv());
}

fn print_lxyz(c: &PlateLinearXYZConstraint) {
    let k = c.coeff().expect("probe coefficients");
    int(k.nb_rows() as i32);
    int(k.nb_columns() as i32);
    c.get_ppc()
        .expect("probe constraints")
        .iter()
        .for_each(print_ppc);
    k.as_slice().iter().for_each(|&v| real(v));
}

fn print_lsc(c: &PlateLinearScalarConstraint) {
    let k = c.coeff().expect("probe coefficients");
    int(k.nb_rows() as i32);
    int(k.nb_columns() as i32);
    c.get_ppc()
        .expect("probe constraints")
        .iter()
        .for_each(print_ppc);
    k.as_slice().iter().for_each(|&v| xyz(v));
}

fn lxyz(r: &mut Reader) -> Result<PlateLinearXYZConstraint, Failure> {
    let (rows, cols) = (r.integer(), r.integer());
    let c = PlateLinearXYZConstraint::with_size(rows, cols)?;
    for j in 1..=cols {
        c.set_ppc(j, ppc(r))?;
    }
    for i in 1..=rows {
        for j in 1..=cols {
            c.set_coeff(i, j, r.real())?;
        }
    }
    Ok(c)
}

fn lsc(r: &mut Reader) -> Result<PlateLinearScalarConstraint, Failure> {
    let (rows, cols) = (r.integer(), r.integer());
    let c = PlateLinearScalarConstraint::with_size(rows, cols)?;
    for j in 1..=cols {
        c.set_ppc(j, ppc(r))?;
    }
    for i in 1..=rows {
        for j in 1..=cols {
            c.set_coeff(i, j, r.xyz())?;
        }
    }
    Ok(c)
}

/// The ga_plate_report block.
fn report(p: &PlatePlate, uv: &[f64]) {
    flag(p.is_done());
    int(p.continuity());
    show(p.uv_box(), |(a, b, c, d)| {
        [a, b, c, d].into_iter().for_each(real)
    });
    for q in uv.chunks(2).filter(|c| c.len() == 2) {
        let q = Xy::new(q[0], q[1]);
        xyz(p.evaluate(q).expect("probe evaluate"));
        for (iu, iv) in [(1, 0), (0, 1), (1, 1), (2, 0)] {
            xyz(p.evaluate_derivative(q, iu, iv).expect("probe derivative"));
        }
    }
    if p.is_done() {
        let coefs = p.coef_pol().expect("probe coefficients");
        coefs
            .get()
            .expect("probe array")
            .array2()
            .as_slice()
            .iter()
            .for_each(|&v| xyz(v));
    }
    let mut seq: Sequence<Xy> = Sequence::default();
    p.uv_constraints(&mut seq);
    int(seq.length());
}

/// `order aniso pponly npp ppc... nxyz lxyz... nsc lsc... uv...`: the ga_plate_solve_case plate.
pub fn solve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (ord, aniso, pponly) = (r.integer(), r.real(), r.flag());
    let mut p = PlatePlate::new();
    for _ in 0..r.integer() {
        p.load_pinpoint(&ppc(&mut r));
    }
    for _ in 0..r.integer() {
        p.load_linear_xyz(&lxyz(&mut r).expect("probe lxyz"))
            .expect("probe load");
    }
    for _ in 0..r.integer() {
        p.load_linear_scalar(&lsc(&mut r).expect("probe lsc"))
            .expect("probe load");
    }
    let uv = r.reals();
    if let Err(error) = p.solve_ti(ord, aniso, &ProgressRange::default()) {
        print!("{} ", error.exception_type());
    }
    if pponly {
        p.set_polynomial_part_only(true);
    }
    report(&p, &uv);
    report(&p.clone(), &uv);
    let mut a = PlatePlate::new();
    a.load_pinpoint(&PlatePinpointConstraint::new(
        Xy::new(9.0, 9.0),
        Xyz::new(1.0, 1.0, 1.0),
        0,
        0,
    ));
    a.copy_from(&p);
    report(&a, &uv);
    end();
}

fn d1(r: &mut Reader) -> PlateD1 {
    let a = r.xyz();
    PlateD1::new(a, r.xyz())
}

fn d2(r: &mut Reader) -> PlateD2 {
    let (a, b) = (r.xyz(), r.xyz());
    PlateD2::new(a, b, r.xyz())
}

fn d3(r: &mut Reader) -> PlateD3 {
    let (a, b, c) = (r.xyz(), r.xyz(), r.xyz());
    PlateD3::new(a, b, c, r.xyz())
}

/// The ga_plate_solve_with plate: the constraint, three corner values, order 3.
fn solve_with(load: impl FnOnce(&mut PlatePlate) -> Result<(), Failure>) -> Result<(), Failure> {
    let mut p = PlatePlate::new();
    load(&mut p)?;
    let corners = [
        (Xy::new(0.0, 0.0), Xyz::new(0.0, 0.0, 0.0)),
        (Xy::new(1.0, 0.0), Xyz::new(1.0, 0.0, 0.5)),
        (Xy::new(0.0, 1.0), Xyz::new(0.0, 1.0, -0.5)),
    ];
    for (uv, value) in corners {
        p.load_pinpoint(&PlatePinpointConstraint::new(uv, value, 0, 0));
    }
    if let Err(error) = p.solve_ti(3, 1.0, &ProgressRange::default()) {
        print!("{} ", error.exception_type());
    }
    report(&p, &[0.3, 0.4]);
    Ok(())
}

fn gtoc(c: &PlateGtoCConstraint) -> Result<(), Failure> {
    int(c.nb_ppc());
    for i in 0..9 {
        print_ppc(&c.get_ppc(i)?);
    }
    xyz(c.d1_surf_init().du());
    xyz(c.d1_surf_init().dv());
    solve_with(|p| p.load_gtoc(c))
}

fn free(c: &PlateFreeGtoCConstraint) -> Result<(), Failure> {
    int(c.nb_ppc());
    for i in 0..5 {
        print_ppc(&c.get_ppc(i)?);
    }
    int(c.nb_lsc());
    for i in 0..c.nb_lsc() {
        print_lsc(c.lsc(i)?);
    }
    solve_with(|p| p.load_free_gtoc(c))
}

fn constraint(r: &mut Reader, kind: i32, uv: Xy) -> Result<(), Failure> {
    match kind {
        0 | 1 => {
            let (o, d) = (r.pnt(), r.dir());
            let (iu, iv) = (r.integer(), r.integer());
            if kind == 0 {
                let c = PlateLineConstraint::new(uv, &Line3::new(o, d), iu, iv)?;
                print_lsc(c.lsc());
                return solve_with(|p| p.load_line(&c));
            }
            let c = PlatePlaneConstraint::new(uv, &Plane3::new(o, d)?, iu, iv)?;
            print_lsc(c.lsc());
            solve_with(|p| p.load_plane(&c))
        }
        2 => {
            let mut s: Sequence<Xy> = Sequence::default();
            for q in r.reals().chunks(2).filter(|c| c.len() == 2) {
                s.append(Xy::new(q[0], q[1]));
            }
            let c = PlateGlobalTranslationConstraint::new(&s)?;
            print_lxyz(c.lxyzc());
            solve_with(|p| p.load_global_translation(&c))
        }
        3 => {
            let mut s: Sequence<PlatePinpointConstraint> = Sequence::default();
            for _ in 0..r.integer() {
                s.append(ppc(r));
            }
            let c = PlateSampledCurveConstraint::new(&s, r.integer())?;
            print_lxyz(c.lxyzc());
            solve_with(|p| p.load_sampled_curve(&c))
        }
        4..=9 => gtoc_kind(r, kind, uv),
        _ => free_kind(r, kind, uv),
    }
}

fn gtoc_kind(r: &mut Reader, kind: i32, uv: Xy) -> Result<(), Failure> {
    let (s1, t1) = (d1(r), d1(r));
    if kind <= 5 {
        let np = r.xyz();
        let c = if kind == 4 {
            PlateGtoCConstraint::g1(uv, &s1, &t1)
        } else {
            PlateGtoCConstraint::g1_projected(uv, &s1, &t1, np)
        };
        return gtoc(&c);
    }
    let (s2, t2) = (d2(r), d2(r));
    if kind <= 7 {
        let np = r.xyz();
        let c = if kind == 6 {
            PlateGtoCConstraint::g2(uv, &s1, &t1, &s2, &t2)?
        } else {
            PlateGtoCConstraint::g2_projected(uv, (&s1, &t1), (&s2, &t2), np)?
        };
        return gtoc(&c);
    }
    let (s3, t3) = (d3(r), d3(r));
    let np = r.xyz();
    let c = if kind == 8 {
        PlateGtoCConstraint::g3(uv, (&s1, &t1), (&s2, &t2), (&s3, &t3))?
    } else {
        PlateGtoCConstraint::g3_projected(uv, (&s1, &t1), (&s2, &t2), (&s3, &t3), np)?
    };
    gtoc(&c)
}

fn free_kind(r: &mut Reader, kind: i32, uv: Xy) -> Result<(), Failure> {
    let (s1, t1) = (d1(r), d1(r));
    if kind == 10 {
        let (load, o) = (r.real(), r.integer());
        return free(&PlateFreeGtoCConstraint::g1(uv, &s1, &t1, load, o)?);
    }
    let (s2, t2) = (d2(r), d2(r));
    if kind == 11 {
        let (load, o) = (r.real(), r.integer());
        return free(&PlateFreeGtoCConstraint::g2(
            uv,
            (&s1, &t1),
            (&s2, &t2),
            load,
            o,
        )?);
    }
    let (s3, t3) = (d3(r), d3(r));
    let (load, o) = (r.real(), r.integer());
    free(&PlateFreeGtoCConstraint::g3(
        uv,
        (&s1, &t1),
        (&s2, &t2),
        (&s3, &t3),
        load,
        o,
    )?)
}

/// `kind args...`: the ga_plate_constraint_case constraints.
pub fn constraint_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let uv = Xy::new(r.real(), r.real());
    if let Err(error) = constraint(&mut r, kind, uv) {
        print!("{} ", error.exception_type());
    }
    end();
}
