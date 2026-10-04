//! Native subset selection, ancestor filtering, duplicate fencing and repeated append.
use rustycad_tkbo::bop_ds::{BOPDSSubIterator, DataStructure};
use rustycad_tkernel::ncollection::{BaseAllocator, List};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use std::fmt::Write;

fn overlapping_boxes() -> DataStructure {
    let mut ds = DataStructure::default();
    let shapes = [0., 0.5].map(|x| {
        BRepPrimAPIMakeBox::at_point(Point3::new(x, 0., 0.), 1., 1., 1.)
            .unwrap()
            .shape()
            .unwrap()
            .clone()
    });
    ds.set_arguments(&shapes);
    ds.init(1e-7).unwrap();
    ds
}

fn selected_indices(ds: &DataStructure, mode: i32) -> List<i32> {
    let mut indices = List::default();
    for index in (0..ds.nb_source_shapes()).rev() {
        let kind = ds.shape_info(index).unwrap().shape_type();
        let selected = match mode {
            0 => matches!(kind, ShapeEnum::Vertex | ShapeEnum::Edge | ShapeEnum::Face),
            1 => kind == ShapeEnum::Vertex,
            2 => kind == ShapeEnum::Edge,
            3 => kind == ShapeEnum::Face,
            _ => false,
        };
        if selected {
            indices.append(index);
        }
        if selected && mode == 2 {
            indices.append(index);
        }
    }
    indices
}

fn append_pairs(
    iterator: &mut BOPDSSubIterator<'_>,
    first: i32,
    second: i32,
    pass: i32,
    rows: &mut String,
) {
    iterator.initialize();
    write!(
        rows,
        "{first} {second} {pass} {}",
        iterator.expected_length()
    )
    .unwrap();
    while iterator.more() {
        let (a, b) = iterator.value().unwrap();
        write!(rows, " {a} {b}").unwrap();
        iterator.next();
    }
    rows.push('\n');
}

fn run_subsets(ds: &DataStructure, first_mode: i32, second_mode: i32, rows: &mut String) {
    let first = selected_indices(ds, first_mode);
    let second = selected_indices(ds, second_mode);
    let mut iterator = BOPDSSubIterator::new();
    iterator.set_ds(Some(ds));
    iterator.set_subset1(&first);
    iterator.set_subset2(&second);
    assert!(std::ptr::eq(iterator.ds().unwrap(), ds));
    assert!(std::ptr::eq(iterator.subset1().unwrap(), &first));
    assert!(std::ptr::eq(iterator.subset2().unwrap(), &second));
    iterator.prepare().unwrap();
    append_pairs(&mut iterator, first_mode, second_mode, 0, rows);
    iterator.intersect().unwrap();
    append_pairs(&mut iterator, first_mode, second_mode, 1, rows);
}

#[test]
fn subset_pair_lists_match_fifty_native_scenarios() {
    let ds = overlapping_boxes();
    let mut rows = String::new();
    for first in 0..5 {
        for second in 0..5 {
            run_subsets(&ds, first, second, &mut rows);
        }
    }
    assert_eq!(rows.lines().count(), 50);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_sub_iterator.expected")
    );
}

#[test]
fn absent_ds_prepares_empty_and_undefined_access_returns_failure() {
    let mut iterator = BOPDSSubIterator::with_allocator(BaseAllocator::common());
    iterator.prepare().unwrap();
    iterator.initialize();
    assert!(!iterator.more());
    assert_eq!(iterator.expected_length(), 0);
    assert!(iterator.ds().is_err());
    assert!(iterator.subset1().is_err());
    assert!(iterator.subset2().is_err());
    assert!(iterator.value().is_err());
}
