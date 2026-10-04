//! Rust translations of the three Bug30655 BVH regression declarations.
#![allow(non_snake_case)]

use rustycad_tkbrep::{
    brep::BRepTool,
    topexp,
    topods::{self, Shape},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bvh::tools::point_triangle_square_distance;
use rustycad_tkmath::bvh::{
    BvhBaseTraverse, BvhBox, BvhBoxSet, BvhDistanceField, BvhGeometry, BvhLinearBuilder, BvhObject,
    BvhPairDistance, BvhPairDistanceTraverse, BvhPairSetTraverse, BvhPairTraverse,
    BvhTriangulation, BvhVec3d, BvhVec4i, array,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkmesh::{brep_mesh::BRepMeshIncrementalMesh, imesh_tools::MeshParameters};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeSphere};
use std::sync::Arc;

#[derive(Clone, Copy)]
struct Triangle([BvhVec3d; 3]);

type TriangleSet = BvhBoxSet<BvhVec3d, Triangle>;

fn append_triangle(mesh: &mut BvhTriangulation<BvhVec3d>, triangle: [BvhVec3d; 3]) {
    let first = mesh.vertices.len() as i32;
    for point in triangle {
        array::append(&mut mesh.vertices, point);
    }
    array::append(
        &mut mesh.elements,
        BvhVec4i::new(first, first + 1, first + 2, 0),
    );
}

fn make_box(
    point: Point3,
    sizes: [f64; 3],
    deflection: f64,
) -> Result<Shape, rustycad_tkernel::standard::Failure> {
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
    let mut maker = BRepPrimAPIMakeBox::at_point(point, sizes[0], sizes[1], sizes[2])?;
    let shape = maker.shape()?.clone();
    mesh_shape(&shape, deflection)?;
    Ok(shape)
}

fn make_sphere(
    center: Point3,
    radius: f64,
    deflection: f64,
) -> Result<Shape, rustycad_tkernel::standard::Failure> {
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
    let mut maker = BRepPrimAPIMakeSphere::at_center(center, radius)?;
    let shape = maker.shape()?.clone();
    mesh_shape(&shape, deflection)?;
    Ok(shape)
}

fn mesh_shape(shape: &Shape, deflection: f64) -> Result<(), rustycad_tkernel::standard::Failure> {
    let parameters = MeshParameters {
        deflection,
        ..MeshParameters::default()
    };
    BRepMeshIncrementalMesh::with_parameters(shape, &parameters, &ProgressRange::default())?;
    Ok(())
}

fn shape_triangles(shape: &Shape) -> Result<Vec<Triangle>, rustycad_tkernel::standard::Failure> {
    // Native makeTriangleBoxSet traverses TopExp's indexed face map, then each face's raw triangles.
    let faces = face_map(shape)?;
    let mut triangles = Vec::new();
    for index in 1..=faces.extent() {
        let face = topods::face(faces.find_key(index)?)?;
        triangles.extend(face_triangles_from_mesh(&face)?.into_iter().map(Triangle));
    }
    Ok(triangles)
}

fn face_map(shape: &Shape) -> Result<IndexedMapOfShape, rustycad_tkernel::standard::Failure> {
    let mut faces = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, ShapeEnum::Face, &mut faces)?;
    Ok(faces)
}

fn face_triangles_from_mesh(
    face: &topods::Face,
) -> Result<Vec<[BvhVec3d; 3]>, rustycad_tkernel::standard::Failure> {
    let (handle, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
    let triangulation = handle.get().ok_or_else(|| {
        rustycad_tkernel::standard::Failure::new(
            "Bug30655 face triangulation is null; expected a meshed face",
        )
    })?;
    // OCCT transforms nodes by the location returned from Triangulation; triangle winding stays raw.
    let transform = location.transformation();
    let mut triangles = Vec::new();
    for index in 1..=triangulation.nb_triangles() {
        let (a, b, c) = triangulation.triangle(index)?.get();
        triangles.push([
            mesh_node(triangulation, a, transform)?,
            mesh_node(triangulation, b, transform)?,
            mesh_node(triangulation, c, transform)?,
        ]);
    }
    Ok(triangles)
}

fn mesh_node(
    triangulation: &rustycad_tkmath::poly::Triangulation,
    index: i32,
    transform: rustycad_tkmath::gp::Transform3,
) -> Result<BvhVec3d, rustycad_tkernel::standard::Failure> {
    let point = triangulation.node(index)?.transformed(transform);
    Ok(BvhVec3d::new(point.x(), point.y(), point.z()))
}

fn box_geometry(
    shape: &Shape,
) -> Result<BvhGeometry<BvhVec3d>, rustycad_tkernel::standard::Failure> {
    let mut geometry = BvhGeometry::default();
    for face in face_map(shape)?.iter() {
        let mut mesh = BvhTriangulation::default();
        for triangle in face_triangles_from_mesh(&topods::face(face)?)? {
            append_triangle(&mut mesh, triangle);
        }
        mesh.mark_dirty();
        geometry.objects_mut().append(Some(Box::new(mesh)));
    }
    geometry.mark_dirty();
    Ok(geometry)
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

#[test]
fn BVH_DistanceFieldTest__Bug30655_BoxDistanceField() {
    let shape = make_box(Point3::new(0.0, 0.0, 0.0), [10.0; 3], 0.1).unwrap();
    let mut geometry = box_geometry(&shape).unwrap();
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
    let first_shape = make_box(Point3::new(0.0, 0.0, 0.0), [10.0; 3], 0.1).unwrap();
    let second_shape = make_box(Point3::new(20.0, 0.0, 0.0), [10.0; 3], 0.1).unwrap();
    let first = shape_triangles(&first_shape).unwrap();
    let second = shape_triangles(&second_shape).unwrap();
    let (computed, stored) = run_distance(&first, &second);
    assert!((computed - 10.0).abs() <= 1.0e-2);
    assert!((stored - 10.0).abs() <= 1.0e-2);
}

#[test]
fn BVH_PairDistanceTest__Bug30655_SphereDistance() {
    let first_shape = make_sphere(Point3::new(0.0, 0.0, 0.0), 100.0, 0.01).unwrap();
    let second_shape = make_sphere(Point3::new(150.0, 150.0, 150.0), 110.0, 0.01).unwrap();
    let first = shape_triangles(&first_shape).unwrap();
    let second = shape_triangles(&second_shape).unwrap();
    let expected = (3.0_f64 * 150.0 * 150.0).sqrt() - 210.0;
    let (computed, stored) = run_distance(&first, &second);
    assert!(
        (computed - expected).abs() <= 1.0e-2,
        "computed {computed}, expected {expected}"
    );
    assert!(
        (stored - expected).abs() <= 1.0e-2,
        "stored {stored}, expected {expected}"
    );
}
