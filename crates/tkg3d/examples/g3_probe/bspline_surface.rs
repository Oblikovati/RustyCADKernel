//! The Geom_BSplineSurface members that the generic surface requests do not reach,
//! and the kind 8 surface specification; the counterpart of
//! tools/oracle/tkg3d_probe_bspline_surface.cxx.
use super::common::{
    Reader, attempt, curve_summary, dump, end, flag, int, name, pnt, real, show, surface_summary,
    trsf, vec,
};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomGeometry, GeomSurface};
use rustycad_tkmath::bsplslib::MoveRange;
use rustycad_tkmath::gp::Point3;

fn knot_vector(r: &mut Reader) -> (Array1<'static, f64>, Array1<'static, i32>) {
    let n = r.integer();
    let mut k = Array1::<f64>::with_bounds(1, n).unwrap();
    let mut m = Array1::<i32>::with_bounds(1, n).unwrap();
    for i in 1..=n {
        k[i] = r.real();
        m[i] = r.integer();
    }
    (k, m)
}

/// Kind 8: bspline rational uperiodic vperiodic udeg vdeg nuk (k m)* nvk (k m)* nu nv
/// (x y z w)*(nu*nv), as g3_bspline_surface reads it.
pub fn spec(r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (rational, up, vp) = (r.flag(), r.flag(), r.flag());
    let (ud, vd) = (r.integer(), r.integer());
    let (uk, um) = knot_vector(r);
    let (vk, vm) = knot_vector(r);
    let (nu, nv) = (r.integer(), r.integer());
    let mut p = Array2::<Point3>::with_bounds(1, nu, 1, nv).unwrap();
    let mut w = Array2::<f64>::with_bounds(1, nu, 1, nv).unwrap();
    for i in 1..=nu {
        for j in 1..=nv {
            p.set_value(i, j, r.pnt()).unwrap();
            w.set_value(i, j, r.real()).unwrap();
        }
    }
    let surface = if rational {
        GeomBSplineSurface::with_weights(&p, &w, (&uk, &um), (&vk, &vm), (ud, vd), (up, vp))?
    } else {
        GeomBSplineSurface::new(&p, (&uk, &um), (&vk, &vm), (ud, vd), (up, vp))?
    };
    Ok(Handle::new(surface).cast())
}

fn int_array(a: &Array1<'_, i32>) {
    int(a.lower());
    int(a.upper());
    a.as_slice().iter().copied().for_each(int);
}
fn real_array(a: &Array1<'_, f64>) {
    int(a.lower());
    int(a.upper());
    a.as_slice().iter().copied().for_each(real);
}

fn state(s: &GeomBSplineSurface) {
    [
        s.nb_u_poles(),
        s.nb_v_poles(),
        s.nb_u_knots(),
        s.nb_v_knots(),
        s.u_degree(),
        s.v_degree(),
    ]
    .into_iter()
    .for_each(int);
    [
        s.is_u_rational(),
        s.is_v_rational(),
        s.is_u_periodic(),
        s.is_v_periodic(),
    ]
    .into_iter()
    .for_each(flag);
    int(s.u_knot_distribution() as i32);
    int(s.v_knot_distribution() as i32);
    int(s.continuity() as i32);
    show(s.first_u_knot_index(), int);
    show(s.last_u_knot_index(), int);
    show(s.first_v_knot_index(), int);
    show(s.last_v_knot_index(), int);
    real_array(&s.u_knots());
    real_array(&s.v_knots());
    int_array(&s.u_multiplicities());
    int_array(&s.v_multiplicities());
    real_array(&s.u_knot_sequence());
    real_array(&s.v_knot_sequence());
    let p = s.poles();
    for i in p.lower_row()..=p.upper_row() {
        for j in p.lower_col()..=p.upper_col() {
            pnt(*p.value(i, j).unwrap());
            real(s.weight(i, j).unwrap());
        }
    }
    flag(s.weights().is_some());
    flag(s.has_eval_representation());
    let (u1, u2, v1, v2) = s.bounds();
    [u1, u2, v1, v2].into_iter().for_each(real);
}

fn one_based<T: Default + Clone>(n: i32) -> Array1<'static, T> {
    Array1::with_bounds(1, n).unwrap()
}

fn getters(s: &GeomBSplineSurface, i: i32, j: i32) {
    show(s.pole(i, j), pnt);
    show(s.weight(i, j), real);
    show(s.u_knot(i), real);
    show(s.v_knot(j), real);
    show(s.u_multiplicity(i), int);
    show(s.v_multiplicity(j), int);
    let extra = |cond: bool| i32::from(cond);
    let mut p =
        Array2::<Point3>::with_bounds(1, s.nb_u_poles() + extra(i > 2), 1, s.nb_v_poles()).unwrap();
    show(s.poles_into(&mut p), |()| pnt(*p.value(1, 1).unwrap()));
    let mut w =
        Array2::<f64>::with_bounds(1, s.nb_u_poles(), 1, s.nb_v_poles() + extra(j > 2)).unwrap();
    show(s.weights_into(&mut w), |()| {
        real(*w.value(1, 1).unwrap());
        real(*w.value(w.upper_row(), w.upper_col()).unwrap());
    });
    let mut ku = one_based::<f64>(s.nb_u_knots() + extra(i > 2));
    show(s.u_knots_into(&mut ku), |()| real_array(&ku));
    let mut kv = one_based::<f64>(s.nb_v_knots());
    show(s.v_knots_into(&mut kv), |()| real_array(&kv));
    let mut su = one_based::<f64>(s.u_knot_sequence().len() as i32);
    show(s.u_knot_sequence_into(&mut su), |()| real_array(&su));
    let mut sv = one_based::<f64>(s.v_knot_sequence().len() as i32 + extra(j > 2));
    show(s.v_knot_sequence_into(&mut sv), |()| real_array(&sv));
    let mut mu = one_based::<i32>(s.nb_u_knots());
    show(s.u_multiplicities_into(&mut mu), |()| int_array(&mu));
    let mut mv = one_based::<i32>(s.nb_v_knots() + extra(i > 2));
    show(s.v_multiplicities_into(&mut mv), |()| int_array(&mv));
}

fn local(s: &GeomBSplineSurface, u: f64, v: f64, knots: ((i32, i32), (i32, i32))) {
    show(s.local_value(u, v, knots), pnt);
    show(s.local_d1(u, v, knots), |d| {
        pnt(d.point);
        [d.d1u, d.d1v].into_iter().for_each(vec);
    });
    show(s.local_d2(u, v, knots), |d| {
        pnt(d.point);
        [d.d1u, d.d1v, d.d2u, d.d2v, d.d2uv]
            .into_iter()
            .for_each(vec);
    });
    show(s.local_d3(u, v, knots), |d| {
        pnt(d.point);
        [
            d.d1u, d.d1v, d.d2u, d.d2v, d.d2uv, d.d3u, d.d3v, d.d3uuv, d.d3uvv,
        ]
        .into_iter()
        .for_each(vec);
    });
    for nu in 0..=2 {
        for nv in 0..=2 {
            show(s.local_dn(u, v, knots, (nu, nv)), vec);
        }
    }
}

fn queries(s: &GeomBSplineSurface, r: &mut Reader) {
    let uv = r.reals();
    let knots = ((r.integer(), r.integer()), (r.integer(), r.integer()));
    let tol = r.reals();
    for pair in uv.chunks(2).filter(|pair| pair.len() == 2) {
        let (u, v) = (pair[0], pair[1]);
        local(s, u, v, knots);
        show(s.periodic_normalization(u, v), |(a, b)| {
            real(a);
            real(b);
        });
        for &t in &tol {
            for rep in [false, true] {
                show(s.locate_u(u, t, rep), |(a, b)| {
                    [a, b].into_iter().for_each(int)
                });
                show(s.locate_v(v, t, rep), |(a, b)| {
                    [a, b].into_iter().for_each(int)
                });
            }
        }
        for check in [false, true] {
            for (iso, is_u) in [(u, true), (v, false)] {
                let curve = if is_u {
                    s.u_iso_checked(iso, check)
                } else {
                    s.v_iso_checked(iso, check)
                };
                show(curve, |c| {
                    let c = Handle::new(c);
                    name(&c);
                    curve_summary(&*c, &[0.0, 0.5, 1.0]);
                });
            }
        }
    }
}

pub fn bspline_surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(base) = super::common::built_surface(&mut r) else {
        return;
    };
    let s: Handle<GeomBSplineSurface> = base.cast();
    let (i, j, tol) = (r.integer(), r.integer(), r.real());
    state(&s);
    getters(&s, i, j);
    queries(&s, &mut r);
    int(GeomBSplineSurface::max_degree());
    show(s.resolution(tol), |(a, b)| {
        [a, b].into_iter().for_each(real)
    });
    show(s.resolution(2. * tol), |(a, b)| {
        [a, b].into_iter().for_each(real)
    });
    dump(&*s, -1);
    state(&s.copy().unwrap().cast::<GeomBSplineSurface>());
    end();
}

fn read_reals(r: &mut Reader) -> Array1<'static, f64> {
    let (lower, n) = (r.integer(), r.integer());
    let mut a = Array1::<f64>::with_bounds(lower, lower + n - 1).unwrap();
    for k in lower..lower + n {
        a[k] = r.real();
    }
    a
}
fn read_ints(r: &mut Reader) -> Array1<'static, i32> {
    let (lower, n) = (r.integer(), r.integer());
    let mut a = Array1::<i32>::with_bounds(lower, lower + n - 1).unwrap();
    for k in lower..lower + n {
        a[k] = r.integer();
    }
    a
}
fn read_pnts(r: &mut Reader) -> Array1<'static, Point3> {
    let (lower, n) = (r.integer(), r.integer());
    let mut a = Array1::<Point3>::with_bounds(lower, lower + n - 1).unwrap();
    for k in lower..lower + n {
        a[k] = r.pnt();
    }
    a
}

// Periodicity, origin, reversal, degree and knot edits (codes 0 to 31).
fn knot_edit(s: &GeomBSplineSurface, r: &mut Reader, op: i32, u: bool) -> Result<(), Failure> {
    match op {
        0 => s.exchange_uv(),
        2 | 3 => {
            if u {
                s.set_u_periodic()
            } else {
                s.set_v_periodic()
            }
        }
        4 | 5 => {
            let i = r.integer();
            if u {
                s.set_u_origin(i)
            } else {
                s.set_v_origin(i)
            }
        }
        6 | 7 => {
            if u {
                s.set_u_not_periodic()
            } else {
                s.set_v_not_periodic()
            }
        }
        8 | 9 => {
            if u {
                s.u_reverse()
            } else {
                s.v_reverse()
            }
        }
        10 => {
            let (a, b) = (r.integer(), r.integer());
            s.increase_degree(a, b)
        }
        12 | 13 => {
            let (k, m) = (read_reals(r), read_ints(r));
            let (t, add) = (r.real(), r.flag());
            if u {
                s.insert_u_knots(&k, &m, t, add)
            } else {
                s.insert_v_knots(&k, &m, t, add)
            }
        }
        14 | 15 => {
            let (i, m, t) = (r.integer(), r.integer(), r.real());
            let removed = if u {
                s.remove_u_knot(i, m, t)
            } else {
                s.remove_v_knot(i, m, t)
            };
            removed.map(flag)
        }
        _ => multiplicity_edit(s, r, op, u),
    }
}

fn multiplicity_edit(
    s: &GeomBSplineSurface,
    r: &mut Reader,
    op: i32,
    u: bool,
) -> Result<(), Failure> {
    match op {
        16 | 17 => {
            let (i, m) = (r.integer(), r.integer());
            if u {
                s.increase_u_multiplicity(i, m)
            } else {
                s.increase_v_multiplicity(i, m)
            }
        }
        18 | 19 => {
            let (a, b, m) = (r.integer(), r.integer(), r.integer());
            if u {
                s.increase_u_multiplicity_range(a, b, m)
            } else {
                s.increase_v_multiplicity_range(a, b, m)
            }
        }
        20 | 21 => {
            let (a, b, st) = (r.integer(), r.integer(), r.integer());
            if u {
                s.increment_u_multiplicity(a, b, st)
            } else {
                s.increment_v_multiplicity(a, b, st)
            }
        }
        22 | 23 => {
            let (x, m, t, add) = (r.real(), r.integer(), r.real(), r.flag());
            if u {
                s.insert_u_knot(x, m, t, add)
            } else {
                s.insert_v_knot(x, m, t, add)
            }
        }
        _ => knot_value_edit(s, r, op, u),
    }
}

fn knot_value_edit(
    s: &GeomBSplineSurface,
    r: &mut Reader,
    op: i32,
    u: bool,
) -> Result<(), Failure> {
    match op {
        24 | 25 => {
            let p: Vec<f64> = (0..6).map(|_| r.real()).collect();
            let (a, b, c) = ((p[0], p[1]), (p[2], p[3]), (p[4], p[5]));
            if u {
                s.segment(a, b, c)
            } else {
                s.check_and_segment(a, b, c)
            }
        }
        26 | 27 => {
            let (i, x) = (r.integer(), r.real());
            if u {
                s.set_u_knot(i, x)
            } else {
                s.set_v_knot(i, x)
            }
        }
        28 | 29 => {
            let k = read_reals(r);
            if u {
                s.set_u_knots(&k)
            } else {
                s.set_v_knots(&k)
            }
        }
        30 | 31 => {
            let (i, x, m) = (r.integer(), r.real(), r.integer());
            if u {
                s.set_u_knot_with_multiplicity(i, x, m)
            } else {
                s.set_v_knot_with_multiplicity(i, x, m)
            }
        }
        _ => pole_edit(s, r, op, u),
    }
}

// Pole and weight edits, MovePoint and Transform (codes 32 to 46).
fn pole_edit(s: &GeomBSplineSurface, r: &mut Reader, op: i32, u: bool) -> Result<(), Failure> {
    match op {
        32 => {
            let (i, j, p) = (r.integer(), r.integer(), r.pnt());
            s.set_pole(i, j, p)
        }
        34 => {
            let (i, j, p, w) = (r.integer(), r.integer(), r.pnt(), r.real());
            s.set_pole_weighted(i, j, p, w)
        }
        36 | 37 => {
            let (i, p) = (r.integer(), read_pnts(r));
            if u {
                s.set_pole_row(i, &p)
            } else {
                s.set_pole_col(i, &p)
            }
        }
        38 | 39 => {
            let (i, p, w) = (r.integer(), read_pnts(r), read_reals(r));
            if u {
                s.set_pole_row_weighted(i, &p, &w)
            } else {
                s.set_pole_col_weighted(i, &p, &w)
            }
        }
        40 => {
            let (i, j, w) = (r.integer(), r.integer(), r.real());
            s.set_weight(i, j, w)
        }
        42 | 43 => {
            let (i, w) = (r.integer(), read_reals(r));
            if u {
                s.set_weight_row(i, &w)
            } else {
                s.set_weight_col(i, &w)
            }
        }
        44 => {
            let (a, b, p) = (r.real(), r.real(), r.pnt());
            let range = MoveRange {
                u_index1: r.integer(),
                u_index2: r.integer(),
                v_index1: r.integer(),
                v_index2: r.integer(),
            };
            let (f1, f2, f3, f4) = s.move_point(a, b, p, range)?;
            [f1, f2, f3, f4].into_iter().for_each(int);
            Ok(())
        }
        46 => s.transform(trsf(r)),
        _ => panic!("unknown edit {op}"),
    }
}

pub fn bspline_surface_edit_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(base) = super::common::built_surface(&mut r) else {
        return;
    };
    let s: Handle<GeomBSplineSurface> = base.cast();
    let uv = r.reals();
    let n = r.integer();
    for _ in 0..n {
        let op = r.integer();
        attempt(|| knot_edit(&s, &mut r, op, op % 2 == 0));
        state(&s);
        surface_summary(&*s, &uv);
    }
    dump(&*s, -1);
    end();
}
