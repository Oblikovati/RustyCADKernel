//! Behavior of the IntRes2d port: the native exceptions of the accessors, the
//! ordering and duplicate rule of Insert, and the segment merge of Append (the
//! numeric results are compared with OCCT by the intres2d probe).
use rustycad_tkgeomalgo::int_res2d::{
    Domain, Intersection, IntersectionPoint, IntersectionSegment, Position, Situation, Transition,
    TypeTrans,
};
use rustycad_tkmath::gp::Point2;

fn point(u1: f64, u2: f64) -> IntersectionPoint {
    let t: Transition = Transition::crossing(false, Position::Middle, TypeTrans::In);
    IntersectionPoint::new(Point2::new(u1, u2), u1, u2, t, t, false)
}

#[test]
fn domain_accessors_raise_domain_error_without_the_bound() {
    let domain = Domain::semi_infinite(Point2::new(1.0, 2.0), 3.0, 1e-7, false);
    assert!(!domain.has_first_point() && domain.has_last_point());
    let error = domain.first_parameter().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    assert_eq!(domain.last_parameter().unwrap(), 3.0);
}

#[test]
fn domain_clamps_infinite_values_and_closes_only_bounded_domains() {
    let far = Point2::new(f64::INFINITY, -1e200);
    let mut domain = Domain::bounded(far, -f64::INFINITY, 0.0, Point2::new(0.0, 0.0), 1.0, 0.0);
    assert_eq!(domain.first_parameter().unwrap(), -2e100);
    assert_eq!(domain.first_point().unwrap().x(), 2e100);
    assert_eq!(domain.first_point().unwrap().y(), -2e100);
    domain.set_equivalent_parameters(0.0, 6.0).unwrap();
    assert!(domain.is_closed());
    assert_eq!(domain.equivalent_parameters(), (0.0, 6.0));
    let mut open = Domain::new();
    let error = open.set_equivalent_parameters(0.0, 1.0).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
}

#[test]
fn transition_touch_members_raise_on_crossings() {
    let crossing = Transition::crossing(true, Position::Head, TypeTrans::Out);
    assert!(crossing.is_tangent().unwrap());
    assert!(crossing.situation().is_err() && crossing.is_opposite().is_err());
    let mut touch = Transition::touch(true, Position::End, Situation::Inside, true);
    assert_eq!(touch.situation().unwrap(), Situation::Inside);
    touch.set_undecided_value(Position::Middle);
    assert!(touch.is_tangent().is_err());
    // An undecided value keeps the stored side, as the native setter does.
    touch.set_value(false, Position::Head, TypeTrans::Touch);
    assert_eq!(touch.situation().unwrap(), Situation::Inside);
}

#[test]
fn reversed_points_swap_parameters_and_transitions() {
    let t1 = Transition::crossing(false, Position::Head, TypeTrans::In);
    let t2 = Transition::crossing(false, Position::End, TypeTrans::Out);
    let p = IntersectionPoint::new(Point2::new(0.0, 0.0), 1.0, 2.0, t1, t2, true);
    assert_eq!((p.param_on_first(), p.param_on_second()), (2.0, 1.0));
    assert_eq!(p.transition_of_first(), t2);
    let s = IntersectionSegment::half_bounded(p, true, true, true);
    assert!(!s.has_first_point() && s.has_last_point());
}

#[test]
fn insert_keeps_the_order_and_drops_duplicates() {
    let mut inter = Intersection::new();
    inter.insert(point(2.0, 0.0)).unwrap();
    inter.insert(point(1.0, 0.0)).unwrap();
    inter.insert(point(3.0, 0.0)).unwrap();
    inter.insert(point(1.0 + 1e-9, 0.0)).unwrap();
    inter.done = true;
    assert_eq!(inter.nb_points().unwrap(), 3);
    assert_eq!(inter.point(1).unwrap().param_on_first(), 1.0);
    assert_eq!(inter.point(3).unwrap().param_on_first(), 3.0);
    assert_eq!(
        inter.point(4).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn append_composite_joins_touching_segments_and_removes_inner_points() {
    let mut first = Intersection::new();
    first.append_segment(IntersectionSegment::bounded(
        point(0.0, 0.0),
        point(1.0, 1.0),
        false,
        false,
    ));
    first.done = true;
    let mut second = Intersection::new();
    second.append_segment(IntersectionSegment::bounded(
        point(1.0, 1.0),
        point(2.0, 2.0),
        false,
        false,
    ));
    second.append_point(point(1.5, 1.5));
    second.done = true;
    let mut merged = Intersection::new();
    merged.append_composite(&first, 0.0, 2.0, 0.0, 2.0).unwrap();
    merged
        .append_composite(&second, 0.0, 2.0, 0.0, 2.0)
        .unwrap();
    assert_eq!(merged.nb_segments().unwrap(), 1);
    assert_eq!(merged.nb_points().unwrap(), 0);
    let segment = merged.segment(1).unwrap();
    assert_eq!(segment.last_point().unwrap().param_on_first(), 2.0);
}

#[test]
fn clone_is_the_copy_constructor_and_assign_the_assignment() {
    let mut inter = Intersection::new();
    inter.append_point(point(1.0, 1.0));
    inter.done = true;
    inter.set_reversed_parameters(true);
    let copy = inter.clone();
    assert!(!copy.is_done() && !copy.reversed_parameters());
    let mut assigned = Intersection::new();
    assigned.assign(&inter);
    assert!(assigned.is_done() && assigned.reversed_parameters());
    assert_eq!(assigned.nb_points().unwrap(), 1);
}
