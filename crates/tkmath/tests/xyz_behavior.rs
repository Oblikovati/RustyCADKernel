//! Additional gp_XYZ access, operator, normalization, and serialization contracts.
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::Xyz;
#[test]
fn coordinate_mutation_and_bounds() {
    let mut xyz = Xyz::default();
    xyz.set_x(1.0);
    xyz.set_y(2.0);
    xyz.set_z(3.0);
    assert_eq!(xyz.coordinates(), &[1.0, 2.0, 3.0]);
    xyz.coordinates_mut()[0] = 4.0;
    *xyz.change_coord(2).unwrap() = 5.0;
    xyz.set_coord(3, 6.0).unwrap();
    assert_eq!(xyz.coordinates(), &[4.0, 5.0, 6.0]);
    for index in [-1, 0, 4, i32::MAX] {
        assert_eq!(
            xyz.coord(index).unwrap_err().exception_type(),
            "Standard_OutOfRange"
        );
        assert!(xyz.set_coord(index, 0.0).is_err());
    }
}
#[test]
fn in_place_and_free_operators_keep_native_meaning() {
    let mut xyz = Xyz::new(1.0, 2.0, 3.0);
    let other = Xyz::new(2.0, 3.0, 4.0);
    xyz += other;
    xyz -= other;
    xyz *= 2.0;
    xyz /= 2.0;
    assert_eq!(xyz, Xyz::new(1.0, 2.0, 3.0));
    assert_eq!(2.0 * xyz, xyz * 2.0);
    assert_eq!(xyz * other, 20.0);
    let product = xyz.multiplied_components(other);
    xyz *= other;
    assert_eq!(xyz, product);
    let expected = xyz ^ other;
    xyz ^= other;
    assert_eq!(xyz, expected);
    let nested = xyz.cross_crossed(other, product);
    xyz.cross_cross(other, product);
    assert_eq!(xyz, nested);
    let reversed = xyz.reversed();
    xyz.reverse();
    assert_eq!(xyz, reversed);
}
#[test]
fn normalize_errors_tolerance_boundary_and_dump() {
    let mut zero = Xyz::default();
    assert_eq!(
        zero.normalize().unwrap_err().message(),
        "gp_XYZ::Normalize() - vector has zero norm"
    );
    assert_eq!(
        zero.normalized().unwrap_err().message(),
        "gp_XYZ::Normalized() - vector has zero norm"
    );
    let mut xyz = Xyz::new(3.0, 4.0, 0.0);
    let expected = xyz.normalized().unwrap();
    xyz.normalize().unwrap();
    assert_eq!(xyz, expected);
    assert!(!xyz.is_equal(&xyz, 0.0));
    let mut stream = DumpStream::default();
    xyz.dump_json(&mut stream).unwrap();
    assert_eq!(stream.text(), "\"gp_XYZ\": [0.6, 0.8, 0]");
    let mut restored = Xyz::default();
    let mut position = 1;
    assert!(
        restored
            .init_from_json(stream.text(), &mut position)
            .unwrap()
    );
    assert_eq!(restored, xyz);
}
