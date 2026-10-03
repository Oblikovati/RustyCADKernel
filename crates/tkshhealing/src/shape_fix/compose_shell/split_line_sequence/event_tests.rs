use super::*;
use rustycad_tkbrep::{brep::BRepBuilder, topods::Vertex};
use rustycad_tkmath::gp::Point3;

fn event(parameter: f64, code: i32) -> LineEvent {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(parameter, 0., 0.), 1e-7)
        .unwrap();
    LineEvent {
        parameter,
        code,
        vertex: vertex.into_shape(),
    }
}

#[test]
fn equal_parameters_keep_input_order_and_nan_stays_unordered() {
    let mut events = vec![event(2., 1), event(1., 2), event(1., 3)];
    sort_events(&mut events);
    assert_eq!(
        events.iter().map(|item| item.code).collect::<Vec<_>>(),
        [2, 3, 1]
    );
    let mut unordered = vec![event(2., 1), event(f64::NAN, 2), event(1., 3)];
    sort_events(&mut unordered);
    assert_eq!(
        unordered.iter().map(|item| item.code).collect::<Vec<_>>(),
        [1, 2, 3]
    );
}

#[test]
fn adjacent_tangency_ends_merge_by_side_without_rechecking_removed_slot() {
    let mut same_side = vec![event(0., ITP_ENDSEG | 1), event(0., ITP_BEGSEG | 1)];
    sort_and_merge(&mut same_side);
    assert_eq!(same_side.len(), 1);
    assert_eq!(same_side[0].code, ITP_TANG | 1);
    let mut crossing = vec![event(0., ITP_ENDSEG | 1), event(0., ITP_BEGSEG | 2)];
    sort_and_merge(&mut crossing);
    assert_eq!(crossing[0].code, ITP_INTER | IOR_BOTH);
}

#[test]
fn vertex_identity_merges_far_parameters_but_distinct_vertices_do_not() {
    let mut first = event(0., ITP_ENDSEG | 1);
    let mut second = event(3., ITP_BEGSEG | 2);
    assert!(!tangency_pair(&first, &second));
    second.vertex = first.vertex.clone();
    assert!(tangency_pair(&first, &second));
    first.code = ITP_INTER;
    assert!(!tangency_pair(&first, &second));
}

#[test]
fn parity_is_sampled_before_current_event_and_half_side_is_retained() {
    let mut parity = LineParity::default();
    assert!(!parity.advance(ITP_INTER));
    assert!(parity.advance(ITP_BEGSEG | 1));
    assert!(!parity.advance(ITP_ENDSEG | 2));
    assert!(!parity.is_odd());
    assert!(!parity.advance(ITP_BEGSEG | 2));
    assert!(!parity.advance(ITP_ENDSEG | 2));
    assert!(!parity.is_odd());
    assert!(!parity.advance(ITP_INTER));
    assert!(parity.is_odd());
    assert!(parity.advance(ITP_TANG));
}
