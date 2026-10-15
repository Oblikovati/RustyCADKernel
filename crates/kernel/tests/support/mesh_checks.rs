// SPDX-License-Identifier: GPL-2.0-only
//! Independent checks of a presentation mesh against closed forms and invariants.
use super::analytic::{Surface, cross, dot, norm, sub};
use rustycad_kernel::presentation::{FaceTriangles, PresentationMesh};
use std::collections::BTreeMap;

/// Mesh vertices are surface points; allow the modelling tolerance of the fixtures.
pub const ON_SURFACE: f64 = 1e-7;

/// The unique fixture surface every vertex of `face` lies on with a matching
/// outward normal; normals separate a cap from the cylinder through its rim.
pub fn matched_surface(face: &FaceTriangles, surfaces: &[Surface]) -> Surface {
    let matches: Vec<Surface> = surfaces
        .iter()
        .copied()
        .filter(|surface| face_lies_on(face, surface))
        .collect();
    assert_eq!(matches.len(), 1, "face {:?} matched {matches:?}", face.face);
    matches[0]
}

fn face_lies_on(face: &FaceTriangles, surface: &Surface) -> bool {
    face.positions
        .iter()
        .zip(&face.normals)
        .all(|(position, normal)| {
            surface.distance(*position) <= ON_SURFACE
                && dot(*normal, surface.outward_normal(*position)) >= 0.99
        })
}

/// The largest closed-form distance from a triangle centroid to `surface`.
pub fn max_centroid_deviation(face: &FaceTriangles, surface: &Surface) -> f64 {
    face.triangles
        .iter()
        .map(|triangle| surface.distance(centroid(face, triangle)))
        .fold(0.0, f64::max)
}

fn centroid(face: &FaceTriangles, triangle: &[u32; 3]) -> [f64; 3] {
    let [a, b, c] = triangle.map(|index| face.positions[index as usize]);
    [0, 1, 2].map(|axis| (a[axis] + b[axis] + c[axis]) / 3.0)
}

/// Asserts that the welded triangles close up: each directed edge is used once
/// and its opposite once, so the mesh is closed, manifold and consistently wound.
pub fn assert_watertight(mesh: &PresentationMesh, name: &str) {
    let mut uses: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for [a, b, c] in welded_triangles(mesh) {
        for edge in [(a, b), (b, c), (c, a)] {
            *uses.entry(edge).or_default() += 1;
        }
    }
    for (&(a, b), &count) in &uses {
        let opposite = uses.get(&(b, a)).copied().unwrap_or(0);
        assert_eq!((count, opposite), (1, 1), "{name}: welded edge {a}-{b}");
    }
}

/// Triangles over vertices welded by exact coordinates; triangles that collapse
/// onto a repeated welded vertex (pole fans) carry no area and are dropped.
fn welded_triangles(mesh: &PresentationMesh) -> Vec<[u32; 3]> {
    let mut ids: BTreeMap<[u64; 3], u32> = BTreeMap::new();
    let mut triangles = Vec::new();
    for face in &mesh.faces {
        for triangle in &face.triangles {
            let [a, b, c] = triangle.map(|index| {
                let key = face.positions[index as usize].map(f64::to_bits);
                let next = ids.len() as u32;
                *ids.entry(key).or_insert(next)
            });
            if a != b && b != c && c != a {
                triangles.push([a, b, c]);
            }
        }
    }
    triangles
}

/// The total triangle area.
pub fn mesh_area(mesh: &PresentationMesh) -> f64 {
    triangles(mesh)
        .map(|[a, b, c]| 0.5 * norm(cross(sub(b, a), sub(c, a))))
        .sum()
}

/// The enclosed volume by the divergence theorem; positive for outward winding.
pub fn signed_volume(mesh: &PresentationMesh) -> f64 {
    triangles(mesh)
        .map(|[a, b, c]| dot(a, cross(b, c)) / 6.0)
        .sum()
}

/// Each triangle's geometric normal agrees with its vertex normals.
pub fn assert_winding_matches_normals(mesh: &PresentationMesh, name: &str) {
    for face in &mesh.faces {
        for triangle in &face.triangles {
            let [a, b, c] = triangle.map(|index| face.positions[index as usize]);
            let geometric = cross(sub(b, a), sub(c, a));
            let normal = face.normals[triangle[0] as usize];
            let mean = triangle.map(|index| face.normals[index as usize]);
            let summed = [0, 1, 2].map(|axis| mean[0][axis] + mean[1][axis] + mean[2][axis]);
            let agrees = dot(geometric, summed) > 0.0 || norm(geometric) < 1e-12;
            assert!(
                agrees,
                "{name} face {:?}: triangle {triangle:?} normal {normal:?}",
                face.face
            );
        }
    }
}

fn triangles(mesh: &PresentationMesh) -> impl Iterator<Item = [[f64; 3]; 3]> + '_ {
    mesh.faces.iter().flat_map(|face| {
        face.triangles
            .iter()
            .map(|triangle| triangle.map(|index| face.positions[index as usize]))
    })
}
