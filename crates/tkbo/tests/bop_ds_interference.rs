//! Regression tests for vertex-based BOPDS interference records.
use rustycad_tkbo::bop_ds::{InterfVE, InterfVF, InterfVV};

#[test]
fn vertex_vertex_interference_preserves_index_sentinels_and_queries() {
    let mut interference = InterfVV::default();
    assert_eq!(interference.indices(), (-1, -1));
    assert_eq!(interference.index_new(), -1);
    assert!(!interference.has_index_new());
    assert_eq!(interference.get_index_new(), None);
    assert!(interference.contains(-1));

    interference.set_indices(3, 8);
    assert_eq!(interference.indices(), (3, 8));
    interference.set_index1(-4);
    interference.set_index2(0);
    assert_eq!(interference.index1(), -4);
    assert_eq!(interference.index2(), 0);
    assert_eq!(interference.opposite_index(-4), 0);
    assert_eq!(interference.opposite_index(0), -4);
    assert_eq!(interference.opposite_index(19), -1);
    assert!(interference.contains(-4));
    assert!(!interference.contains(8));

    interference.set_index_new(-2);
    let mut output = 77;
    assert!(!interference.has_index_new_into(&mut output));
    assert_eq!(output, -2);
    assert!(interference.has_index_new());
    assert_eq!(interference.get_index_new(), Some(-2));
    interference.set_index_new(0);
    assert!(interference.has_index_new_into(&mut output));
    assert_eq!(output, 0);
    interference.set_index_new(-1);
    assert!(!interference.has_index_new());
    assert_eq!(interference.get_index_new(), None);
}

#[test]
fn vertex_edge_and_face_interferences_store_parameters_independently() {
    let mut edge = InterfVE::default();
    assert_eq!(edge.parameter(), 0.0);
    edge.set_parameter(-0.0);
    assert_eq!(edge.parameter().to_bits(), (-0.0_f64).to_bits());
    edge.set_indices(6, 9);
    assert_eq!(edge.opposite_index(9), 6);

    let mut face = InterfVF::default();
    assert_eq!(face.uv(), (0.0, 0.0));
    face.set_uv(-1.25, 2.5);
    assert_eq!(face.uv(), (-1.25, 2.5));
    face.set_indices(11, 13);
    assert!(face.contains(13));
}
