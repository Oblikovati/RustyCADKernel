//! Intf probe operations over the package functions and the result records, the Rust
//! twin of tools/oracle/tkgeomalgo_probe_intf.cxx.
use super::common::{Reader, attempt, captured, end, flag, int, pnt, real, xyz};
use rustycad_tkgeomalgo::intf::{
    Interference, PIType, SectionLine, SectionPoint, TangentZone, contain, plane_equation,
};

/// `p1 p2 p3 point`: the plane equation of the triangle, then whether it contains the point.
pub fn triangle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (p1, p2, p3, point) = (r.pnt(), r.pnt(), r.pnt(), r.pnt());
    let (normal, distance) = plane_equation(p1, p2, p3);
    xyz(normal);
    real(distance);
    flag(contain(p1, p2, p3, point));
    end();
}

fn pi_type(r: &mut Reader) -> PIType {
    PIType::ALL[r.integer() as usize]
}

/// `ga_spoint`: a 3d (`0 ...`) or 2d (`1 ...`) section point.
pub fn spoint(r: &mut Reader) -> SectionPoint {
    if r.integer() == 0 {
        let p = r.pnt();
        let (d1, a1, a2, p1) = (pi_type(r), r.integer(), r.integer(), r.real());
        let (d2, a3, a4, p2) = (pi_type(r), r.integer(), r.integer(), r.real());
        return SectionPoint::with_data(p, d1, a1, a2, p1, d2, a3, a4, p2, r.real());
    }
    let p = r.pnt2d();
    let (d1, a1, p1) = (pi_type(r), r.integer(), r.real());
    let (d2, a2, p2) = (pi_type(r), r.integer(), r.real());
    SectionPoint::with_data_2d(p, d1, a1, p1, d2, a2, p2, r.real())
}

pub fn spoints(r: &mut Reader) -> Vec<SectionPoint> {
    (0..r.integer()).map(|_| spoint(r)).collect()
}

/// `ga_spoint_print`: every accessor of a section point.
pub fn spoint_print(p: &SectionPoint) {
    pnt(p.pnt());
    let four = |(d, a1, a2, par): (PIType, i32, i32, f64)| {
        int(d as i32);
        int(a1);
        int(a2);
        real(par);
    };
    let three = |(d, a, par): (PIType, i32, f64)| {
        int(d as i32);
        int(a);
        real(par);
    };
    four(p.info_first());
    three(p.info_first_address());
    four(p.info_second());
    three(p.info_second_address());
    real(p.param_on_first());
    real(p.param_on_second());
    int(p.type_on_first() as i32);
    int(p.type_on_second() as i32);
    real(p.incidence());
}

/// `a b`: both points, the comparisons both ways, then both after a.merge(b).
pub fn spoint_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mut a, mut b) = (spoint(&mut r), spoint(&mut r));
    spoint_print(&SectionPoint::new());
    spoint_print(&a);
    spoint_print(&b);
    flag(a.is_equal(&b));
    flag(a == b);
    flag(a.is_on_same_edge(&b));
    flag(b.is_on_same_edge(&a));
    a.merge(&mut b);
    spoint_print(&a);
    spoint_print(&b);
    end();
}

pub fn sline_print(line: &SectionLine) {
    int(line.number_of_points());
    for i in 1..=line.number_of_points() {
        spoint_print(line.get_point(i).expect("probe section line point"));
    }
}

fn sline_of(r: &mut Reader, points: &[SectionPoint]) -> SectionLine {
    let mut line = SectionLine::new();
    for _ in 0..r.integer() {
        line.append(points[r.integer() as usize]);
    }
    line
}

fn sline_command(r: &mut Reader, line: &mut SectionLine, p: &[SectionPoint], command: i32) {
    let point = |r: &mut Reader| p[r.integer() as usize];
    match command {
        0 => line.append(point(r)),
        1 => line.prepend(point(r)),
        2 => line.reverse(),
        3 => line.close(),
        4 | 5 => {
            let mut other = sline_of(r, p);
            if command == 4 {
                line.append_line(&mut other);
            } else {
                line.prepend_line(&mut other);
            }
            int(other.number_of_points());
        }
        6 => {
            let k = point(r);
            attempt(|| line.is_end(&k).map(int));
        }
        7 => flag(line.contains(&point(r))),
        8 => attempt(|| line.is_closed().map(flag)),
        9 => {
            let i = r.integer();
            attempt(|| line.get_point(i).map(spoint_print));
        }
        10 => {
            let indent = r.integer();
            captured(|text| line.dump(indent, text));
        }
        11 => {
            let copy = line.clone();
            flag(copy.is_equal(line));
            flag(copy == *line);
            sline_print(&copy);
        }
        _ => {
            let mut other = sline_of(r, p);
            flag(other == *line);
            other.assign(line);
            flag(other.is_equal(line));
        }
    }
}

pub fn sline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let points = spoints(&mut r);
    let mut line = SectionLine::new();
    for _ in 0..r.integer() {
        let command = r.integer();
        sline_command(&mut r, &mut line, &points, command);
        int(line.number_of_points());
    }
    sline_print(&line);
    end();
}

pub fn tzone_print(zone: &TangentZone) {
    int(zone.number_of_points());
    let (a, b) = zone.param_on_first();
    real(a);
    real(b);
    let (a, b) = zone.param_on_second();
    real(a);
    real(b);
    for (s0, a, s1, b) in [zone.info_first(), zone.info_second()] {
        int(s0);
        real(a);
        int(s1);
        real(b);
    }
    for i in 1..=zone.number_of_points() {
        spoint_print(zone.get_point(i).expect("probe tangent zone point"));
    }
}

pub fn tzone_of(r: &mut Reader, points: &[SectionPoint]) -> TangentZone {
    let mut zone = TangentZone::new();
    for _ in 0..r.integer() {
        zone.append(points[r.integer() as usize]);
    }
    zone
}

fn tzone_command(r: &mut Reader, zone: &mut TangentZone, p: &[SectionPoint], command: i32) {
    let point = |r: &mut Reader| p[r.integer() as usize];
    match command {
        0 => zone.append(point(r)),
        1 => zone.polygon_insert(point(r)),
        2 | 3 => {
            let (i, k) = (r.integer(), point(r));
            if command == 2 {
                attempt(|| zone.insert_before(i, k));
            } else {
                attempt(|| zone.insert_after(i, k));
            }
        }
        4 => flag(zone.insert(&point(r))),
        5 => {
            let other = tzone_of(r, p);
            zone.append_zone(&other);
        }
        6 => flag(zone.contains(&point(r))),
        7 => flag(zone.range_contains(&point(r))),
        8 => {
            let i = r.integer();
            attempt(|| zone.get_point(i).map(spoint_print));
        }
        9 => {
            let other = tzone_of(r, p);
            flag(zone.has_common_range(&other));
            flag(other.has_common_range(zone));
        }
        _ => {
            let copy = zone.clone();
            flag(copy.is_equal(zone));
            flag(copy == *zone);
        }
    }
}

pub fn tzone_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let points = spoints(&mut r);
    let mut zone = TangentZone::new();
    for _ in 0..r.integer() {
        let command = r.integer();
        tzone_command(&mut r, &mut zone, &points, command);
        int(zone.number_of_points());
    }
    tzone_print(&zone);
    end();
}

/// `ga_interference_print`: the tolerance, the points, lines and zones, then the range
/// errors of the three accessors.
pub fn interference_print(interference: &Interference) {
    real(interference.get_tolerance());
    int(interference.nb_section_points());
    for i in 1..=interference.nb_section_points() {
        spoint_print(interference.pnt_value(i).expect("probe section point"));
    }
    int(interference.nb_section_lines());
    for i in 1..=interference.nb_section_lines() {
        sline_print(interference.line_value(i).expect("probe section line"));
    }
    int(interference.nb_tangent_zones());
    for i in 1..=interference.nb_tangent_zones() {
        tzone_print(interference.zone_value(i).expect("probe tangent zone"));
    }
    attempt(|| {
        interference
            .pnt_value(interference.nb_section_points() + 1)
            .map(drop)
    });
    attempt(|| interference.line_value(0).map(drop));
    attempt(|| {
        interference
            .zone_value(interference.nb_tangent_zones() + 1)
            .map(drop)
    });
}

fn interference_command(
    r: &mut Reader,
    interference: &mut Interference,
    p: &[SectionPoint],
    command: i32,
) {
    let point = |r: &mut Reader| p[r.integer() as usize];
    match command {
        0 => {
            let zone = tzone_of(r, p);
            interference.tangent_zones_mut().append(zone);
        }
        1 => {
            let zone = tzone_of(r, p);
            attempt(|| interference.insert_zone(&zone).map(flag));
        }
        2 => {
            let (a, b) = (point(r), point(r));
            attempt(|| interference.insert_points(&a, &b));
        }
        3 => flag(interference.contains(&point(r))),
        4 => captured(|text| interference.dump(text)),
        5 => {
            let k = point(r);
            interference.section_points_mut().append(k);
        }
        6 => interference.self_interference(r.flag()),
        _ => interference.set_tolerance(r.real()),
    }
}

pub fn interference_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let points = spoints(&mut r);
    let mut interference = Interference::new(false);
    for _ in 0..r.integer() {
        let command = r.integer();
        interference_command(&mut r, &mut interference, &points, command);
        int(interference.nb_tangent_zones());
    }
    interference_print(&interference);
    end();
}
