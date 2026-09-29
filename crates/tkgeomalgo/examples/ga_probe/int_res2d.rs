//! IntRes2d probe operations and the result printers shared by the 2D intersection
//! probes; the Rust twin of tools/oracle/tkgeomalgo_probe_intres2d.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, show};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_res2d::{
    Domain, Intersection, IntersectionPoint, IntersectionSegment, Position, Situation, Transition,
    TypeTrans,
};
use rustycad_tkmath::gp::Point2;

/// `ga_trans`: position, type, tangency (not for Undecided), side and orientation (Touch).
pub fn trans(t: &Transition) -> Result<(), Failure> {
    int(t.position_on_curve() as i32);
    int(t.transition_type() as i32);
    if t.transition_type() != TypeTrans::Undecided {
        flag(t.is_tangent()?);
    }
    if t.transition_type() == TypeTrans::Touch {
        int(t.situation()? as i32);
        flag(t.is_opposite()?);
    }
    Ok(())
}
pub fn ipoint(p: &IntersectionPoint) -> Result<(), Failure> {
    pnt2d(p.value());
    real(p.param_on_first());
    real(p.param_on_second());
    trans(&p.transition_of_first())?;
    trans(&p.transition_of_second())
}
pub fn iseg(s: &IntersectionSegment) -> Result<(), Failure> {
    flag(s.is_opposite());
    flag(s.has_first_point());
    if s.has_first_point() {
        ipoint(&s.first_point()?)?;
    }
    flag(s.has_last_point());
    if s.has_last_point() {
        ipoint(&s.last_point()?)?;
    }
    Ok(())
}
/// `ga_inter`: done, then the points and the segments.
pub fn inter(r: &Intersection) -> Result<(), Failure> {
    flag(r.is_done());
    if !r.is_done() {
        return Ok(());
    }
    int(r.nb_points()?);
    for i in 1..=r.nb_points()? {
        ipoint(&r.point(i)?)?;
    }
    int(r.nb_segments()?);
    for i in 1..=r.nb_segments()? {
        iseg(&r.segment(i)?)?;
    }
    Ok(())
}

fn position(code: i32) -> Position {
    [Position::Head, Position::Middle, Position::End][code as usize]
}
fn situation(code: i32) -> Situation {
    [Situation::Inside, Situation::Outside, Situation::Unknown][code as usize]
}
fn read_trans(r: &mut Reader<'_>) -> Transition {
    let (pos, kind, tangent) = (r.integer(), r.integer(), r.flag());
    let (situ, oppos) = (r.integer(), r.flag());
    match kind {
        2 => Transition::touch(tangent, position(pos), situation(situ), oppos),
        3 => Transition::undecided(position(pos)),
        _ => Transition::crossing(
            tangent,
            position(pos),
            [TypeTrans::In, TypeTrans::Out][kind as usize],
        ),
    }
}
fn read_ipoint(r: &mut Reader<'_>) -> IntersectionPoint {
    let (u1, u2) = (r.real(), r.real());
    let p: Point2 = r.pnt2d();
    let (t1, t2) = (read_trans(r), read_trans(r));
    IntersectionPoint::new(p, u1, u2, t1, t2, false)
}

/// `n point...`: each point inserted, then the resulting list.
pub fn insert_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut result = Intersection::new();
    let n = r.integer();
    let points: Vec<IntersectionPoint> = (0..n).map(|_| read_ipoint(&mut r)).collect();
    attempt(|| {
        for point in points {
            result.insert(point)?;
        }
        result.done = true;
        inter(&result)
    });
    end();
}

fn read_part(r: &mut Reader<'_>) -> Intersection {
    let mut part = Intersection::new();
    for _ in 0..r.integer() {
        part.append_point(read_ipoint(r));
    }
    for _ in 0..r.integer() {
        let oppos = r.flag();
        let (a, b) = (read_ipoint(r), read_ipoint(r));
        part.append_segment(IntersectionSegment::bounded(a, b, oppos, false));
    }
    part.done = true;
    part
}

/// `f1 l1 f2 l2 part part`: two composite parts appended in turn.
pub fn append_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (f1, l1, f2, l2) = (r.real(), r.real(), r.real(), r.real());
    let (first, second) = (read_part(&mut r), read_part(&mut r));
    let mut merged = Intersection::new();
    attempt(|| {
        merged.append_composite(&first, f1, l1, f2, l2)?;
        merged.append_composite(&second, f1, l1, f2, l2)?;
        inter(&merged)
    });
    end();
}

/// The domain of the IntRes2d and IntImpParGen probes: `kind p1 t1 p2 t2 closed z zp`
/// with the points given by `point_of`, closed separately (the caller prints the
/// exception of SetEquivalentParameters).
pub fn read_raw_domain(
    r: &mut Reader<'_>,
    point_of: impl Fn(f64) -> Point2,
) -> (Domain, bool, f64, f64) {
    let kind = r.integer();
    let (p1, t1, p2, t2) = (r.real(), r.real(), r.real(), r.real());
    let (closed, z, zp) = (r.flag(), r.real(), r.real());
    let mut d = Domain::new();
    match kind {
        1 => d.set_values(point_of(p1), p1, t1, point_of(p2), p2, t2),
        2 => d.set_semi_infinite_values(point_of(p1), p1, t1, true),
        3 => d.set_semi_infinite_values(point_of(p2), p2, t2, false),
        _ => {}
    }
    (d, closed, z, zp)
}

/// `kind p1 t1 p2 t2 closed z zp`: every domain accessor.
pub fn domain_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mut d, closed, z, zp) = read_raw_domain(&mut r, |u| Point2::new(u, -u));
    if closed {
        attempt(|| d.set_equivalent_parameters(z, zp));
    }
    flag(d.has_first_point());
    flag(d.has_last_point());
    flag(d.is_closed());
    show(d.first_parameter(), real);
    show(d.first_point(), pnt2d);
    show(d.first_tolerance(), real);
    show(d.last_parameter(), real);
    show(d.last_point(), pnt2d);
    show(d.last_tolerance(), real);
    let (e1, e2) = d.equivalent_parameters();
    real(e1);
    real(e2);
    end();
}
