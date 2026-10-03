//! Wrapper behavior compared with the pinned ShapeUpgrade_ShapeDivideArea oracle.
use rustycad_tkernel::precision;
use rustycad_tkshhealing::shape_upgrade::{
    ShapeDivider, ShapeUpgradeFaceDivideArea, ShapeUpgradeShapeDivideArea,
};

fn observe(divider: &ShapeUpgradeShapeDivideArea) -> [f64; 5] {
    let first = (divider as &dyn ShapeDivider).split_face_tool();
    let second = (divider as &dyn ShapeDivider).split_face_tool();
    let child = first.cast::<ShapeUpgradeFaceDivideArea>();
    [
        divider.max_area(),
        f64::from(divider.nb_parts()),
        child.max_area(),
        f64::from(child.nb_parts()),
        f64::from(first == second),
    ]
}

#[test]
fn defaults_and_setter_side_effects_match_native_rows() {
    let actual: Vec<[f64; 5]> = vec![default_row(), mode_toggle_row()]
        .into_iter()
        .chain(uv_rows())
        .collect();
    assert_eq!(actual, expected_rows());
}

fn default_row() -> [f64; 5] {
    let divider = ShapeUpgradeShapeDivideArea::empty();
    assert_eq!(divider.max_area(), precision::infinite());
    assert_eq!(divider.nb_parts(), 0);
    assert!(!divider.splitting_by_number());
    observe(&divider)
}

fn mode_toggle_row() -> [f64; 5] {
    let divider = ShapeUpgradeShapeDivideArea::empty();
    divider.set_max_area(12.5);
    divider.set_nb_parts(7);
    divider.set_splitting_by_number(true);
    divider.set_splitting_by_number(false);
    assert!(!divider.splitting_by_number());
    observe(&divider)
}

fn uv_rows() -> Vec<[f64; 5]> {
    let divider = ShapeUpgradeShapeDivideArea::empty();
    divider.set_numbers_uv_splits(2, 3).unwrap();
    assert!(divider.splitting_by_number());
    let positive = observe(&divider);
    divider.set_numbers_uv_splits(0, 4).unwrap();
    vec![positive, observe(&divider)]
}

fn expected_rows() -> Vec<[f64; 5]> {
    include_str!("../../../tools/oracle/tkshhealing_shape_divide_area.expected")
        .lines()
        .map(|line| {
            let row: Vec<f64> = line
                .split_whitespace()
                .skip(1)
                .map(|word| word.parse().unwrap())
                .collect();
            row.try_into().unwrap()
        })
        .collect()
}

#[test]
fn overflowing_uv_counts_retain_last_valid_configuration() {
    let divider = ShapeUpgradeShapeDivideArea::empty();
    divider.set_numbers_uv_splits(2, 3).unwrap();
    let error = divider.set_numbers_uv_splits(i32::MAX, 2).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert_eq!(divider.max_area(), -1.);
    assert_eq!(divider.nb_parts(), 6);
    assert!(divider.splitting_by_number());
    let tool = (&divider as &dyn ShapeDivider)
        .split_face_tool()
        .cast::<ShapeUpgradeFaceDivideArea>();
    assert_eq!(tool.max_area(), -1.);
    assert_eq!(tool.nb_parts(), 6);
}
