//! Records the current Rust/native divergence for moved Poly node-array aliases.
use rustycad_tkmath::poly::{ArrayOfNodes, ArrayOfUvNodes};

#[test]
fn node_move_currently_drops_the_native_source_view() {
    let mut source = ArrayOfNodes::new(2).unwrap();
    source.set_coordinates(0, [1.0, 2.0, 3.0]).unwrap();
    let mut target = ArrayOfNodes::default();
    target.move_from(&mut source);
    assert_eq!((source.len(), target.len()), (0, 2));
    assert!(source.coordinates(0).is_err());
    target.set_coordinates(0, [4.0, 5.0, 6.0]).unwrap();
    assert_eq!(target.coordinates(0).unwrap(), [4.0, 5.0, 6.0]);
}

#[test]
fn uv_node_move_currently_drops_the_native_source_view() {
    let mut source = ArrayOfUvNodes::new(2).unwrap();
    source.set_coordinates(0, [1.0, 2.0]).unwrap();
    let mut target = ArrayOfUvNodes::default();
    target.move_from(&mut source);
    assert_eq!((source.len(), target.len()), (0, 2));
    assert!(source.coordinates(0).is_err());
    target.set_coordinates(0, [4.0, 5.0]).unwrap();
    assert_eq!(target.coordinates(0).unwrap(), [4.0, 5.0]);
}
