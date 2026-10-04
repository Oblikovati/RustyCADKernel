// `BVH_DistanceField::Build` uses `OSD_Parallel::For` over independent Z slices.
use super::{BvhFieldVector, nearest_point::direction_to_nearest_point};
use crate::bvh::{
    BvhBaseTraverse, BvhGeometry, BvhObject, BvhPrimitiveSetAccess, BvhTraverse, BvhTree,
    BvhTriangulation, array, distance_field::nearest_point::dot3,
    distance_field::square_distance::triangle_normal, scalar::BvhScalar,
};
use rustycad_tkernel::{
    ncollection::{LinearVector, Vec4},
    osd::Parallel,
    standard::Failure,
};
use std::{cmp::Ordering, sync::Mutex};

/// Immutable geometry state for parallel queries after every required BVH is ready.
pub(super) struct BvhDistanceGeometrySnapshot<V: BvhFieldVector> {
    tree: BvhTree<V>,
    objects: Vec<Option<BvhTriangleSnapshot<V>>>,
}

pub(super) struct DistanceFieldParallelContext<'a, V: BvhFieldVector> {
    dimensions: [i32; 3],
    corner_min: &'a V,
    voxel_size: &'a V,
    compute_sign: bool,
    geometry: &'a BvhDistanceGeometrySnapshot<V>,
}

impl<'a, V: BvhFieldVector> DistanceFieldParallelContext<'a, V> {
    pub(super) fn new(
        dimensions: [i32; 3],
        corner_min: &'a V,
        voxel_size: &'a V,
        compute_sign: bool,
        geometry: &'a BvhDistanceGeometrySnapshot<V>,
    ) -> Self {
        Self {
            dimensions,
            corner_min,
            voxel_size,
            compute_sign,
            geometry,
        }
    }
}

struct BvhTriangleSnapshot<V: BvhFieldVector> {
    tree: BvhTree<V>,
    vertices: LinearVector<V>,
    elements: LinearVector<Vec4<i32>>,
}

impl<V: BvhFieldVector> BvhDistanceGeometrySnapshot<V> {
    /// Captures clean trees without changing native lazy-build state.
    pub(super) fn try_new(geometry: &mut BvhGeometry<V>) -> Result<Option<Self>, Failure> {
        if geometry.is_dirty() {
            return Ok(None);
        }
        let tree = geometry.bvh()?.clone();
        let objects = capture_objects(geometry)?;
        Ok(objects.map(|objects| Self { tree, objects }))
    }
}

fn capture_objects<V: BvhFieldVector>(
    geometry: &mut BvhGeometry<V>,
) -> Result<Option<Vec<Option<BvhTriangleSnapshot<V>>>>, Failure> {
    let mut snapshots = Vec::with_capacity(geometry.objects().len());
    for object in geometry.objects_mut().iter_mut() {
        let Some(object) = object.as_deref_mut() else {
            return Ok(None);
        };
        let Some(triangulation) = object.triangulation_mut() else {
            snapshots.push(None);
            continue;
        };
        if triangulation.is_dirty() {
            return Ok(None);
        }
        snapshots.push(Some(capture_triangulation(triangulation)?));
    }
    Ok(Some(snapshots))
}

fn capture_triangulation<V: BvhFieldVector>(
    triangulation: &mut BvhTriangulation<V>,
) -> Result<BvhTriangleSnapshot<V>, Failure> {
    Ok(BvhTriangleSnapshot {
        tree: triangulation.bvh()?.clone(),
        vertices: triangulation.vertices.clone(),
        elements: triangulation.elements.clone(),
    })
}

/// Fills disjoint Z planes through the ported OCCT worker pool.
pub(super) fn parallel_slices<V: BvhFieldVector>(
    voxels: &mut [V::Scalar],
    context: &DistanceFieldParallelContext<'_, V>,
) -> Result<(), Failure> {
    let [dimension_x, dimension_y, dimension_z] = context.dimensions;
    let plane_size = (dimension_x * dimension_y) as usize;
    let active_size = plane_size * dimension_z as usize;
    let failures = Mutex::new(None);
    let planes = voxels[..active_size].chunks_mut(plane_size).enumerate();
    run_parallel_planes(planes, context, &failures)?;
    first_failure(failures)
}

fn run_parallel_planes<'a, V: BvhFieldVector>(
    planes: std::iter::Enumerate<std::slice::ChunksMut<'a, V::Scalar>>,
    context: &DistanceFieldParallelContext<'_, V>,
    failures: &Mutex<Option<Failure>>,
) -> Result<(), Failure> {
    Parallel::for_each(
        planes,
        |(slice, plane)| {
            fill_plane(plane, slice as i32, context, failures);
        },
        false,
        context.dimensions[2],
    )
}

fn first_failure(failures: Mutex<Option<Failure>>) -> Result<(), Failure> {
    failures
        .into_inner()
        .unwrap_or_else(|poison| poison.into_inner())
        .map_or(Ok(()), Err)
}

fn fill_plane<V: BvhFieldVector>(
    plane: &mut [V::Scalar],
    slice: i32,
    context: &DistanceFieldParallelContext<'_, V>,
    failures: &Mutex<Option<Failure>>,
) {
    for (offset, output) in plane.iter_mut().enumerate() {
        if failure_exists(failures) || fill_voxel(output, offset, slice, context, failures) {
            break;
        }
    }
}

fn failure_exists(failures: &Mutex<Option<Failure>>) -> bool {
    failures
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .is_some()
}

fn fill_voxel<V: BvhFieldVector>(
    output: &mut V::Scalar,
    offset: usize,
    slice: i32,
    context: &DistanceFieldParallelContext<'_, V>,
    failures: &Mutex<Option<Failure>>,
) -> bool {
    match voxel_value(offset, slice, context) {
        Ok(value) => *output = value,
        Err(failure) => record_failure(failures, failure),
    }
    failure_exists(failures)
}

fn voxel_value<V: BvhFieldVector>(
    offset: usize,
    slice: i32,
    context: &DistanceFieldParallelContext<'_, V>,
) -> Result<V::Scalar, Failure> {
    let dimension_x = context.dimensions[0];
    let x = offset as i32 % dimension_x;
    let y = offset as i32 / dimension_x;
    let point = voxel_center([x, y, slice], context.corner_min, context.voxel_size);
    let mut query = BvhGeometryDistanceQuery::new(point, context.geometry);
    select_geometry(&mut query)?;
    signed_distance(query.distance, query.is_outside, context.compute_sign)
}

fn voxel_center<V: BvhFieldVector>(voxel: [i32; 3], minimum: &V, size: &V) -> V {
    let mut center = V::default();
    for axis in 0..3 {
        let offset = V::Scalar::from_int(voxel[axis]) + V::Scalar::from_double(0.5);
        center[axis] = minimum[axis] + size[axis] * offset;
    }
    center
}

fn select_geometry<V: BvhFieldVector>(
    query: &mut BvhGeometryDistanceQuery<'_, V>,
) -> Result<(), Failure> {
    crate::bvh::select_elements(query, Some(&query.geometry.tree))?;
    Ok(())
}

fn signed_distance<T: BvhScalar>(
    distance: T,
    is_outside: bool,
    compute_sign: bool,
) -> Result<T, Failure> {
    let distance = T::from_double(distance.to_double().sqrt());
    if !compute_sign || is_outside {
        return Ok(distance);
    }
    Ok(-distance)
}

fn record_failure(failures: &Mutex<Option<Failure>>, failure: Failure) {
    let mut slot = failures.lock().unwrap_or_else(|poison| poison.into_inner());
    if slot.is_none() {
        *slot = Some(failure);
    }
}

struct BvhGeometryDistanceQuery<'a, V: BvhFieldVector> {
    point: V,
    distance: V::Scalar,
    is_outside: bool,
    geometry: &'a BvhDistanceGeometrySnapshot<V>,
}

impl<'a, V: BvhFieldVector> BvhGeometryDistanceQuery<'a, V> {
    fn new(point: V, geometry: &'a BvhDistanceGeometrySnapshot<V>) -> Self {
        Self {
            point,
            distance: V::Scalar::MAX,
            is_outside: true,
            geometry,
        }
    }
}

impl<V: BvhFieldVector> BvhBaseTraverse<V::Scalar> for BvhGeometryDistanceQuery<'_, V> {}

impl<V: BvhFieldVector> BvhTraverse<V> for BvhGeometryDistanceQuery<'_, V> {
    fn reject_node(&self, minimum: &V, maximum: &V, metric: &mut V::Scalar) -> bool {
        *metric = super::distance_to_box(&self.point, minimum, maximum);
        *metric > self.distance
    }
    fn accept(&mut self, index: i32, _metric: &V::Scalar) -> Result<bool, Failure> {
        let Some(object) = self
            .geometry
            .objects
            .get(index as usize)
            .ok_or_else(|| object_index_failure(index))?
        else {
            return Ok(false);
        };
        let (distance, is_outside) = distance_to_object(self.point, object)?;
        self.update(distance, is_outside)
    }
}

impl<V: BvhFieldVector> BvhGeometryDistanceQuery<'_, V> {
    fn update(&mut self, distance: V::Scalar, is_outside: bool) -> Result<bool, Failure> {
        // Native keeps the first tie and rejects NaN with this strict comparison.
        if distance.partial_cmp(&self.distance) != Some(Ordering::Less) {
            return Ok(false);
        }
        self.distance = distance;
        self.is_outside = is_outside;
        Ok(true)
    }
}

fn distance_to_object<V: BvhFieldVector>(
    point: V,
    object: &BvhTriangleSnapshot<V>,
) -> Result<(V::Scalar, bool), Failure> {
    let mut query = BvhTriangleDistanceQuery::new(point, object);
    crate::bvh::select_elements(&mut query, Some(&object.tree))?;
    Ok((query.distance, query.is_outside))
}

struct BvhTriangleDistanceQuery<'a, V: BvhFieldVector> {
    point: V,
    distance: V::Scalar,
    is_outside: bool,
    triangulation: &'a BvhTriangleSnapshot<V>,
}

impl<'a, V: BvhFieldVector> BvhTriangleDistanceQuery<'a, V> {
    fn new(point: V, triangulation: &'a BvhTriangleSnapshot<V>) -> Self {
        Self {
            point,
            distance: V::Scalar::MAX,
            is_outside: true,
            triangulation,
        }
    }
}

impl<V: BvhFieldVector> BvhBaseTraverse<V::Scalar> for BvhTriangleDistanceQuery<'_, V> {}

impl<V: BvhFieldVector> BvhTraverse<V> for BvhTriangleDistanceQuery<'_, V> {
    fn reject_node(&self, minimum: &V, maximum: &V, metric: &mut V::Scalar) -> bool {
        *metric = super::distance_to_box(&self.point, minimum, maximum);
        *metric > self.distance
    }
    fn accept(&mut self, index: i32, _metric: &V::Scalar) -> Result<bool, Failure> {
        let record = array::value(&self.triangulation.elements, index)?;
        let nodes = triangle_nodes(self.triangulation, record)?;
        let direction = direction_to_nearest_point(&self.point, &nodes[0], &nodes[1], &nodes[2]);
        let distance = dot3(&direction, &direction);
        if distance < self.distance {
            self.distance = distance;
            self.is_outside = dot3(&triangle_normal(&nodes), &direction) > V::Scalar::ZERO;
            return Ok(true);
        }
        Ok(false)
    }
}

fn triangle_nodes<V: BvhFieldVector>(
    triangulation: &BvhTriangleSnapshot<V>,
    record: &Vec4<i32>,
) -> Result<[V; 3], Failure> {
    Ok([
        *array::value(&triangulation.vertices, record.x)?,
        *array::value(&triangulation.vertices, record.y)?,
        *array::value(&triangulation.vertices, record.z)?,
    ])
}

fn object_index_failure(index: i32) -> Failure {
    Failure::out_of_range(&format!(
        "BVH_Geometry object index {index}: expected a captured object slot"
    ))
}
