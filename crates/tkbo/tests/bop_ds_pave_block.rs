//! Regression cases for BOPDS_PaveBlock, a pave-splitting record used by PaveFiller.
use rustycad_tkbo::bop_ds::{Pave, PaveBlock};
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};

#[test]
fn default_edge_and_endpoint_queries_match_native_sentinels() {
    let mut block = PaveBlock::default();
    let mut edge = 99;
    assert_eq!((block.edge(), block.original_edge()), (-1, -1));
    assert!(!block.has_edge());
    assert!(!block.has_edge_into(&mut edge));
    assert_eq!(edge, -1);
    assert_eq!(block.range(), (99.0, 99.0));
    assert_eq!(block.indices(), (-1, -1));
    assert!(!block.is_to_update());
    assert!(!block.has_shrunk_data());
    assert!(!block.is_splittable());
    block.set_edge(4);
    block.set_original_edge(3);
    assert!(block.has_edge_into(&mut edge));
    assert_eq!(edge, 4);
    assert!(block.is_split_edge());
}

#[test]
fn endpoint_bounds_ignore_edge_ids_and_accept_reversed_order() {
    let mut first = PaveBlock::default();
    first.set_pave1(Pave::new(5, 0.2));
    first.set_pave2(Pave::new(8, 0.8));
    let mut reversed = PaveBlock::default();
    reversed.set_pave1(Pave::new(8, 0.8));
    reversed.set_pave2(Pave::new(5, 0.2));
    assert_eq!(first.pave1(), Pave::new(5, 0.2));
    assert_eq!(first.pave2(), Pave::new(8, 0.8));
    assert_eq!(first.range(), (0.2, 0.8));
    assert_eq!(first.indices(), (5, 8));
    assert!(first.has_same_bounds(&reversed));
}

#[test]
fn extra_paves_deduplicate_by_vertex_and_search_with_strict_tolerance() {
    let mut block = PaveBlock::default();
    block.append_ext_pave(Pave::new(7, 0.5));
    block.append_ext_pave(Pave::new(7, 0.75));
    block.append_ext_pave1(Pave::new(7, 0.9));
    assert!(block.is_to_update());
    assert_eq!(block.ext_paves(), &[Pave::new(7, 0.5), Pave::new(7, 0.9)]);
    let mut index = 99;
    assert!(block.contains_parameter(0.6, 0.11, &mut index));
    assert_eq!(index, 7);
    index = 99;
    assert!(!block.contains_parameter(0.625, 0.125, &mut index));
    assert_eq!(index, 99);
    block.remove_ext_pave(7);
    assert!(block.ext_paves().is_empty());
    assert!(!block.is_to_update());
}

#[test]
fn update_sorts_paves_and_builds_neighbor_intervals() {
    let mut source = PaveBlock::default();
    source.set_original_edge(12);
    source.set_pave1(Pave::new(1, 0.0));
    source.set_pave2(Pave::new(2, 1.0));
    source.append_ext_pave(Pave::new(4, 0.75));
    source.append_ext_pave(Pave::new(3, 0.25));
    let mut result = Vec::new();
    source.update(&mut result, true);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].indices(), (1, 3));
    assert_eq!(result[1].indices(), (3, 4));
    assert_eq!(result[2].indices(), (4, 2));
    assert!(result.iter().all(|block| block.original_edge() == 12));
    assert!(source.ext_paves().is_empty());
}

#[test]
fn update_without_endpoints_uses_only_extra_paves() {
    let mut source = PaveBlock::default();
    source.set_original_edge(9);
    source.append_ext_pave(Pave::new(2, 0.8));
    source.append_ext_pave(Pave::new(1, 0.2));
    let mut result = Vec::new();
    source.update(&mut result, false);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].indices(), (1, 2));
    assert_eq!(result[0].range(), (0.2, 0.8));
    assert_eq!(result[0].original_edge(), 9);
}

#[test]
fn shrunk_data_is_copied_and_dump_keeps_native_structure() {
    let mut block = PaveBlock::default();
    let bounds = BoundingBox3::new(Point3::new(1.0, 2.0, 3.0), Point3::new(4.0, 5.0, 6.0));
    block.set_shrunk_data(0.1, 0.9, &bounds, true);
    let (first, last, stored, splittable) = block.shrunk_data();
    assert_eq!((first, last, splittable), (0.1, 0.9, true));
    assert!(block.has_shrunk_data());
    assert_eq!(stored.get().unwrap(), [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let mut dump = String::new();
    block.dump(&mut dump).unwrap();
    assert!(dump.starts_with(" PB:{ E:-1 orE:-1 Pave1:"));
    assert!(dump.ends_with(" Pave2: { -1 99.000 } }"), "{dump:?}");
}
