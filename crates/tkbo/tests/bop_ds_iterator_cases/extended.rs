use super::*;
use rustycad_tkernel::ncollection::Map;
#[test]
fn native_extended_lists_accumulate_and_survive_prepare() {
    assert_eq!(BOPDSIterator::nb_ext_interfs(), 4);
    let mut actual = String::new();
    for mode in 0..5 {
        for repeat in 1..3 {
            let mut structure = DataStructure::default();
            structure.set_arguments(&[
                box_shape(Point3::default()),
                box_shape(Point3::new(0.5, 0., 0.)),
            ]);
            structure.init(1e-7).unwrap();
            let mut iterator = BOPDSIterator::default();
            iterator.prepare(&structure).unwrap();
            let selected = selected_indices(&mut structure, mode);
            for _ in 0..repeat {
                iterator.intersect_extended(&structure, &selected).unwrap();
            }
            actual.push_str(&format!("case {mode} {repeat}\n"));
            extended_output(&mut iterator, &mut actual);
            iterator.prepare(&structure).unwrap();
            extended_output(&mut iterator, &mut actual);
        }
    }
    assert_eq!(
        actual,
        include_str!("../../../../tools/oracle/tkbo_iterator_extended.expected")
    );
}
fn selected_indices(structure: &mut DataStructure, mode: i32) -> Map<i32> {
    let mut selected = Map::default();
    let mut first = -1;
    let mut last = -1;
    for index in 0..structure.nb_source_shapes() {
        let kind = structure.shape_info(index).unwrap().shape_type();
        if kind == ShapeEnum::Vertex {
            if first < 0 {
                first = index;
            }
            last = index;
            if mode == 2 || mode == 3 {
                selected.add(index);
            }
        }
        if mode == 3 && kind == ShapeEnum::Edge {
            selected.add(index);
        }
    }
    if mode == 1 || mode == 4 {
        selected.add(first);
        structure
            .change_shape_info(first)
            .unwrap()
            .change_box()
            .enlarge(0.75);
    }
    if mode == 4 {
        structure.add_shape_sd(first, last);
    }
    selected
}
fn extended_output(iterator: &mut BOPDSIterator, output: &mut String) {
    use ShapeEnum::{Edge, Face, Vertex};
    for (first, second) in [
        (Vertex, Vertex),
        (Vertex, Edge),
        (Edge, Edge),
        (Vertex, Face),
        (Face, Face),
    ] {
        iterator.initialize(first, second);
        output.push_str(&format!("{}", iterator.expected_length()));
        while iterator.more() {
            let (first, second) = iterator.value().unwrap();
            output.push_str(&format!(" {first} {second}"));
            iterator.next();
        }
        output.push('\n');
    }
}
