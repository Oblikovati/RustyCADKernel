// SPDX-License-Identifier: GPL-2.0-only
//! Acceptance of `tessellate` against closed forms and invariants (ADR 0077).
mod support;
use rustycad_kernel::presentation::{DisplayOptions, PresentationMesh, tessellate};
use std::collections::BTreeSet;
use support::bodies::{Fixture, all_fixtures, sphere_fixture, torus_fixture};
use support::mesh_checks::{
    assert_watertight, assert_winding_matches_normals, matched_surface, max_centroid_deviation,
    mesh_area, signed_volume,
};

fn mesh_of(fixture: &Fixture, linear: f64) -> PresentationMesh {
    let options = DisplayOptions::new(linear, 0.5).unwrap();
    tessellate(&fixture.body, &options).unwrap()
}

#[test]
fn every_closed_fixture_is_watertight_and_outward() {
    for fixture in all_fixtures() {
        let mesh = mesh_of(&fixture, 0.01);
        assert_watertight(&mesh, fixture.name);
        assert_winding_matches_normals(&mesh, fixture.name);
        assert!(
            signed_volume(&mesh) > 0.0,
            "{}: inward winding",
            fixture.name
        );
    }
}

#[test]
fn every_triangle_stays_within_the_requested_deflection_of_its_face() {
    for linear in [0.1, 0.01] {
        for fixture in all_fixtures() {
            let mesh = mesh_of(&fixture, linear);
            for face in &mesh.faces {
                let surface = matched_surface(face, &fixture.surfaces);
                let deviation = max_centroid_deviation(face, &surface);
                assert!(
                    deviation <= linear,
                    "{} face {:?}: {deviation} > {linear}",
                    fixture.name,
                    face.face
                );
            }
        }
    }
}

#[test]
fn faces_cover_each_fixture_surface_once() {
    for fixture in all_fixtures() {
        let mesh = mesh_of(&fixture, 0.05);
        assert_eq!(mesh.faces.len(), fixture.surfaces.len(), "{}", fixture.name);
        let indices: Vec<u32> = mesh.faces.iter().map(|face| face.face.0).collect();
        assert_eq!(indices, (0..indices.len() as u32).collect::<Vec<_>>());
    }
}

/// The relative area error of the mesh at deflection `linear`, after checking area
/// and volume against the chord bound.
fn bounded_area_error(fixture: &Fixture, linear: f64) -> f64 {
    let mesh = mesh_of(fixture, linear);
    let area = (mesh_area(&mesh) - fixture.area).abs() / fixture.area;
    let volume = (signed_volume(&mesh) - fixture.volume).abs() / fixture.volume;
    // A chord sag d on radius R removes at most a relative 2 d / R of area or volume.
    let bound = 2.0 * linear / fixture.min_radius;
    let name = fixture.name;
    assert!(area <= bound, "{name} d={linear}: area {area} > {bound}");
    assert!(volume <= 1.5 * bound, "{name} d={linear}: volume {volume}");
    area
}

#[test]
fn mesh_area_and_volume_converge_to_closed_forms() {
    for fixture in all_fixtures() {
        let area_errors: Vec<f64> = [0.1, 0.01, 0.001]
            .map(|linear| bounded_area_error(&fixture, linear))
            .to_vec();
        let flat = fixture.min_radius.is_infinite();
        let converged = flat || area_errors[2] < area_errors[0];
        assert!(
            converged,
            "{}: no convergence {area_errors:?}",
            fixture.name
        );
    }
}

#[test]
fn edge_polylines_lie_on_the_face_meshes() {
    for fixture in all_fixtures() {
        let mesh = mesh_of(&fixture, 0.02);
        let nodes: BTreeSet<[u64; 3]> = mesh
            .faces
            .iter()
            .flat_map(|face| face.positions.iter().map(|p| p.map(f64::to_bits)))
            .collect();
        for edge in &mesh.edges {
            let on_mesh = edge
                .points
                .iter()
                .all(|p| nodes.contains(&p.map(f64::to_bits)));
            assert!(
                on_mesh && edge.points.len() >= 2,
                "{} edge {:?}",
                fixture.name,
                edge.edge
            );
        }
    }
}

#[test]
fn degenerated_edges_are_listed_without_polylines() {
    let sphere = mesh_of(&sphere_fixture(), 0.05);
    assert_eq!(sphere.degenerated_edges.len(), 2);
    assert_eq!(sphere.edges.len() + sphere.degenerated_edges.len(), 3);
    let torus = mesh_of(&torus_fixture(), 0.05);
    assert!(torus.degenerated_edges.is_empty());
    assert_eq!((torus.edges.len(), torus.vertices.len()), (2, 1));
}

#[test]
fn effective_settings_report_the_completed_parameters() {
    let mesh = mesh_of(&sphere_fixture(), 0.05);
    assert_eq!(mesh.requested.linear_deflection(), 0.05);
    assert_eq!(mesh.effective.linear_deflection, 0.05);
    assert_eq!(mesh.effective.angular_deflection, 0.5);
    assert_eq!(mesh.effective.interior_linear_deflection, 0.05);
    assert_eq!(mesh.effective.interior_angular_deflection, 1.0);
    assert!(mesh.effective.min_size > 0.0 && mesh.effective.min_size < 0.05);
}
