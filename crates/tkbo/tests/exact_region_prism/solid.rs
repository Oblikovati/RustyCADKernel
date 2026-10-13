// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::Vector3;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakePrism, BRepPrimAPIMakeSweep};
use rustycad_tktopalgo::{
    brep_builder_api::BRepBuilderAPIMakeShape, brep_check::BRepCheckAnalyzer, brep_gprop::BRepGProp,
};

/// Compare all native face reports and prism roles; example: `verify_region_prism(&region, 1.)`.
pub(super) fn verify_region_prism(region: &Shape, scale: f64) {
    assert!(mapped_validity(region));
    let expected = native_rows("region");
    assert_face_metrics(region, &expected[0], scale);
    let mut maker =
        BRepPrimAPIMakePrism::new(region, Vector3::new(0., 0., 2. * scale), true, true).unwrap();
    let solid = maker.shape().unwrap().clone();
    verify_solid(&solid, scale);
    verify_roles(region, &solid, &mut maker, scale);
}
fn mapped_validity(shape: &Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, true)
        .unwrap()
        .is_valid()
        .unwrap()
}
fn verify_solid(solid: &Shape, scale: f64) {
    assert_eq!(solid.shape_type().unwrap(), ShapeEnum::Solid);
    assert!(mapped_validity(solid));
    let counts = [
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        ShapeEnum::Face,
        ShapeEnum::Shell,
    ]
    .map(|kind| shapes(solid, kind).extent());
    assert_eq!(counts, [2, 3, 3, 1]);
    assert_eq!(counts[0] + counts[2] - counts[1], 2);
    verify_volume(solid, scale);
}
fn verify_volume(solid: &Shape, scale: f64) {
    let mut volume = GProps::new();
    BRepGProp::volume_properties(solid, &mut volume, true, false, false).unwrap();
    let expected = native_rows("solid");
    assert_close(volume.mass() / scale.powi(3), expected[0][0]);
    assert_eq!(
        shapes(solid, ShapeEnum::Face).extent() as f64,
        expected[0][1]
    );
    assert_eq!(expected[0][2], 1.);
}
fn verify_roles(region: &Shape, solid: &Shape, maker: &mut BRepPrimAPIMakePrism, scale: f64) {
    let edges = shapes(region, ShapeEnum::Edge);
    let generated = maker.generated(edges.find_key(1).unwrap()).unwrap().clone();
    assert_eq!(generated.len(), 1);
    let roles = [
        generated.first().unwrap().clone(),
        maker.first_shape().unwrap(),
        maker.last_shape().unwrap(),
    ];
    let faces = shapes(solid, ShapeEnum::Face);
    let expected = native_rows("face");
    assert_eq!(roles.len(), faces.extent());
    assert_eq!(roles.len(), expected.len());
    for (role, row) in roles.iter().zip(expected) {
        assert_face_metrics(role, &row, scale);
    }
    verify_role_coverage(&roles, &faces);
}
fn verify_role_coverage(roles: &[Shape], faces: &IndexedMapOfShape) {
    for index in 1..=faces.extent() {
        assert_eq!(
            roles
                .iter()
                .filter(|role| role.is_same(faces.find_key(index).unwrap()))
                .count(),
            1
        );
    }
}
fn face_metrics(shape: &Shape, scale: f64) -> [f64; 8] {
    let mut props = GProps::new();
    BRepGProp::surface_properties(shape, &mut props, false, false).unwrap();
    let center = props.centre_of_mass();
    let adaptor = BRepAdaptorSurface::with_face(&face(shape).unwrap(), true).unwrap();
    [
        props.mass() / scale.powi(2),
        center.x() / scale,
        center.y() / scale,
        center.z() / scale,
        shapes(shape, ShapeEnum::Wire).extent() as f64,
        shapes(shape, ShapeEnum::Edge).extent() as f64,
        adaptor.get_type().unwrap() as i32 as f64,
        f64::from(mapped_validity(shape)),
    ]
}
fn assert_face_metrics(shape: &Shape, expected: &[f64], scale: f64) {
    let actual = face_metrics(shape, scale);
    assert_eq!(actual[4..], expected[4..]);
    for (actual, expected) in actual[..4].iter().zip(&expected[..4]) {
        assert_close(*actual, *expected);
    }
}
fn assert_close(actual: f64, expected: f64) {
    // tol:metric independent integrator receipt in normalized fixture units, not modelling policy.
    assert!(
        (actual - expected).abs() < 1e-8,
        "metric {actual}: expected {expected} within 1e-8"
    );
}
fn native_rows(label: &str) -> Vec<Vec<f64>> {
    include_str!("occt-8.0.0.csv")
        .lines()
        .filter_map(|line| line.strip_prefix(&format!("{label},")))
        .map(|line| {
            line.split(',')
                .map(|value| value.parse().unwrap())
                .collect()
        })
        .collect()
}
