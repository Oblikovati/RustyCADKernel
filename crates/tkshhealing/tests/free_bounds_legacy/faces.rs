#[test]
fn SingleFaceNoCrash() {
    let face = square_boundary_face(0.);
    let result = ShapeAnalysisFreeBounds::forecast(&face, 0.01, true, true).unwrap();
    let _closed = result.closed_wires();
    let _open = result.open_wires();
}
#[test]
fn TwoDisjointFacesNoCrash() {
    let compound = healing_support::compound(&[
        square_boundary_face(0.).into_shape(),
        square_boundary_face(5.).into_shape(),
    ]);
    let result = ShapeAnalysisFreeBounds::forecast(&compound, 0.01, true, true).unwrap();
    assert_eq!(
        Explorer::new(result.closed_wires(), ShapeEnum::Wire, ShapeEnum::Shape)
            .unwrap()
            .count(),
        2
    );
    assert_eq!(
        Explorer::new(result.open_wires(), ShapeEnum::Wire, ShapeEnum::Shape)
            .unwrap()
            .count(),
        0
    );
}
