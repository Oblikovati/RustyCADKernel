//! Behavior tests of ShapeExtend_Explorer.
mod support;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::ncollection::{HSequence, List};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::shape_extend::{ShapeDispatch, ShapeExtendExplorer};
use support::{shapes, square_edges, wire};

fn compound(items: &[Shape]) -> Shape {
    let builder = BRepBuilder::default();
    let mut c = Compound::default();
    builder.make_compound(&mut c);
    for item in items {
        builder.add(&mut c, item).unwrap();
    }
    c.into_shape()
}

#[test]
fn shape_type_merges_edges_with_wires_and_reports_mixes() {
    let ex = ShapeExtendExplorer::new();
    let edges = shapes(&square_edges());
    let w = wire(&edges[..2]).into_shape();
    assert_eq!(
        ex.shape_type(&compound(&[edges[0].clone(), w.clone()]), true)
            .unwrap(),
        ShapeEnum::Wire
    );
    assert_eq!(
        ex.shape_type(&compound(&[w.clone(), edges[1].clone()]), true)
            .unwrap(),
        ShapeEnum::Wire
    );
    assert_eq!(
        ex.shape_type(&compound(&[]), true).unwrap(),
        ShapeEnum::Shape
    );
    assert_eq!(
        ex.shape_type(&compound(std::slice::from_ref(&w)), false)
            .unwrap(),
        ShapeEnum::Compound
    );
    let vertex = support::vertex(2.0, 2.0).into_shape();
    assert_eq!(
        ex.shape_type(&compound(&[w, vertex]), true).unwrap(),
        ShapeEnum::Compound
    );
}

#[test]
fn sorted_compound_returns_one_item_as_such_and_none_as_null() {
    let ex = ShapeExtendExplorer::new();
    let edges = shapes(&square_edges());
    let w = wire(&edges).into_shape();
    let sorted = ex
        .sorted_compound(&w, ShapeEnum::Edge, true, false)
        .unwrap();
    assert_eq!(sorted.shape_type().unwrap(), ShapeEnum::Compound);
    assert_eq!(sorted.nb_children(), 4);
    let single = ex
        .sorted_compound(
            &compound(&[edges[0].clone()]),
            ShapeEnum::Edge,
            false,
            false,
        )
        .unwrap();
    assert!(single.is_same(&edges[0]));
    assert!(
        ex.sorted_compound(&w, ShapeEnum::Face, true, false)
            .unwrap()
            .is_null()
    );
    let as_wire = ex
        .sorted_compound(&edges[0], ShapeEnum::Wire, false, false)
        .unwrap();
    assert_eq!(as_wire.shape_type().unwrap(), ShapeEnum::Wire);
}

#[test]
fn sequences_lists_and_compounds_convert_into_each_other() {
    let ex = ShapeExtendExplorer::new();
    let edges = shapes(&square_edges());
    let inner = compound(&edges[..2]);
    let outer = compound(&[inner.clone(), edges[2].clone()]);
    let flat = ex.seq_from_compound(&outer, true).unwrap();
    assert_eq!(flat.sequence(|s| s.length()), 3);
    assert_eq!(
        ex.seq_from_compound(&outer, false)
            .unwrap()
            .sequence(|s| s.length()),
        2
    );
    let mut list = List::default();
    list.append(edges[3].clone());
    ex.list_from_seq(&flat, &mut list, false);
    assert_eq!(list.extent(), 4);
    let back = ex.seq_from_list(&list);
    assert_eq!(ex.compound_from_seq(&back).unwrap().nb_children(), 4);
    assert!(ex.compound_from_seq(&Handle::default()).is_err());
}

#[test]
fn dispatch_list_sorts_by_type_and_creates_missing_outputs() {
    let ex = ShapeExtendExplorer::new();
    let edges = shapes(&square_edges());
    let list = Handle::new(HSequence::default());
    list.append(edges[0].clone());
    list.append(wire(&edges[1..2]).into_shape());
    list.append(Shape::default());
    let mut out = ShapeDispatch::default();
    ex.dispatch_list(&list, &mut out).unwrap();
    assert_eq!(out.edges.sequence(|s| s.length()), 1);
    assert_eq!(out.wires.sequence(|s| s.length()), 1);
    assert_eq!(out.faces.sequence(|s| s.length()), 0);
}
