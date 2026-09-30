// Rust translation of FoundationClasses/TKMath/BVH/BVH_DistanceField.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH::SquareDistanceToPoint` and its triangulation and geometry selectors.
use super::BvhFieldVector;
use super::nearest_point::{direction_to_nearest_point, distance_to_box, dot3};
use crate::bvh::array;
use crate::bvh::distance::{BvhDistance, BvhDistanceTraverse};
use crate::bvh::geometry::BvhGeometry;
use crate::bvh::object::BvhObject;
use crate::bvh::scalar::BvhScalar;
use crate::bvh::traverse::{BvhBaseTraverse, BvhSetTraverse, BvhTraverse};
use crate::bvh::triangulation::BvhTriangulation;
use rustycad_tkernel::standard::Failure;

/// `BVH::SquareDistanceToPoint`: distance data plus the inside/outside flag
/// (initially outside). Nodes are rejected by `DistanceToBox`, and the descent
/// never stops early. Example: `BvhSquareDistanceToPoint::<BvhVec3d>::default()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BvhSquareDistanceToPoint<V: BvhFieldVector> {
    /// The `BVH_Distance` base data; the object is the query point.
    pub distance: BvhDistance<V::Scalar, V>,
    is_outside: bool,
}
impl<V: BvhFieldVector> Default for BvhSquareDistanceToPoint<V> {
    fn default() -> Self {
        Self {
            distance: BvhDistance::default(),
            is_outside: true,
        }
    }
}
impl<V: BvhFieldVector> BvhSquareDistanceToPoint<V> {
    /// Whether the point lies on the outer side of the nearest triangle.
    /// Example: `state.is_outside()`.
    pub fn is_outside(&self) -> bool {
        self.is_outside
    }
    /// `RejectNode`: the square distance to the node box against the current one.
    /// Example: `state.reject_node(&min, &max, &mut metric)`.
    pub fn reject_node(&self, corner_min: &V, corner_max: &V, metric: &mut V::Scalar) -> bool {
        *metric = distance_to_box(self.distance.object(), corner_min, corner_max);
        *metric > self.distance.distance()
    }
}

/// Forwards the `BVH_Distance` rules; `Stop` is redefined to never stop.
macro_rules! square_distance_rules {
    ($selector:ident) => {
        impl<V: BvhFieldVector> BvhBaseTraverse<V::Scalar> for $selector<'_, V> {
            fn is_metric_better(&self, left: &V::Scalar, right: &V::Scalar) -> bool {
                self.base.distance.is_metric_better(left, right)
            }
            fn reject_metric(&self, metric: &V::Scalar) -> bool {
                self.base.distance.reject_metric(metric)
            }
        }
        impl<V: BvhFieldVector> BvhDistanceTraverse<V> for $selector<'_, V> {
            type Object = V;
            fn distance_state(&self) -> &BvhDistance<V::Scalar, V> {
                &self.base.distance
            }
            fn distance_state_mut(&mut self) -> &mut BvhDistance<V::Scalar, V> {
                &mut self.base.distance
            }
        }
    };
}

/// `BVH::PointTriangulationSquareDistance`: square distance from a point to a
/// triangulation, with the side given by the nearest triangle normal.
/// Example: `let mut tool = BvhPointTriangulationSquareDistance::new(point, Some(&mut mesh));`.
#[derive(Debug)]
pub struct BvhPointTriangulationSquareDistance<'a, V: BvhFieldVector> {
    /// The `SquareDistanceToPoint` base data.
    pub base: BvhSquareDistanceToPoint<V>,
    set: Option<&'a mut BvhTriangulation<V>>,
}
impl<'a, V: BvhFieldVector> BvhPointTriangulationSquareDistance<'a, V> {
    /// Sets the query point and the triangulation (`SetObject`, `SetBVHSet`).
    pub fn new(point: V, set: Option<&'a mut BvhTriangulation<V>>) -> Self {
        let mut base = BvhSquareDistanceToPoint::default();
        base.distance.set_object(point);
        Self { base, set }
    }
}
impl<V: BvhFieldVector> BvhTraverse<V> for BvhPointTriangulationSquareDistance<'_, V> {
    fn reject_node(&self, corner_min: &V, corner_max: &V, metric: &mut V::Scalar) -> bool {
        self.base.reject_node(corner_min, corner_max, metric)
    }
    /// Keeps a strictly nearer triangle; the point is outside when the normal
    /// `(v1 - v0) x (v2 - v0)` points towards it.
    fn accept(&mut self, index: i32, _metric: &V::Scalar) -> Result<bool, Failure> {
        let set = self.set.as_deref().ok_or_else(missing_set)?;
        let record = *array::value(&set.elements, index)?;
        let vertex = |slot: i32| array::value(&set.vertices, slot).copied();
        let nodes = [vertex(record.x)?, vertex(record.y)?, vertex(record.z)?];
        let point = *self.base.distance.object();
        let direction = direction_to_nearest_point(&point, &nodes[0], &nodes[1], &nodes[2]);
        let distance = dot3(&direction, &direction);
        // `<` rather than a negated `>=`: a NaN distance is never kept.
        if distance < self.base.distance.distance() {
            self.base.distance.set_distance(distance);
            let normal = triangle_normal(&nodes);
            self.base.is_outside = dot3(&normal, &direction) > V::Scalar::ZERO;
            return Ok(true);
        }
        Ok(false)
    }
}
impl<V: BvhFieldVector> BvhSetTraverse<V> for BvhPointTriangulationSquareDistance<'_, V> {
    type Set = BvhTriangulation<V>;
    fn bvh_set_mut(&mut self) -> Option<&mut BvhTriangulation<V>> {
        self.set.as_deref_mut()
    }
}
square_distance_rules!(BvhPointTriangulationSquareDistance);

/// Cross product of the edges `v1 - v0` and `v2 - v0`; other components stay zero.
fn triangle_normal<V: BvhFieldVector>(nodes: &[V; 3]) -> V {
    let (first, second) = (nodes[1] - nodes[0], nodes[2] - nodes[0]);
    let mut normal = V::default();
    normal[0] = first[1] * second[2] - first[2] * second[1];
    normal[1] = first[2] * second[0] - first[0] * second[2];
    normal[2] = first[0] * second[1] - first[1] * second[0];
    normal
}

fn missing_set() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BVH distance selector has no BVH set, expected SetBVHSet before Accept",
        "",
    )
}

/// `BVH::SquareDistanceToObject`: square distance from a point to an object
/// that is a triangulation, updating `is_outside` only then.
///
/// Any other object (or a null handle) fails the upstream
/// `Standard_ASSERT_RETURN` and yields `numeric_limits<T>::max()`. The
/// triangulation tree is built first when the triangulation is dirty.
/// Example: `square_distance_to_object(Some(&mut mesh), &point, &mut outside)?`.
pub fn square_distance_to_object<V: BvhFieldVector>(
    object: Option<&mut (dyn BvhObject<V> + Send + 'static)>,
    point: &V,
    is_outside: &mut bool,
) -> Result<V::Scalar, Failure> {
    let Some(triangulation) = object.and_then(|object| object.triangulation_mut()) else {
        return Ok(V::Scalar::MAX);
    };
    let mut tool = BvhPointTriangulationSquareDistance::new(*point, Some(triangulation));
    tool.compute_distance()?;
    *is_outside = tool.base.is_outside();
    Ok(tool.base.distance.distance())
}

/// `BVH::PointGeometrySquareDistance`: square distance from a point to the
/// nearest object of a geometry. Example: `BvhPointGeometrySquareDistance::new(point, Some(&mut g))`.
#[derive(Debug)]
pub struct BvhPointGeometrySquareDistance<'a, V: BvhFieldVector> {
    /// The `SquareDistanceToPoint` base data.
    pub base: BvhSquareDistanceToPoint<V>,
    set: Option<&'a mut BvhGeometry<V>>,
}
impl<'a, V: BvhFieldVector> BvhPointGeometrySquareDistance<'a, V> {
    /// Sets the query point and the geometry (`SetObject`, `SetBVHSet`).
    pub fn new(point: V, set: Option<&'a mut BvhGeometry<V>>) -> Self {
        let mut base = BvhSquareDistanceToPoint::default();
        base.distance.set_object(point);
        Self { base, set }
    }
}
impl<V: BvhFieldVector> BvhTraverse<V> for BvhPointGeometrySquareDistance<'_, V> {
    fn reject_node(&self, corner_min: &V, corner_max: &V, metric: &mut V::Scalar) -> bool {
        self.base.reject_node(corner_min, corner_max, metric)
    }
    /// Keeps a strictly nearer object together with its side.
    fn accept(&mut self, index: i32, _metric: &V::Scalar) -> Result<bool, Failure> {
        let point = *self.base.distance.object();
        let geometry = self.set.as_deref_mut().ok_or_else(missing_set)?;
        let handle = geometry
            .objects_mut()
            .change_value(index)
            .map_err(|error| {
                Failure::out_of_range(&format!("BVH_Geometry object index {index}: {error:?}"))
            })?;
        let mut is_outside = true;
        let distance = square_distance_to_object(handle.as_deref_mut(), &point, &mut is_outside)?;
        if distance < self.base.distance.distance() {
            self.base.distance.set_distance(distance);
            self.base.is_outside = is_outside;
            return Ok(true);
        }
        Ok(false)
    }
}
impl<V: BvhFieldVector> BvhSetTraverse<V> for BvhPointGeometrySquareDistance<'_, V> {
    type Set = BvhGeometry<V>;
    fn bvh_set_mut(&mut self) -> Option<&mut BvhGeometry<V>> {
        self.set.as_deref_mut()
    }
}
square_distance_rules!(BvhPointGeometrySquareDistance);

/// `BVH::SquareDistanceToGeomerty`: square distance from a point to a geometry,
/// building the top-level tree when dirty.
/// Example: `square_distance_to_geometry(&mut geometry, &point, &mut outside)?`.
pub fn square_distance_to_geometry<V: BvhFieldVector>(
    geometry: &mut BvhGeometry<V>,
    point: &V,
    is_outside: &mut bool,
) -> Result<V::Scalar, Failure> {
    let mut tool = BvhPointGeometrySquareDistance::new(*point, Some(geometry));
    tool.compute_distance()?;
    *is_outside = tool.base.is_outside();
    Ok(tool.base.distance.distance())
}
