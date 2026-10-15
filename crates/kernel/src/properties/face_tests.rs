// SPDX-License-Identifier: GPL-2.0-only
//! Face-level oracles: surface kind, area, area centroid, loops and orientation.
use super::{FaceProperties, IntegrationMethod, LoopRole, face_properties};
use crate::test_shapes::{
    annulus_prism, assert_point_within, assert_within, cone, cylinder, elliptic_prism, faces,
    sphere, torus,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::geom_abs::SurfaceType;
use std::f64::consts::PI;

fn of_kind(shape: &Shape, kind: SurfaceType) -> Vec<FaceProperties> {
    let all = faces(shape)
        .into_iter()
        .map(|face| face_properties(&face).unwrap());
    all.filter(|properties| properties.surface == kind)
        .collect()
}

fn check_area_centroid(face: &FaceProperties, area: f64, centroid: [f64; 3], what: &str) {
    assert_within(face.area.value, face.area.error_bound, area, what);
    assert_point_within(&face.centroid, centroid, what);
}

#[test]
fn cylinder_lateral_and_caps() {
    let (r, h) = (1.5, 4.0);
    let shape = cylinder(r, h);
    let lateral = &of_kind(&shape, SurfaceType::Cylinder)[0];
    check_area_centroid(lateral, 2.0 * PI * r * h, [0.0, 0.0, h / 2.0], "lateral");
    assert_eq!(lateral.method, IntegrationMethod::ClosedForm);
    assert_eq!(lateral.loops.len(), 1);
    // The seam is used twice: four edge uses close the [0, 2pi] x [0, h] rectangle.
    assert_eq!(lateral.loops[0].edge_uses, 4);
    let domain = lateral.loops[0].parameter_area;
    assert_within(
        domain.value,
        domain.error_bound,
        2.0 * PI * h,
        "parameter area",
    );
    check_caps(&shape, r);
}

fn check_caps(shape: &Shape, r: f64) {
    let caps = of_kind(shape, SurfaceType::Plane);
    let heights: Vec<f64> = caps.iter().map(|cap| cap.centroid.point[2]).collect();
    for (cap, z) in caps.iter().zip(heights) {
        check_area_centroid(cap, PI * r * r, [0.0, 0.0, z], "cap");
    }
    assert_ne!(caps[0].orientation, caps[1].orientation);
}

#[test]
fn sphere_cone_and_torus_surfaces() {
    let sphere_face = &of_kind(&sphere(2.0), SurfaceType::Sphere)[0];
    check_area_centroid(sphere_face, 16.0 * PI, [0.0; 3], "sphere");
    let (r, h) = (2.0, 3.0);
    let lateral = &of_kind(&cone(r, 0.0, h), SurfaceType::Cone)[0];
    // The lateral surface of a right cone has its area centroid at h/3 above the base.
    check_area_centroid(lateral, PI * r * r.hypot(h), [0.0, 0.0, h / 3.0], "cone");
    let ring = &of_kind(&torus(3.0, 1.0), SurfaceType::Torus)[0];
    check_area_centroid(ring, 12.0 * PI * PI, [0.0; 3], "torus");
}

#[test]
fn annulus_cap_has_outer_and_inner_loops() {
    let (outer, inner) = (2.0, 0.75);
    let shape = annulus_prism(outer, inner, 1.5);
    let cap = &of_kind(&shape, SurfaceType::Plane)[0];
    assert_eq!(cap.loops.len(), 2);
    let roles: Vec<LoopRole> = cap.loops.iter().map(|summary| summary.role).collect();
    assert!(roles.contains(&LoopRole::Outer) && roles.contains(&LoopRole::Inner));
    let signed: f64 = cap
        .loops
        .iter()
        .map(|summary| summary.parameter_area.value)
        .sum();
    assert!((signed - PI * (outer * outer - inner * inner)).abs() < 1e-12);
    assert_eq!(of_kind(&shape, SurfaceType::Cylinder).len(), 2);
}

#[test]
fn elliptic_trims_and_free_form_surface_use_adaptive_quadrature() {
    let (a, b) = (2.0, 1.0);
    let shape = elliptic_prism(a, b, 3.0);
    let cap = &of_kind(&shape, SurfaceType::Plane)[0];
    assert_eq!(cap.method, IntegrationMethod::AdaptiveQuadrature);
    let exact = PI * a * b;
    assert!((cap.area.value - exact).abs() <= cap.area.error_bound);
    assert!(cap.area.error_bound < 1e-10);
    let lateral = faces(&shape)
        .into_iter()
        .map(|face| face_properties(&face).unwrap());
    let free_form: Vec<FaceProperties> = lateral
        .filter(|face| face.surface != SurfaceType::Plane)
        .collect();
    assert_eq!(free_form[0].method, IntegrationMethod::AdaptiveQuadrature);
    assert!(free_form[0].area.error_bound < 1e-9);
}
