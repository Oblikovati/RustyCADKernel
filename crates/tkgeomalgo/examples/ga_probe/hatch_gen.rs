//! HatchGen probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_hatchgen.cxx.
//! The point printers serve the Geom2dHatch probe too.
use super::common::{Reader, attempt, captured, end, flag, int, real, show};
use super::int_res2d::read_ipoint;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::hatch_gen::{
    Domain, IntersectionPoint, IntersectionPointDump, IntersectionType, PointOnElement,
    PointOnHatching,
};

pub fn base(p: &IntersectionPoint) {
    int(p.index());
    real(p.parameter());
    int(p.position() as i32);
    int(p.state_before() as i32);
    int(p.state_after() as i32);
    flag(p.segment_beginning());
    flag(p.segment_end());
}
pub fn pe(p: &PointOnElement) {
    base(p);
    int(p.intersection_type() as i32);
}
/// A point on hatching and every point on element it holds.
pub fn ph(p: &PointOnHatching) {
    base(p);
    int(p.nb_points());
    for i in 1..=p.nb_points() {
        pe(p.point(i).expect("in range"));
    }
}

fn set(p: &mut IntersectionPoint, field: i32, v: f64) {
    match field {
        1 => p.set_index(v as i32),
        2 => p.set_parameter(v),
        3 => p.set_position(Orientation::ALL[v as usize]),
        4 => p.set_state_before(State::ALL[v as usize]),
        5 => p.set_state_after(State::ALL[v as usize]),
        6 => p.set_segment_beginning(v != 0.0),
        7 => p.set_segment_end(v != 0.0),
        _ => {}
    }
}
pub fn dump(p: &dyn IntersectionPointDump, index: i32) {
    captured(|text| {
        p.dump(index, text).expect("text");
        Ok(())
    });
}
fn kind(code: i32) -> IntersectionType {
    [
        IntersectionType::True,
        IntersectionType::Touch,
        IntersectionType::Tangent,
        IntersectionType::Undetermined,
    ][code as usize]
}

/// `point index confusion field value type`: the point on element and its comparisons.
pub fn element_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let ip = read_ipoint(&mut r);
    let (index, conf) = (r.integer(), r.real());
    let (field, v, code) = (r.integer(), r.real(), r.integer());
    attempt(|| {
        let mut p = PointOnElement::from_intersection_point(&ip)?;
        pe(&p);
        p.set_index(index);
        let mut q = p;
        set(&mut q, field, v);
        if code >= 0 {
            q.set_intersection_type(kind(code));
        }
        pe(&q);
        flag(p.is_identical(&q, conf));
        flag(p.is_different(&q, conf));
        flag(q.is_identical(&p, conf));
        flag(q.is_different(&p, conf));
        dump(&p, index);
        dump(&q, 0);
        Ok(())
    });
    let d = PointOnElement::new();
    pe(&d);
    dump(&d, -1);
    end();
}

fn read_elements(r: &mut Reader<'_>) -> (i32, Vec<PointOnElement>) {
    let n = r.integer();
    let mut elements = Vec::new();
    for _ in 0..n {
        let e = read_ipoint(r);
        let (index, field, v) = (r.integer(), r.integer(), r.real());
        attempt(|| {
            let mut point = PointOnElement::from_intersection_point(&e)?;
            point.set_index(index);
            set(&mut point, field, v);
            elements.push(point);
            Ok(())
        });
    }
    (n, elements)
}

/// `point n (point index field value)*n confusion other rem`: a point on hatching.
pub fn hatching_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let ip = read_ipoint(&mut r);
    let (n, elements) = read_elements(&mut r);
    let conf = r.real();
    let other = read_ipoint(&mut r);
    let rem = r.integer();
    let mut p = PointOnHatching::from_intersection_point(&ip);
    for element in &elements {
        p.add_point(element, conf);
    }
    ph(&p);
    for i in 0..=p.nb_points() + 1 {
        show(p.point(i), pe);
    }
    let o = PointOnHatching::from_intersection_point(&other);
    for (a, b) in [(&p, &o), (&o, &p)] {
        flag(a.is_lower(b, conf));
        flag(a.is_equal(b, conf));
        flag(a.is_greater(b, conf));
    }
    dump(&p, n);
    attempt(|| p.rem_point(rem));
    ph(&p);
    p.clr_points();
    ph(&p);
    dump(&p, 0);
    let d = PointOnHatching::new();
    ph(&d);
    dump(&d, 0);
    end();
}

pub fn domain_out(d: &Domain) {
    flag(d.has_first_point());
    show(d.first_point(), ph);
    flag(d.has_second_point());
    show(d.second_point(), ph);
}

/// `point1 point2 n op*n index`: the domain constructors and setters, then the dump.
pub fn domain_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut p1 = PointOnHatching::from_intersection_point(&read_ipoint(&mut r));
    let mut p2 = PointOnHatching::from_intersection_point(&read_ipoint(&mut r));
    p1.set_index(1);
    p2.set_index(2);
    let mut element = PointOnElement::new();
    element.set_index(7);
    p2.add_point(&element, 0.0);
    let mut d = Domain::new();
    domain_out(&d);
    domain_out(&Domain::bounded(&p1, &p2));
    domain_out(&Domain::semi_infinite(&p1, true));
    domain_out(&Domain::semi_infinite(&p2, false));
    for _ in 0..r.integer() {
        match r.integer() {
            0 => d.set_points(&p1, &p2),
            1 => d.set_infinite_points(),
            2 => d.set_first_point(&p1),
            3 => d.set_first_infinite(),
            4 => d.set_second_point(&p2),
            _ => d.set_second_infinite(),
        }
        domain_out(&d);
    }
    let index = r.integer();
    captured(|text| {
        d.dump(index, text).expect("text");
        Ok(())
    });
    end();
}
