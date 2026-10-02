//! Mutable interference-array access on `BOPDS_DS`.
use rustycad_tkbo::bop_ds::{
    DataStructure, InterfEE, InterfEF, InterfFF, InterfVE, InterfVF, InterfVV,
};

#[test]
fn interference_arrays_keep_distinct_records_and_clear_with_the_structure() {
    let mut structure = DataStructure::default();
    append_vertex_records(&mut structure);
    append_edge_records(&mut structure);
    append_face_records(&mut structure);
    assert_interference_counts(&mut structure);
    structure.clear();
    assert_all_arrays_empty(&mut structure);
}

fn append_vertex_records(structure: &mut DataStructure) {
    structure.interf_vv().append(InterfVV::default());
    structure.interf_ve().append(InterfVE::default());
    structure.interf_vf().append(InterfVF::default());
}

fn append_edge_records(structure: &mut DataStructure) {
    structure.interf_ee().append(InterfEE::default());
    structure.interf_ef().append(InterfEF::default());
}

fn append_face_records(structure: &mut DataStructure) {
    structure.interf_ff().append(InterfFF::default());
}

fn assert_interference_counts(structure: &mut DataStructure) {
    assert_eq!(structure.interf_vv().len(), 1);
    assert_eq!(structure.interf_ve().len(), 1);
    assert_eq!(structure.interf_vf().len(), 1);
    assert_eq!(structure.interf_ee().len(), 1);
    assert_eq!(structure.interf_ef().len(), 1);
    assert_eq!(structure.interf_ff().len(), 1);
}

fn assert_all_arrays_empty(structure: &mut DataStructure) {
    assert!(structure.interf_vv().is_empty());
    assert!(structure.interf_ve().is_empty());
    assert!(structure.interf_vf().is_empty());
    assert!(structure.interf_ee().is_empty());
    assert!(structure.interf_ef().is_empty());
    assert!(structure.interf_ff().is_empty());
}
