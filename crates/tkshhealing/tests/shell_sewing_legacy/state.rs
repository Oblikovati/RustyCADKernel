#[test]
fn InstanceCanProcessSuccessiveShapes() {
    let mut sewer = ShapeUpgradeShellSewing::new();
    let first = sewer
        .apply_sewing(&box_shape(0., (1., 2., 3.)), 0.)
        .unwrap();
    let second = sewer
        .apply_sewing(&box_shape(10., (4., 5., 6.)), 0.)
        .unwrap();
    assert!(!first.is_null());
    solid_counts(&second, 1, 6);
    valid(&second);
}
#[test]
fn ReapplyingToSewedShapeIsStable() {
    let input = open_shell(0.);
    let mut sewer = ShapeUpgradeShellSewing::new();
    let first = sewer.apply_sewing(&input, 1e-6).unwrap();
    assert!(!first.is_null());
    let second = sewer.apply_sewing(&first, 1e-6).unwrap();
    counts(&second, 2, 7);
}
