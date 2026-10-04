//! Protected gp orthogonalization replay; pinned native fixture in tools/oracle.
use rustycad_tkmath::gp::{Transform2, Transform3, TransformForm};

fn stored_transform(matrix: &str) -> Transform3 {
    let json =
        format!("\"Location\": [5, 6, 7], \"Matrix\": [{matrix}], \"shape\": 7, \"scale\": -2");
    let mut transform = Transform3::default();
    assert!(transform.init_from_json(&json, &mut 1).unwrap());
    transform
}

#[test]
fn spatial_hook_orthonormalizes_without_reclassifying_or_rescaling() {
    let mut transform = stored_transform("2, 1, 1, 0, 3, 1, 0, 0, 4");
    transform.orthogonalize().unwrap();
    assert_eq!(transform.matrix(), Transform3::default().matrix());
    assert_eq!(transform.form(), TransformForm::Compound);
    assert_eq!(transform.scale_factor(), -2.0);
    assert_eq!(transform.translation_part().coordinates(), &[5.0, 6.0, 7.0]);
    let normalized = transform;
    transform.orthogonalize().unwrap();
    assert_eq!(transform, normalized);
}

#[test]
fn planar_hook_orthonormalizes_without_reclassifying_or_rescaling() {
    let mut transform = Transform2::from(stored_transform("2, 1, 1, 0, 3, 1, 0, 0, 4"));
    transform.orthogonalize().unwrap();
    assert_eq!(transform.matrix(), Transform2::default().matrix());
    assert_eq!(transform.form(), TransformForm::Compound);
    assert_eq!(transform.scale_factor(), -2.0);
    assert_eq!(transform.translation_part().coordinates(), &[5.0, 6.0]);
    let normalized = transform;
    transform.orthogonalize().unwrap();
    assert_eq!(transform, normalized);
}

#[test]
fn spatial_hook_retains_every_field_when_a_column_cannot_normalize() {
    for matrix in ["0, 1, 1, 0, 3, 1, 0, 0, 4", "2, 4, 1, 0, 0, 1, 0, 0, 4"] {
        let mut transform = stored_transform(matrix);
        let before = transform;
        let error = transform.orthogonalize().unwrap_err();
        assert_eq!(error.exception_type(), "Standard_ConstructionError");
        assert_eq!(transform, before);
    }
}

#[test]
fn planar_hook_retains_every_field_when_a_column_cannot_normalize() {
    for matrix in ["0, 1, 1, 0, 3, 1, 0, 0, 4", "2, 4, 1, 0, 0, 1, 0, 0, 4"] {
        let mut transform = Transform2::from(stored_transform(matrix));
        let before = transform;
        let error = transform.orthogonalize().unwrap_err();
        assert_eq!(error.exception_type(), "Standard_ConstructionError");
        assert_eq!(transform, before);
    }
}
