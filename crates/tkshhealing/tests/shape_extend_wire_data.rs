//! Behavior tests of ShapeExtend_WireData, including the native quirks the port keeps.
mod support;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkshhealing::shape_extend::ShapeExtendWireData;
use support::{shapes, square_edges, wire};

#[test]
fn init_from_a_wire_keeps_the_order_and_closes_the_wire() {
    let edges = square_edges();
    let data = ShapeExtendWireData::from_wire(&wire(&shapes(&edges)), true, true).unwrap();
    assert_eq!(data.nb_edges(), 4);
    for (i, e) in edges.iter().enumerate() {
        assert!(data.edge(i as i32 + 1).unwrap().is_same(e));
    }
    assert_eq!(data.edge(-1).unwrap().orientation(), Orientation::Reversed);
    assert!(data.wire().unwrap().closed().unwrap());
    assert_eq!(
        data.edge(5).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn a_reversed_wire_is_read_backwards_and_a_broken_chain_is_reported() {
    // The chain is checked in iteration order while the edges of a REVERSED wire are
    // prepended, so a reversed closed wire is reported as not chained, as natively.
    let edges = square_edges();
    let reversed = wire(&shapes(&edges)).oriented(Orientation::Reversed);
    let data = ShapeExtendWireData::new();
    let wire_reversed = rustycad_tkbrep::topods::wire(&reversed).unwrap();
    assert!(!data.init_wire(&wire_reversed, true, true).unwrap());
    assert_eq!(data.nb_edges(), 4);
    assert!(data.edge(1).unwrap().is_same(&edges[3]));
    let broken = wire(&shapes(&[edges[0].clone(), edges[2].clone()]));
    assert!(!data.init_wire(&broken, true, true).unwrap());
    assert_eq!(data.nb_edges(), 2);
}

#[test]
fn internal_edges_go_apart_in_the_manifold_mode_only() {
    let edges = square_edges();
    let internal = Edge::unchecked(edges[1].oriented(Orientation::Internal));
    let data = ShapeExtendWireData::new();
    data.add_edge(&edges[0], 0).unwrap();
    data.add_edge(&internal, 0).unwrap();
    assert_eq!((data.nb_edges(), data.nb_non_manifold_edges()), (1, 1));
    data.set_manifold_mode(false);
    data.add_edge(&internal, 0).unwrap();
    assert_eq!(data.nb_edges(), 2);
}

#[test]
fn a_null_edge_joins_the_non_manifold_edges_as_natively() {
    // TopoDS_Shape() is EXTERNAL: Add tests the orientation before the null check.
    let data = ShapeExtendWireData::new();
    data.add_edge(&Edge::default(), 0).unwrap();
    assert_eq!(data.nb_non_manifold_edges(), 1);
    assert!(data.nonmanifold_edge(1).unwrap().is_null());
    data.set_manifold_mode(false);
    data.add_edge(&Edge::default(), 0).unwrap();
    assert_eq!(data.nb_edges(), 0);
}

#[test]
fn set_last_rotates_and_reverse_reverses_order_and_edges() {
    let edges = square_edges();
    let data = ShapeExtendWireData::from_wire(&wire(&shapes(&edges)), true, true).unwrap();
    data.set_last(2).unwrap();
    assert!(data.edge(1).unwrap().is_same(&edges[2]));
    assert!(data.edge(4).unwrap().is_same(&edges[1]));
    data.reverse().unwrap();
    assert!(data.edge(1).unwrap().is_same(&edges[1]));
    assert_eq!(data.edge(1).unwrap().orientation(), Orientation::Reversed);
    data.remove(0).unwrap();
    assert_eq!(data.nb_edges(), 3);
    assert_eq!(
        data.remove(7).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn seams_are_an_edge_forward_and_reversed() {
    let edges = square_edges();
    let data = ShapeExtendWireData::new();
    data.add_edge(&edges[0], 0).unwrap();
    data.add_oriented_edge(&edges[1], 0).unwrap();
    data.add_oriented_edge(&edges[0], 1).unwrap();
    assert!(data.is_seam(1) && data.is_seam(3) && !data.is_seam(2));
    let reversed = Edge::unchecked(edges[0].reversed());
    assert_eq!(data.index(&reversed).unwrap(), 3);
    assert_eq!(data.index(&edges[0]).unwrap(), 1);
}

#[test]
fn init_from_itself_gives_an_empty_list_and_clear_keeps_the_old_handle() {
    let edges = square_edges();
    let data = ShapeExtendWireData::from_wire(&wire(&shapes(&edges)), true, true).unwrap();
    data.init(&data).unwrap();
    assert_eq!(data.nb_edges(), 0);
    let internal = Edge::unchecked(edges[1].oriented(Orientation::Internal));
    data.add_edge(&internal, 0).unwrap();
    let kept = data.nonmanifold_edges();
    data.clear();
    assert_eq!(kept.sequence(|s| s.length()), 1);
    assert_eq!(data.nb_non_manifold_edges(), 0);
}

#[test]
fn add_wire_data_inserts_from_a_rank() {
    let edges = square_edges();
    let other = ShapeExtendWireData::from_wire(&wire(&shapes(&edges[2..])), true, true).unwrap();
    let data = ShapeExtendWireData::from_wire(&wire(&shapes(&edges[..2])), true, true).unwrap();
    data.add_wire_data(&other, 2).unwrap();
    let order: Vec<bool> = [0, 2, 3, 1]
        .iter()
        .enumerate()
        .map(|(i, &k)| data.edge(i as i32 + 1).unwrap().is_same(&edges[k]))
        .collect();
    assert_eq!(order, [true; 4]);
}
