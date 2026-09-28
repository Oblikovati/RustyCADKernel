//! Range edge cases absent from the legacy assertions.
use rustycad_tkernel::{ncollection::List, standard::dump::DumpStream};
use rustycad_tkmath::bnd::{IntersectStatus, Range1};
#[test]
fn native_boundary_precedence_and_append_semantics() {
    let range = Range1::new(3.0, 15.0).unwrap();
    assert_eq!(range.is_intersected(3.0, -4.0), IntersectStatus::Boundary);
    let mut pieces = List::default();
    pieces.append(Range1::default());
    range.split(3.0, &mut pieces, -4.0).unwrap();
    assert_eq!(pieces.len(), 2);
    assert_eq!(*pieces.last().unwrap(), range);
    let mut pieces = List::default();
    range.split(5.0, &mut pieces, -4.0).unwrap();
    let bounds: Vec<_> = pieces.iter().map(|part| part.get().unwrap()).collect();
    assert_eq!(bounds, [[3.0, 5.0], [5.0, 9.0], [9.0, 13.0], [13.0, 15.0]]);
}
#[test]
fn noncanonical_void_nan_and_output_preservation() {
    let mut range = Range1::new(3.0, 10.0).unwrap();
    range.trim_from(15.0);
    assert_ne!(range, Range1::default());
    let mut first = 91.0;
    let mut last = 92.0;
    assert!(!range.get_bounds(&mut first, &mut last));
    assert_eq!((first, last), (91.0, 92.0));
    range.common(Range1::new(0.0, 1.0).unwrap());
    assert_eq!(range.delta(), -5.0);
    range.shift(4.0);
    range.trim_to(0.0);
    assert_eq!(range.delta(), -5.0);
    let finite = Range1::new(1.0, 2.0).unwrap();
    range = finite;
    range.add(f64::NAN);
    assert_eq!(range, finite);
    assert!(range.contains(f64::NAN));
    range.enlarge(-2.0);
    assert!(range.is_void());
}
#[test]
fn dump_and_void_split() {
    let mut stream = DumpStream::default();
    Range1::new(1.0, 2.0)
        .unwrap()
        .dump_json(&mut stream)
        .unwrap();
    assert_eq!(
        stream.text(),
        "\"className\": \"Bnd_Range\", \"First\": 1, \"Last\": 2"
    );
    let mut list = List::default();
    Range1::default().split(1.0, &mut list, 0.0).unwrap();
    assert_eq!(*list.first().unwrap(), Range1::default());
    let mut empty = Range1::default();
    assert!(!empty.union(Range1::new(1.0, 2.0).unwrap()));
}
