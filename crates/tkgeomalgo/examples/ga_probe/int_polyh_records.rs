//! IntPolyh probe operations over the records (points, start points, section lines,
//! arrays, couples, edges) and the tools, the Rust twin of the second half of
//! tools/oracle/tkgeomalgo_probe_intpolyh.cxx.
use super::common::{Reader, attempt, captured, end, flag, int, pnt, real, show, xyz};
use super::ics_geometry::surface;
use super::int_polyh::{edge, point, read_point, read_sp, sp};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_polyh::{
    IntPolyhArrayOfPointNormal, IntPolyhArrayOfPoints, IntPolyhCouple, IntPolyhEdge,
    IntPolyhSectionLine, IntPolyhStartPoint, IntPolyhTools, IntPolyhTriangle,
};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};

/// `ga_ph_printed`: the text a dump wrote.
fn printed(dump: impl FnOnce(&mut String) -> fmt::Result) {
    captured(|out| dump(out).map_err(|_| Failure::new("fmt")));
}

/// `ga_ph_point_case`.
pub fn point_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (read_point(&mut r), read_point(&mut r));
    let rr = r.real();
    let s = surface(&mut r);
    let i = r.integer();
    for p in [
        IntPolyhPoint::default(),
        a + b,
        a - b,
        a / rr,
        a * rr,
        a.add(&b),
        a.sub(&b),
        a.divide(rr),
        a.multiplication(rr),
    ] {
        point(&p);
    }
    real(a.square_modulus());
    real(a.square_distance(&b));
    real(a.dot(&b));
    let mut c = a;
    c.cross(&a, &b);
    point(&c);
    c.set(1.0, 2.0, 3.0, 4.0, 5.0, i);
    point(&c);
    c.set(6.0, 7.0, 8.0, 9.0, 10.0, 1);
    point(&c);
    c.set_x(rr);
    c.set_y(-rr);
    c.set_z(2.0 * rr);
    c.set_u(3.0 * rr);
    c.set_v(4.0 * rr);
    c.set_part_of_common(i);
    c.set_degenerated(true);
    point(&c);
    let mut m = c;
    show(m.middle(&*s, &a, &b).map(|()| m), |m| point(&m));
    printed(|out| a.dump(out));
    printed(|out| b.dump_index(i, out));
    end();
}
use rustycad_tkgeomalgo::int_polyh::IntPolyhPoint;

/// `ga_ph_startpoint_case`.
pub fn startpoint_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (read_sp(&mut r), read_sp(&mut r));
    let (p1, p2, p3) = (r.integer(), r.integer(), r.integer());
    let t = IntPolyhTriangle::new(p1, p2, p3);
    sp(&IntPolyhStartPoint::default());
    sp(&a);
    int(a.check_same_sp(&b));
    int(b.check_same_sp(&a));
    let (surf_id, points) = a.get_edge_points(&t);
    int(surf_id);
    points.unwrap_or([-9, -9, -9]).into_iter().for_each(int);
    let mut c = b;
    c.set_xyz(1.0, 2.0, 3.0);
    c.set_uv1(4.0, 5.0);
    c.set_uv2(6.0, 7.0);
    c.set_edge1(8);
    c.set_lambda1(0.25);
    c.set_edge2(9);
    c.set_lambda2(0.75);
    c.set_couple_value(10, 11);
    c.set_angle(0.5);
    c.set_chain_list(12);
    sp(&c);
    printed(|out| a.dump(out));
    printed(|out| b.dump_index(7, out));
    end();
}

fn sectionline_op(line: &mut IntPolyhSectionLine, r: &mut Reader, sps: &[IntPolyhStartPoint]) {
    match r.integer() {
        0 => line.init(r.integer()),
        1 => line.increment_nb_start_points(),
        2 => line.prepend(&sps[r.integer() as usize]),
        3 => {
            let (idx, i) = (r.integer(), r.integer());
            attempt(|| {
                *line.change_value(idx)? = sps[i as usize];
                Ok(())
            });
        }
        4 => {
            let c = line.clone();
            let mut e = IntPolyhSectionLine::new();
            e.copy_from(&c);
            line.copy_from(&e);
        }
        5 => {
            let idx = r.integer();
            show(line.value(idx), sp);
        }
        6 => printed(|out| line.dump(out)),
        _ => *line = IntPolyhSectionLine::with_size(r.integer()),
    }
}

/// `ga_ph_sectionline_case`.
pub fn sectionline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let sps: Vec<IntPolyhStartPoint> = (0..r.integer()).map(|_| read_sp(&mut r)).collect();
    let mut line = IntPolyhSectionLine::new();
    for _ in 0..r.integer() {
        sectionline_op(&mut line, &mut r, &sps);
        int(line.get_n());
        int(line.nb_start_points());
    }
    end();
}

fn array_op(a: &mut IntPolyhArrayOfPoints, r: &mut Reader) {
    match r.integer() {
        0 => a.init(r.integer()),
        1 => a.increment_nb_items(),
        2 => a.set_nb_items(r.integer()),
        3 => {
            let i = r.integer();
            a[i] = read_point(r);
        }
        4 => {
            let mut b = IntPolyhArrayOfPoints::with_size(3, 256);
            b.copy_from(a);
            let c = a.clone();
            a.copy_from(&b);
            int(c.get_n());
            b.increment_nb_items();
            int(b.get_n());
        }
        _ => printed(|out| a.dump(out)),
    }
}

/// `ga_ph_array_case`.
pub fn array_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (n, inc) = (r.integer(), r.integer());
    let mut a = IntPolyhArrayOfPoints::with_size(n, inc);
    for _ in 0..r.integer() {
        array_op(&mut a, &mut r);
        int(a.nb_items());
        int(a.get_n());
    }
    (0..a.nb_items()).for_each(|i| point(a.value(i).expect("probe point")));
    end();
}

/// `ga_ph_tools_case`.
pub fn tools_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = surface(&mut r);
    let (nsu, nsv) = (r.integer(), r.integer());
    show(IntPolyhTools::is_enlarge_possible(&s), |(u, v)| {
        flag(u);
        flag(v);
    });
    for e in 0..2 {
        let (mut up, mut vp) = (Array1::default(), Array1::default());
        attempt(|| {
            IntPolyhTools::make_sampling(&s, (nsu, nsv), e != 0, &mut up, &mut vp)?;
            up.iter().chain(vp.iter()).copied().for_each(real);
            Ok(())
        });
        show(IntPolyhTools::compute_deflection(&s, &up, &vp), real);
        let mut pn = IntPolyhArrayOfPointNormal::default();
        attempt(|| {
            IntPolyhTools::fill_array_of_point_normal(&s, &up, &vp, &mut pn)?;
            int(pn.nb_items());
            for i in 0..pn.nb_items() {
                pnt(pn[i].point);
                xyz(*pn[i].normal.xyz());
            }
            Ok(())
        });
    }
    end();
}

fn hash_of(c: &IntPolyhCouple) -> u64 {
    let mut h = DefaultHasher::new();
    c.hash(&mut h);
    h.finish()
}

/// `ga_ph_couple_case`.
pub fn couple_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (r.integer(), r.integer());
    let angle = r.real();
    let i = r.integer();
    let (d, mut c, e, f) = (
        IntPolyhCouple::default(),
        IntPolyhCouple::new(a, b, angle),
        IntPolyhCouple::new(b, a, -2.0),
        IntPolyhCouple::new(a, a + 1, -2.0),
    );
    int(d.first_value());
    int(d.second_value());
    flag(d.is_analyzed());
    real(d.angle());
    flag(c.is_equal(&e));
    flag(c == f);
    flag(hash_of(&c) == hash_of(&e));
    c.set_analyzed(true);
    c.set_angle(-angle);
    c.set_couple_value(b, a);
    int(c.first_value());
    int(c.second_value());
    flag(c.is_analyzed());
    real(c.angle());
    printed(|out| c.dump(i, out));
    let (de, mut ed) = (IntPolyhEdge::default(), IntPolyhEdge::new(a, b, i, -i));
    edge(&de);
    edge(&ed);
    ed.set_first_point(i);
    ed.set_second_point(a);
    ed.set_first_triangle(b);
    ed.set_second_triangle(-1);
    edge(&ed);
    printed(|out| ed.dump(i, out));
    end();
}
