//! Rust twins of `ProbePointDistance` (BVH_Distance) and `ProbePairDistance`
//! (BVH_PairDistance) over triangulation sets.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::tools::{
    point_box_square_distance_by_corners, point_triangle_square_distance,
};
use rustycad_tkmath::bvh::{
    BvhBaseTraverse, BvhDistance, BvhDistanceTraverse, BvhPairDistance, BvhPairDistanceTraverse,
    BvhPairSetTraverse, BvhPairTraverse, BvhScalar, BvhSetTraverse, BvhTraverse, BvhTriangulation,
    BvhVector, array,
};
use std::fmt::Write;

/// `bvh_query_triangle_distance`: point to triangle `index` of `mesh`.
fn triangle_distance<V: BvhVector>(point: &V, mesh: &BvhTriangulation<V>, index: i32) -> V::Scalar {
    let record = *array::value(&mesh.elements, index).unwrap();
    let vertex = |slot: i32| *array::value(&mesh.vertices, slot).unwrap();
    point_triangle_square_distance(
        point,
        &vertex(record.x),
        &vertex(record.y),
        &vertex(record.z),
    )
}

pub(super) struct PointDistance<'a, V: BvhVector> {
    pub state: BvhDistance<V::Scalar, V>,
    pub set: Option<&'a mut BvhTriangulation<V>>,
    pub log: String,
}
impl<V: BvhVector> BvhBaseTraverse<V::Scalar> for PointDistance<'_, V> {
    fn is_metric_better(&self, left: &V::Scalar, right: &V::Scalar) -> bool {
        self.state.is_metric_better(left, right)
    }
    fn reject_metric(&self, metric: &V::Scalar) -> bool {
        self.state.reject_metric(metric)
    }
    fn stop(&self) -> bool {
        self.state.stop()
    }
}
impl<V: BvhVector> BvhTraverse<V> for PointDistance<'_, V> {
    fn reject_node(&self, min: &V, max: &V, metric: &mut V::Scalar) -> bool {
        *metric = point_box_square_distance_by_corners(self.state.object(), min, max);
        self.state.reject_metric(metric)
    }
    fn accept(&mut self, index: i32, _metric: &V::Scalar) -> Result<bool, Failure> {
        let mesh = self.set.as_deref().unwrap();
        let distance = triangle_distance(self.state.object(), mesh, index);
        write!(self.log, "{index} ").unwrap();
        if distance < self.state.distance() {
            self.state.set_distance(distance);
            return Ok(true);
        }
        Ok(false)
    }
}
impl<V: BvhVector> BvhSetTraverse<V> for PointDistance<'_, V> {
    type Set = BvhTriangulation<V>;
    fn bvh_set_mut(&mut self) -> Option<&mut BvhTriangulation<V>> {
        self.set.as_deref_mut()
    }
}
impl<V: BvhVector> BvhDistanceTraverse<V> for PointDistance<'_, V> {
    type Object = V;
    fn distance_state(&self) -> &BvhDistance<V::Scalar, V> {
        &self.state
    }
    fn distance_state_mut(&mut self) -> &mut BvhDistance<V::Scalar, V> {
        &mut self.state
    }
}

pub(super) struct PairDistance<'a, V: BvhVector> {
    pub state: BvhPairDistance<V>,
    pub first: &'a mut BvhTriangulation<V>,
    pub second: &'a mut BvhTriangulation<V>,
    pub log: String,
}

/// `Candidate`: the nearest vertex of triangle `index` of `from` to triangle `other` of `to`.
fn candidate<V: BvhVector>(
    from: &BvhTriangulation<V>,
    index: i32,
    to: &BvhTriangulation<V>,
    other: i32,
) -> V::Scalar {
    let record = *array::value(&from.elements, index).unwrap();
    let mut best = V::Scalar::MAX;
    for slot in [record.x, record.y, record.z] {
        let vertex = *array::value(&from.vertices, slot).unwrap();
        let distance = triangle_distance(&vertex, to, other);
        if distance < best {
            best = distance;
        }
    }
    best
}

impl<V: BvhVector> BvhBaseTraverse<V::Scalar> for PairDistance<'_, V> {
    fn is_metric_better(&self, left: &V::Scalar, right: &V::Scalar) -> bool {
        self.state.is_metric_better(left, right)
    }
    fn reject_metric(&self, metric: &V::Scalar) -> bool {
        self.state.reject_metric(metric)
    }
    fn stop(&self) -> bool {
        self.state.stop()
    }
}
impl<V: BvhVector> BvhPairTraverse<V> for PairDistance<'_, V> {
    fn reject_node(&self, min1: &V, max1: &V, min2: &V, max2: &V, metric: &mut V::Scalar) -> bool {
        self.state.reject_node(min1, max1, min2, max2, metric)
    }
    fn accept(&mut self, first: i32, second: i32) -> Result<bool, Failure> {
        let mut distance = candidate(self.first, first, self.second, second);
        let other = candidate(self.second, second, self.first, first);
        if other < distance {
            distance = other;
        }
        write!(self.log, "{first} {second} ").unwrap();
        if distance < self.state.distance() {
            self.state.set_distance(distance);
            return Ok(true);
        }
        Ok(false)
    }
}
impl<V: BvhVector> BvhPairSetTraverse<V> for PairDistance<'_, V> {
    type Set = BvhTriangulation<V>;
    fn bvh_sets_mut(&mut self) -> Option<(&mut BvhTriangulation<V>, &mut BvhTriangulation<V>)> {
        Some((&mut *self.first, &mut *self.second))
    }
}
impl<V: BvhVector> BvhPairDistanceTraverse<V> for PairDistance<'_, V> {
    fn pair_distance_state(&self) -> &BvhPairDistance<V> {
        &self.state
    }
    fn pair_distance_state_mut(&mut self) -> &mut BvhPairDistance<V> {
        &mut self.state
    }
}
