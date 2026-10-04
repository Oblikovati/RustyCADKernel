use super::RejectionCache;
use rustycad_tkgeomalgo::int_res2d::{IntersectionPoint, Position, Transition, TypeTrans};
use rustycad_tkmath::gp::Point2;
use std::fmt::Write;

#[test]
fn native_private_cache_snapshot() {
    let mut cache = RejectionCache::new(65);
    let mut output = rejection_snapshot(&mut cache);
    output.push_str(&ordering_snapshot(&mut cache));
    output.push_str(&transition_snapshot(&mut cache));
    assert_eq!(
        output,
        include_str!("../../../../../tools/oracle/tkhlr_rejection_cache.expected")
    );
}

fn rejection_snapshot(cache: &mut RejectionCache) -> String {
    let mut output = String::new();
    for second in [1, 31, 32, 33, 64, 65] {
        cache.set_no_intersection(2, second);
        let forward = u8::from(cache.no_intersection(2, second));
        let reverse = u8::from(cache.no_intersection(second, 2));
        writeln!(output, "reject {second} {forward} {reverse}").unwrap();
    }
    let untouched = u8::from(cache.no_intersection(1, 65));
    writeln!(output, "untouched {untouched}").unwrap();
    output
}

fn ordering_snapshot(cache: &mut RejectionCache) -> String {
    for (edge, parameter) in [(4, 4.5), (2, 2.5), (3, 3.5), (2, 22.5)] {
        cache.set(1, edge, parameter);
    }
    let (two, three, four) = (cache.get(1, 2), cache.get(1, 3), cache.get(1, 4));
    let missing = u8::from(cache.get(1, 5) == f64::MAX);
    let mut output = format!("ordered {two} {three} {four} {missing}\n");
    cache.set(5, 1, 9.5);
    let (first, second) = cache.single_intersection(1, 5);
    let (first, second) = (u8::from(first == f64::MAX), u8::from(second == f64::MAX));
    writeln!(output, "absent {first} {second}").unwrap();
    output
}

fn transition_snapshot(cache: &mut RejectionCache) -> String {
    let crossing = Transition::crossing(false, Position::Middle, TypeTrans::In);
    let undecided = Transition::undecided(Position::Middle);
    cache.set_intersection(3, 4, &point(0.25, 0.75, crossing, undecided));
    let (first, second) = cache.single_intersection(3, 4);
    let mut output = format!("crossing {first} {second}\n");
    cache.set_intersection(3, 4, &point(99.0, 98.0, undecided, crossing));
    let (first, second) = cache.single_intersection(4, 3);
    writeln!(output, "ignored {first} {second}").unwrap();
    output
}

fn point(first: f64, second: f64, t1: Transition, t2: Transition) -> IntersectionPoint {
    IntersectionPoint::new(Point2::new(0.0, 0.0), first, second, t1, t2, false)
}

#[test]
fn empty_storage_and_endpoint_exclusion() {
    let empty = RejectionCache::new(0);
    assert!(empty.rejected_pairs.is_empty());
    let mut cache = RejectionCache::new(2);
    let middle = Transition::crossing(false, Position::Middle, TypeTrans::Out);
    let head = Transition::crossing(false, Position::Head, TypeTrans::In);
    cache.set_intersection(1, 2, &point(0.1, 0.2, middle, head));
    assert_eq!(cache.single_intersection(1, 2), (f64::MAX, f64::MAX));
    cache.set_intersection(1, 2, &point(0.1, 0.2, head, middle));
    assert_eq!(cache.single_intersection(1, 2), (f64::MAX, f64::MAX));
    cache.set_intersection(1, 2, &point(0.1, 0.2, middle, middle));
    assert_eq!(cache.single_intersection(1, 2), (0.1, 0.2));
}
