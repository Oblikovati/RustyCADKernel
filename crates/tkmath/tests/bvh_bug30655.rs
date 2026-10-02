//! Rust translations of the three Bug30655 BVH regression declarations.
#![allow(non_snake_case)]

use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::tools::point_triangle_square_distance;
use rustycad_tkmath::bvh::{
    BvhBaseTraverse, BvhBox, BvhBoxSet, BvhDistanceField, BvhGeometry, BvhLinearBuilder, BvhObject,
    BvhPairDistance, BvhPairDistanceTraverse, BvhPairSetTraverse, BvhPairTraverse,
    BvhTriangulation, BvhVec3d, BvhVec4i, array,
};
use std::sync::Arc;

#[derive(Clone, Copy)]
struct Triangle([BvhVec3d; 3]);

type TriangleSet = BvhBoxSet<BvhVec3d, Triangle>;

fn append_triangle(mesh: &mut BvhTriangulation<BvhVec3d>, triangle: Triangle) {
    let first = mesh.vertices.len() as i32;
    for point in triangle.0 {
        array::append(&mut mesh.vertices, point);
    }
    array::append(
        &mut mesh.elements,
        BvhVec4i::new(first, first + 1, first + 2, 0),
    );
}

fn box_triangles(minimum: BvhVec3d, maximum: BvhVec3d) -> Vec<Triangle> {
    box_triangles_with_mask(minimum, maximum, 0)
}

fn box_points(minimum: BvhVec3d, maximum: BvhVec3d) -> [BvhVec3d; 8] {
    [
        BvhVec3d::new(minimum.x, minimum.y, minimum.z),
        BvhVec3d::new(maximum.x, minimum.y, minimum.z),
        BvhVec3d::new(maximum.x, maximum.y, minimum.z),
        BvhVec3d::new(minimum.x, maximum.y, minimum.z),
        BvhVec3d::new(minimum.x, minimum.y, maximum.z),
        BvhVec3d::new(maximum.x, minimum.y, maximum.z),
        BvhVec3d::new(maximum.x, maximum.y, maximum.z),
        BvhVec3d::new(minimum.x, maximum.y, maximum.z),
    ]
}

fn box_face_meshes(minimum: BvhVec3d, maximum: BvhVec3d, mask: usize) -> [[Triangle; 2]; 6] {
    let points = box_points(minimum, maximum);
    let faces = [
        [0, 1, 2, 0, 2, 3],
        [4, 5, 6, 4, 6, 7],
        [0, 1, 5, 0, 5, 4],
        [1, 2, 6, 1, 6, 5],
        [2, 3, 7, 2, 7, 6],
        [3, 0, 4, 3, 4, 7],
    ];
    std::array::from_fn(|side| face_triangles(points, faces[side], mask & (1 << side) != 0))
}

fn box_triangles_with_mask(minimum: BvhVec3d, maximum: BvhVec3d, mask: usize) -> Vec<Triangle> {
    box_face_meshes(minimum, maximum, mask)
        .into_iter()
        .flatten()
        .collect()
}

fn face_triangles(points: [BvhVec3d; 8], face: [usize; 6], reverse: bool) -> [Triangle; 2] {
    let mut triangles = [
        Triangle([points[face[0]], points[face[1]], points[face[2]]]),
        Triangle([points[face[3]], points[face[4]], points[face[5]]]),
    ];
    if reverse {
        for triangle in &mut triangles {
            triangle.0.swap(1, 2);
        }
    }
    triangles
}

fn box_geometry() -> BvhGeometry<BvhVec3d> {
    let mut geometry = BvhGeometry::default();
    // Raw triangle winding drives signed distances; reverse +X to preserve the
    // positive outside voxel observed in OCCT's BRepMesh fixture.
    let faces = box_face_meshes(
        BvhVec3d::new(0.0, 0.0, 0.0),
        BvhVec3d::new(10.0, 10.0, 10.0),
        8,
    );
    for face in faces {
        let mut mesh = BvhTriangulation::default();
        for triangle in face {
            append_triangle(&mut mesh, triangle);
        }
        mesh.mark_dirty();
        geometry.objects_mut().append(Some(Box::new(mesh)));
    }
    geometry.mark_dirty();
    geometry
}

fn triangle_box(triangle: Triangle) -> BvhBox<BvhVec3d> {
    let mut bounding = BvhBox::default();
    for point in triangle.0 {
        bounding.add(&point);
    }
    bounding
}

fn triangle_set(triangles: &[Triangle]) -> TriangleSet {
    let builder = Arc::new(BvhLinearBuilder::<BvhVec3d>::default());
    let mut set = TriangleSet::new(Some(builder));
    for triangle in triangles {
        set.add(*triangle, triangle_box(*triangle));
    }
    set.build().unwrap();
    set
}

struct MeshDistance<'a> {
    state: BvhPairDistance<BvhVec3d>,
    first: &'a mut TriangleSet,
    second: &'a mut TriangleSet,
}

impl BvhBaseTraverse<f64> for MeshDistance<'_> {
    fn is_metric_better(&self, left: &f64, right: &f64) -> bool {
        self.state.is_metric_better(left, right)
    }
    fn reject_metric(&self, metric: &f64) -> bool {
        self.state.reject_metric(metric)
    }
    fn stop(&self) -> bool {
        self.state.stop()
    }
}

impl BvhPairTraverse<BvhVec3d> for MeshDistance<'_> {
    fn reject_node(
        &self,
        min1: &BvhVec3d,
        max1: &BvhVec3d,
        min2: &BvhVec3d,
        max2: &BvhVec3d,
        metric: &mut f64,
    ) -> bool {
        self.state.reject_node(min1, max1, min2, max2, metric)
    }
    fn accept(&mut self, first: i32, second: i32) -> Result<bool, Failure> {
        let left = self.first.element(first)?.0;
        let right = self.second.element(second)?.0;
        let distance = triangle_triangle_square_distance(left, right);
        if distance < self.state.distance() {
            self.state.set_distance(distance);
            return Ok(true);
        }
        Ok(false)
    }
}

impl BvhPairSetTraverse<BvhVec3d> for MeshDistance<'_> {
    type Set = TriangleSet;
    fn bvh_sets_mut(&mut self) -> Option<(&mut Self::Set, &mut Self::Set)> {
        Some((&mut *self.first, &mut *self.second))
    }
}

impl BvhPairDistanceTraverse<BvhVec3d> for MeshDistance<'_> {
    fn pair_distance_state(&self) -> &BvhPairDistance<BvhVec3d> {
        &self.state
    }
    fn pair_distance_state_mut(&mut self) -> &mut BvhPairDistance<BvhVec3d> {
        &mut self.state
    }
}

fn triangle_triangle_square_distance(first: [BvhVec3d; 3], second: [BvhVec3d; 3]) -> f64 {
    let mut minimum = f64::MAX;
    for (from, to) in [(first, second), (second, first)] {
        for point in from {
            minimum = minimum.min(point_triangle_square_distance(
                &point, &to[0], &to[1], &to[2],
            ));
        }
        for edge in [[from[0], from[1]], [from[1], from[2]], [from[2], from[0]]] {
            minimum = minimum.min(edge_triangle_square_distance(edge, to));
        }
    }
    minimum
}

fn edge_triangle_square_distance(edge: [BvhVec3d; 2], triangle: [BvhVec3d; 3]) -> f64 {
    let direction = edge[1] - edge[0];
    let length = vector_length(direction);
    if length < 1.0e-7 {
        return f64::MAX;
    }
    let direction = direction / length;
    let delta = length / 100.0;
    let mut minimum = f64::MAX;
    for sample in 1..100 {
        let point = edge[0] + direction * (sample as f64 * delta);
        minimum = minimum.min(point_triangle_square_distance(
            &point,
            &triangle[0],
            &triangle[1],
            &triangle[2],
        ));
    }
    minimum
}

fn vector_length(vector: BvhVec3d) -> f64 {
    (vector.x * vector.x + vector.y * vector.y + vector.z * vector.z).sqrt()
}

fn run_distance(first_triangles: &[Triangle], second_triangles: &[Triangle]) -> (f64, f64) {
    let mut first = triangle_set(first_triangles);
    let mut second = triangle_set(second_triangles);
    let mut distance = MeshDistance {
        state: BvhPairDistance::default(),
        first: &mut first,
        second: &mut second,
    };
    let result = distance.compute_distance().unwrap();
    assert!(distance.state.is_done());
    (result.sqrt(), distance.state.distance().sqrt())
}

fn sphere_patch(center: BvhVec3d, direction: BvhVec3d, radius: f64) -> Triangle {
    let tangent = BvhVec3d::new(direction.y, -direction.x, 0.0);
    let tangent = tangent * (1.0 / vector_length(tangent));
    let bitangent = BvhVec3d::new(
        direction.y * tangent.z - direction.z * tangent.y,
        direction.z * tangent.x - direction.x * tangent.z,
        direction.x * tangent.y - direction.y * tangent.x,
    );
    let point = |axis: BvhVec3d| center + axis * radius;
    let theta: f64 = 0.004;
    let near = point(direction);
    let along_tangent = point(direction * theta.cos() + tangent * theta.sin());
    let along_bitangent = point(direction * theta.cos() + bitangent * theta.sin());
    Triangle([near, along_tangent, along_bitangent])
}

#[test]
fn BVH_DistanceFieldTest__Bug30655_BoxDistanceField() {
    let mut geometry = box_geometry();
    let mut field = BvhDistanceField::<BvhVec3d>::new(2, true).unwrap();
    assert!(field.build(&mut geometry).unwrap());
    assert_eq!(
        (
            field.dimension_x(),
            field.dimension_y(),
            field.dimension_z()
        ),
        (2, 2, 2)
    );
    assert_ne!(field.voxel_size().x, 0.0);
    assert_ne!(field.voxel_size().y, 0.0);
    assert_ne!(field.voxel_size().z, 0.0);
    assert!(field.packed_data().iter().all(|value| value.is_finite()));
    assert!(field.packed_data().iter().any(|value| *value < 0.0));
    assert!(field.packed_data().iter().any(|value| *value > 0.0));
}

#[test]
fn BVH_PairDistanceTest__Bug30655_BoxDistance() {
    let first = box_triangles(
        BvhVec3d::new(0.0, 0.0, 0.0),
        BvhVec3d::new(10.0, 10.0, 10.0),
    );
    let second = box_triangles(
        BvhVec3d::new(20.0, 0.0, 0.0),
        BvhVec3d::new(30.0, 10.0, 10.0),
    );
    let (computed, stored) = run_distance(&first, &second);
    assert!((computed - 10.0).abs() <= 1.0e-2);
    assert!((stored - 10.0).abs() <= 1.0e-2);
}

#[test]
fn BVH_PairDistanceTest__Bug30655_SphereDistance() {
    let direction = BvhVec3d::new(
        1.0 / 3.0_f64.sqrt(),
        1.0 / 3.0_f64.sqrt(),
        1.0 / 3.0_f64.sqrt(),
    );
    let first = [sphere_patch(BvhVec3d::new(0.0, 0.0, 0.0), direction, 100.0)];
    let second = [sphere_patch(
        BvhVec3d::new(150.0, 150.0, 150.0),
        direction * -1.0,
        110.0,
    )];
    let expected = (3.0_f64 * 150.0 * 150.0).sqrt() - 210.0;
    let (computed, stored) = run_distance(&first, &second);
    assert!((computed - expected).abs() <= 1.0e-2);
    assert!((stored - expected).abs() <= 1.0e-2);
}
