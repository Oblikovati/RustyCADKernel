// SPDX-License-Identifier: GPL-2.0-only
//! Independent closed-form oracles: textbook volume, area and centroid of each primitive.
use super::{BodyProperties, IntegrationMethod, body_properties};
use crate::test_shapes::{
    annulus_prism, assert_point_within, assert_within, box_solid, cone, cylinder, elliptic_prism,
    moved, sphere, torus,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::f64::consts::PI;

struct Expected {
    volume: f64,
    area: f64,
    centroid: [f64; 3],
}

fn check(shape: &Shape, expected: Expected, what: &str) -> BodyProperties {
    let properties = body_properties(shape).unwrap();
    assert_eq!(properties.method, IntegrationMethod::ClosedForm, "{what}");
    let (volume, area) = (properties.volume, properties.area);
    assert_within(volume.value, volume.error_bound, expected.volume, what);
    assert_within(area.value, area.error_bound, expected.area, what);
    assert_point_within(&properties.centroid, expected.centroid, what);
    properties
}

#[test]
fn box_matches_closed_form() {
    let expected = Expected {
        volume: 24.0,
        area: 52.0,
        centroid: [1.0, 1.5, 2.0],
    };
    let properties = check(&box_solid(2.0, 3.0, 4.0), expected, "box");
    assert_eq!(properties.face_count, 6);
}

#[test]
fn cylinder_matches_closed_form() {
    let (r, h) = (1.5, 4.0);
    let expected = Expected {
        volume: PI * r * r * h,
        area: 2.0 * PI * r * r + 2.0 * PI * r * h,
        centroid: [0.0, 0.0, h / 2.0],
    };
    check(&cylinder(r, h), expected, "cylinder");
}

#[test]
fn sphere_matches_closed_form() {
    let r: f64 = 2.5;
    let expected = Expected {
        volume: 4.0 / 3.0 * PI * r.powi(3),
        area: 4.0 * PI * r * r,
        centroid: [0.0; 3],
    };
    check(&sphere(r), expected, "sphere");
}

#[test]
fn cone_and_frustum_match_closed_form() {
    let (r1, h) = (2.0, 3.0);
    let pointed = Expected {
        volume: PI * r1 * r1 * h / 3.0,
        area: PI * r1 * r1 + PI * r1 * r1.hypot(h),
        centroid: [0.0, 0.0, h / 4.0],
    };
    check(&cone(r1, 0.0, h), pointed, "cone");
    let r2 = 0.5;
    let sum = r1 * r1 + r1 * r2 + r2 * r2;
    let frustum = Expected {
        volume: PI * h * sum / 3.0,
        area: PI * (r1 * r1 + r2 * r2) + PI * (r1 + r2) * (r1 - r2).hypot(h),
        centroid: [
            0.0,
            0.0,
            h * (r1 * r1 + 2.0 * r1 * r2 + 3.0 * r2 * r2) / (4.0 * sum),
        ],
    };
    check(&cone(r1, r2, h), frustum, "frustum");
}

#[test]
fn torus_matches_pappus() {
    let (major, minor) = (3.0, 0.75);
    let expected = Expected {
        volume: 2.0 * PI * PI * major * minor * minor,
        area: 4.0 * PI * PI * major * minor,
        centroid: [0.0; 3],
    };
    check(&torus(major, minor), expected, "torus");
}

#[test]
fn annulus_prism_matches_closed_form() {
    let (outer, inner, h) = (2.0, 0.75, 1.5);
    let ring = PI * (outer * outer - inner * inner);
    let expected = Expected {
        volume: ring * h,
        area: 2.0 * ring + 2.0 * PI * (outer + inner) * h,
        centroid: [0.0, 0.0, h / 2.0],
    };
    let properties = check(&annulus_prism(outer, inner, h), expected, "annulus prism");
    assert_eq!(properties.face_count, 4);
}

#[test]
fn located_solids_keep_volume_and_move_the_centroid() {
    let base = body_properties(&torus(3.0, 1.0)).unwrap();
    let placed = body_properties(&moved(&torus(3.0, 1.0), 0.7)).unwrap();
    assert_within(
        placed.volume.value,
        placed.volume.error_bound,
        base.volume.value,
        "volume",
    );
    let expected = body_properties(&moved(&sphere(1.0), 0.7))
        .unwrap()
        .centroid
        .point;
    assert_point_within(&placed.centroid, expected, "moved centroid");
}

#[test]
fn free_form_lateral_surface_uses_nested_quadrature() {
    let (a, b, h) = (2.0, 1.0, 3.0);
    let properties = body_properties(&elliptic_prism(a, b, h)).unwrap();
    assert_eq!(properties.method, IntegrationMethod::AdaptiveQuadrature);
    let (volume, exact) = (properties.volume, PI * a * b * h);
    assert!(
        (volume.value - exact).abs() <= volume.error_bound,
        "{volume:?} vs {exact}"
    );
    assert!(volume.error_bound < 1e-9);
    let (area, exact) = (
        properties.area,
        2.0 * PI * a * b + h * ellipse_perimeter(a, b),
    );
    assert!(
        (area.value - exact).abs() <= area.error_bound,
        "{area:?} vs {exact}"
    );
    let centroid = properties.centroid;
    for (axis, expected) in [0.0, 0.0, h / 2.0].into_iter().enumerate() {
        assert!(
            (centroid.point[axis] - expected).abs() <= centroid.error_bound,
            "{centroid:?}"
        );
    }
}

/// `4 a E(e)` by the arithmetic-geometric mean (Gauss): an oracle independent of any
/// quadrature; `K = pi / (2 AGM(1, b/a))`, `E = K (1 - sum 2^(n-1) c_n^2)`.
fn ellipse_perimeter(a: f64, b: f64) -> f64 {
    let (mut x, mut y) = (1.0_f64, b / a);
    let mut c_squared = 1.0 - y * y;
    let (mut sum, mut power) = (0.5 * c_squared, 0.5);
    while c_squared > 1e-32 {
        let (mean, geometric) = (0.5 * (x + y), (x * y).sqrt());
        c_squared = (0.5 * (x - y)).powi(2);
        (x, y, power) = (mean, geometric, 2.0 * power);
        sum += power * c_squared;
    }
    4.0 * a * PI / (2.0 * x) * (1.0 - sum)
}

#[test]
fn ellipse_perimeter_oracle_matches_the_circle_and_a_known_value() {
    assert!((ellipse_perimeter(1.0, 1.0) - 2.0 * PI).abs() < 1e-15);
    // Ramanujan's second approximation is accurate to about 1e-9 at this eccentricity.
    let h = (1.0_f64 / 3.0).powi(2);
    let ramanujan = PI * 3.0 * (1.0 + 3.0 * h / (10.0 + (4.0 - 3.0 * h).sqrt()));
    assert!((ellipse_perimeter(2.0, 1.0) - ramanujan).abs() < 1e-6);
}

#[test]
fn free_form_area_agrees_with_the_native_fixed_order_integration() {
    let shape = elliptic_prism(2.0, 1.0, 3.0);
    let mut native = GProps::new();
    BRepGProp::surface_properties(&shape, &mut native, false, false).unwrap();
    let area = body_properties(&shape).unwrap().area;
    // The native rule has no error bound; it agrees to its own fixed-order accuracy.
    assert!(
        (area.value - native.mass()).abs() < 1e-4 * area.value,
        "{area:?} {}",
        native.mass()
    );
}

#[test]
fn results_are_identical_across_threads() {
    let compute = || {
        let shapes = [
            box_solid(1.0, 2.0, 3.0),
            cone(2.0, 0.5, 3.0),
            torus(3.0, 1.0),
        ];
        shapes.map(|shape| body_properties(&shape).unwrap())
    };
    let reference = compute();
    let handles: Vec<_> = (0..4).map(|_| std::thread::spawn(compute)).collect();
    for handle in handles {
        assert_eq!(handle.join().unwrap(), reference);
    }
}
