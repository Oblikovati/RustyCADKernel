// SPDX-License-Identifier: GPL-2.0-only
//! Acceptance of `ray_pick` against closed-form hits (ADR 0078).
mod support;
use rustycad_kernel::pick::{PickCompleteness, PickHit, PickRay, PickResult, ray_pick};
use rustycad_kernel::query_error::{BodyQuery, BodyQueryFailure};
use rustycad_kernel::topology::{BodyTopology, FaceIndex};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use support::analytic::Surface;
use support::bodies::{
    Fixture, annulus_prism_fixture, box_fixture, cylinder_fixture, sphere_fixture, torus_fixture,
};

const CLOSE: f64 = 1e-12;

fn pick(body: &Shape, origin: [f64; 3], direction: [f64; 3]) -> PickResult {
    ray_pick(body, &PickRay::new(origin, direction).unwrap()).unwrap()
}

/// Asserts the hit distances against closed forms and returns the hits.
fn assert_distances(fixture: &Fixture, origin: [f64; 3], direction: [f64; 3], expected: &[f64]) {
    let result = pick(&fixture.body, origin, direction);
    let distances: Vec<f64> = result.hits.iter().map(|hit| hit.distance).collect();
    assert_eq!(result.nearest(), result.hits.first());
    assert_eq!(
        distances.len(),
        expected.len(),
        "{}: {distances:?}",
        fixture.name
    );
    for (found, wanted) in distances.iter().zip(expected) {
        assert!(
            (found - wanted).abs() <= CLOSE,
            "{}: {distances:?}",
            fixture.name
        );
    }
    assert_eq!(result.completeness, PickCompleteness::Analytic);
    result
        .hits
        .iter()
        .for_each(|hit| assert_exact_hit(fixture, hit, origin, direction));
}

/// The hit point is on the ray, on the face surface at (u, v), and on one of the
/// independent closed-form fixture surfaces.
fn assert_exact_hit(fixture: &Fixture, hit: &PickHit, origin: [f64; 3], direction: [f64; 3]) {
    let name = fixture.name;
    let length = gap(direction, [0.0; 3]);
    let on_ray = [0, 1, 2].map(|axis| origin[axis] + hit.distance * direction[axis] / length);
    assert!(
        gap(on_ray, hit.point) <= CLOSE,
        "{name}: {hit:?} off the ray"
    );
    let evaluated = face_point(&fixture.body, hit);
    assert!(
        gap(evaluated, hit.point) <= 1e-9,
        "{name}: {hit:?} uv mismatch"
    );
    let on_fixture = |surface: &Surface| surface.distance(hit.point) <= 1e-9;
    assert!(
        fixture.surfaces.iter().any(on_fixture),
        "{name}: {hit:?} off the fixture"
    );
}

/// The face surface of the hit evaluated at the hit parameters.
fn face_point(body: &Shape, hit: &PickHit) -> [f64; 3] {
    let topology = BodyTopology::enumerate(body).unwrap();
    let face = topology.face(hit.face).unwrap();
    let surface = BRepAdaptorSurface::with_face(&face, false).unwrap();
    let point = surface.value(hit.uv[0], hit.uv[1]).unwrap();
    [point.x(), point.y(), point.z()]
}

fn gap(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3)
        .map(|axis| (a[axis] - b[axis]).powi(2))
        .sum::<f64>()
        .sqrt()
}

#[test]
fn box_faces_are_hit_at_closed_form_distances() {
    let fixture = box_fixture();
    assert_distances(&fixture, [0.5, 1.0, 10.0], [0.0, 0.0, -1.0], &[6.0, 10.0]);
    assert_distances(&fixture, [-3.0, 1.5, 2.0], [1.0, 0.0, 0.0], &[3.0, 5.0]);
    assert_distances(&fixture, [1.0, 9.0, 1.0], [0.0, -2.0, 0.0], &[6.0, 9.0]);
}

#[test]
fn sphere_hits_follow_the_line_sphere_roots() {
    let fixture = sphere_fixture();
    let half_chord = (2.25_f64 - 0.25).sqrt();
    assert_distances(
        &fixture,
        [0.3, 0.4, -10.0],
        [0.0, 0.0, 1.0],
        &[10.0 - half_chord, 10.0 + half_chord],
    );
    // From inside, only the forward crossing counts.
    assert_distances(&fixture, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], &[1.5]);
}

#[test]
fn cylinder_side_hits_follow_the_line_circle_roots() {
    let fixture = cylinder_fixture();
    let half_chord = (1.0_f64 - 0.04).sqrt();
    assert_distances(
        &fixture,
        [-5.0, 0.2, 1.0],
        [1.0, 0.0, 0.0],
        &[5.0 - half_chord, 5.0 + half_chord],
    );
    // Through the seam (u = 0) the crossing is reported once.
    assert_distances(&fixture, [5.0, 0.0, 1.0], [-1.0, 0.0, 0.0], &[4.0, 6.0]);
}

#[test]
fn torus_and_annulus_hits_cross_every_wall() {
    let torus = torus_fixture();
    assert_distances(
        &torus,
        [-10.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        &[6.0, 8.0, 12.0, 14.0],
    );
    assert_distances(&torus, [3.0, 0.0, -10.0], [0.0, 0.0, 1.0], &[9.0, 11.0]);
    let annulus = annulus_prism_fixture();
    assert_distances(
        &annulus,
        [-10.0, 0.0, 0.75],
        [1.0, 0.0, 0.0],
        &[8.0, 9.0, 11.0, 12.0],
    );
    assert_distances(&annulus, [1.5, 0.0, -1.0], [0.0, 0.0, 1.0], &[1.0, 2.5]);
}

#[test]
fn misses_and_rays_pointing_away_have_no_hits() {
    let sphere = sphere_fixture();
    assert_distances(&sphere, [1.5 + 1e-6, 0.0, -10.0], [0.0, 0.0, 1.0], &[]);
    assert_distances(&sphere, [0.0, 0.0, 5.0], [0.0, 0.0, 1.0], &[]);
    let annulus = annulus_prism_fixture();
    // Down the hole of the annulus.
    assert_distances(&annulus, [0.0, 0.0, 5.0], [0.0, 0.0, -1.0], &[]);
}

#[test]
fn grazing_rays_touch_once_on_the_boundary_or_tangent_point() {
    let sphere = sphere_fixture();
    assert_distances(&sphere, [1.5, 0.0, -10.0], [0.0, 0.0, 1.0], &[10.0]);
    let cylinder = cylinder_fixture();
    assert_distances(&cylinder, [-5.0, 1.0, 1.0], [1.0, 0.0, 0.0], &[5.0]);
}

#[test]
fn a_ray_in_a_face_plane_is_coplanar_and_hits_only_the_crossed_edges() {
    let fixture = box_fixture();
    let result = pick(&fixture.body, [-1.0, 0.0, 2.0], [1.0, 0.0, 0.0]);
    assert_eq!(result.coplanar_faces.len(), 1);
    let coplanar = BodyTopology::enumerate(&fixture.body)
        .unwrap()
        .face(result.coplanar_faces[0]);
    let surface = BRepAdaptorSurface::with_face(&coplanar.unwrap(), false).unwrap();
    assert!(
        surface
            .plane()
            .unwrap()
            .distance(rustycad_tkmath::gp::Point3::new(5.0, 0.0, 1.0))
            < CLOSE
    );
    let boundary: Vec<bool> = result.hits.iter().map(|hit| hit.on_boundary).collect();
    assert_eq!(boundary, [true, true]);
    assert_distances(&fixture, [-1.0, 0.0, 2.0], [1.0, 0.0, 0.0], &[1.0, 3.0]);
}

#[test]
fn hits_tied_at_an_edge_or_corner_are_ordered_by_face_index() {
    let fixture = box_fixture();
    let result = pick(&fixture.body, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
    let corner: Vec<&PickHit> = result.hits.iter().take(3).collect();
    assert!(
        corner
            .iter()
            .all(|hit| hit.distance == corner[0].distance && hit.on_boundary)
    );
    let faces: Vec<FaceIndex> = corner.iter().map(|hit| hit.face).collect();
    let mut sorted = faces.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(faces, sorted, "three distinct faces in index order");
    assert_eq!(
        pick(&fixture.body, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]),
        result
    );
}

#[test]
fn pick_keys_match_the_presentation_face_keys() {
    use rustycad_kernel::presentation::{DisplayOptions, tessellate};
    let fixture = box_fixture();
    let mesh = tessellate(&fixture.body, &DisplayOptions::new(0.1, 0.5).unwrap()).unwrap();
    let hit = pick(&fixture.body, [0.5, 1.0, 10.0], [0.0, 0.0, -1.0]).hits[0];
    let face = &mesh.faces[hit.face.position()];
    assert!(face.positions.iter().all(|position| position[2] == 4.0));
}

#[test]
fn invalid_rays_and_null_bodies_are_refused() {
    for (origin, direction) in [
        ([0.0; 3], [0.0; 3]),
        ([f64::NAN, 0.0, 0.0], [1.0, 0.0, 0.0]),
    ] {
        let error = PickRay::new(origin, direction).unwrap_err();
        assert_eq!(
            (error.operation, error.class),
            (BodyQuery::RayPick, BodyQueryFailure::InvalidRay)
        );
    }
    let ray = PickRay::new([0.0; 3], [1.0, 0.0, 0.0]).unwrap();
    let error = ray_pick(&Shape::default(), &ray).unwrap_err();
    assert_eq!(
        (error.operation, error.class),
        (BodyQuery::RayPick, BodyQueryFailure::NullBody)
    );
}

#[test]
fn eight_threads_pick_the_same_hits() {
    let fixture = torus_fixture();
    let expected = pick(&fixture.body, [-10.0, 0.5, 0.1], [1.0, 0.0, 0.0]);
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| pick(&fixture.body, [-10.0, 0.5, 0.1], [1.0, 0.0, 0.0])))
            .collect();
        workers
            .into_iter()
            .for_each(|worker| assert_eq!(worker.join().unwrap(), expected));
    });
}
