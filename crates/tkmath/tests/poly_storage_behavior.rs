//! Borrowed writes and conversion boundaries supplement the native declarations.
use rustycad_tkmath::{
    gp::{Point2, Point3},
    poly::{ArrayOfNodes, ArrayOfUvNodes},
};
#[test]
fn float_nodes_round_and_write_through_borrowed_storage() {
    let mut buffer = [[0.0_f32; 3]; 2];
    {
        let mut nodes = ArrayOfNodes::from_float_slice(&mut buffer).unwrap();
        nodes
            .set_value(0, Point3::new(1.0 / 3.0, -0.0, 1e100))
            .unwrap();
        let mut copy = nodes.clone();
        copy.resize(3, true).unwrap();
        assert_eq!(copy.value(0).unwrap().x(), f64::from((1.0 / 3.0) as f32));
        assert_eq!(copy.value(0).unwrap().y().to_bits(), (-0.0_f64).to_bits());
        assert!(copy.value(0).unwrap().z().is_infinite());
        assert!(nodes.set_double_precision(true).is_err());
        assert!(nodes.value(2).is_err());
    }
    assert_eq!(buffer[0], [((1.0 / 3.0) as f32), -0.0, f32::INFINITY]);
}
#[test]
fn uv_assignment_preserves_precision_and_rejects_length_mismatch() {
    let mut source = ArrayOfUvNodes::new(2).unwrap();
    source.set_value(1, Point2::new(2.0, 3.0)).unwrap();
    let mut target = ArrayOfUvNodes::default();
    assert!(target.is_empty());
    target.set_double_precision(false).unwrap();
    target.resize(1, false).unwrap();
    assert_eq!(
        target.assign(&source).unwrap_err().exception_type(),
        "Standard_DimensionMismatch"
    );
    target.resize(2, false).unwrap();
    target.assign(&source).unwrap();
    assert!(!target.is_double_precision());
    assert_eq!(target.coordinates(1).unwrap(), [2.0, 3.0]);
    source.assign(&ArrayOfUvNodes::new(1).unwrap()).unwrap_err();
}
