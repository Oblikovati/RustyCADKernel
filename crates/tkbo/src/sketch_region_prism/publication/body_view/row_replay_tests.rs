// SPDX-License-Identifier: GPL-2.0-only
//! Partial symbolic row replay only: no actual publication correspondence or body acceptance.
use super::*;
use crate::sketch_region_prism::frozen::FrozenSweepRole;
fn symbolic_rows() -> Vec<u8> {
    let names = test_fixture::names();
    let source = test_fixture::node(test_fixture::key(2051, FrozenSweepRole::First, 6));
    let member = members::project(&source, &names, &[member_tests::metadata()]).unwrap();
    let (source_use, vertex) = occurrence_tests::operands();
    let occurrence = occurrences::project(&source_use, &names, &[vertex]).unwrap();
    let face = faces::project(&face_tests::captured_face(), &names).unwrap();
    let mut bytes = b"Martensite.SymbolicBodyRowsOnly\0".to_vec();
    bytes.extend_from_slice(names.replay_bytes());
    replay::member(&member, &mut bytes);
    replay::occurrence(&occurrence, &mut bytes);
    replay::face(&face, &mut bytes);
    bytes
}
#[test]
fn symbolic_rows_match_serial_and_independent_thread_encodings() {
    let expected = symbolic_rows();
    let threads: Vec<_> = (0..4).map(|_| std::thread::spawn(symbolic_rows)).collect();
    for thread in threads {
        assert_eq!(thread.join().unwrap(), expected);
    }
    assert_eq!(symbolic_rows(), expected);
}
#[test]
fn symbolic_body_row_process_replay() {
    let bytes = symbolic_rows();
    println!(
        "SYMBOLIC_BODY_ROWS={}",
        bytes
            .iter()
            .map(|value| format!("{value:02x}"))
            .collect::<String>()
    );
}
