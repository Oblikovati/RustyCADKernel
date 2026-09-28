//! Rust twins of the probe selectors: logged single and pair traversals.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::tools::{
    box_box_square_distance_by_corners, point_box_square_distance_by_corners,
};
use rustycad_tkmath::bvh::{
    BvhBaseTraverse, BvhBox, BvhPairTraverse, BvhScalar, BvhTraverse, BvhVector,
};
use std::cell::RefCell;
use std::fmt::Write;

fn bits<T: BvhScalar>(value: T) -> u64 {
    value.to_double().to_bits()
}

/// `ProbeLogSelector`: closer boxes first, pruned beyond `max_sq`, whole
/// branches accepted within `accept_sq`, stopped after `stop_count` accepts.
pub(super) struct LogSelector<V: BvhVector> {
    pub point: V,
    pub max_sq: V::Scalar,
    pub accept_sq: V::Scalar,
    pub stop_count: i32,
    pub count: i32,
    pub log: RefCell<String>,
}
impl<V: BvhVector> BvhBaseTraverse<V::Scalar> for LogSelector<V> {
    fn is_metric_better(&self, left: &V::Scalar, right: &V::Scalar) -> bool {
        left < right
    }
    fn reject_metric(&self, metric: &V::Scalar) -> bool {
        *metric > self.max_sq
    }
    fn stop(&self) -> bool {
        self.stop_count > 0 && self.count >= self.stop_count
    }
}
impl<V: BvhVector> BvhTraverse<V> for LogSelector<V> {
    fn accept_metric(&self, metric: &V::Scalar) -> bool {
        *metric <= self.accept_sq
    }
    fn reject_node(&self, min: &V, max: &V, metric: &mut V::Scalar) -> bool {
        *metric = point_box_square_distance_by_corners(&self.point, min, max);
        write!(self.log.borrow_mut(), "r {} ", bits(*metric)).unwrap();
        *metric > self.max_sq
    }
    fn accept(&mut self, index: i32, metric: &V::Scalar) -> Result<bool, Failure> {
        self.count += 1;
        write!(self.log.borrow_mut(), "a {index} {} ", bits(*metric)).unwrap();
        Ok(index % 3 != 0)
    }
}

/// `ProbeBoxSelector`: the default rules with a box overlap test.
pub(super) struct BoxSelector<V: BvhVector> {
    pub query: BvhBox<V>,
    pub log: String,
}
impl<V: BvhVector> BvhBaseTraverse<V::Scalar> for BoxSelector<V> {}
impl<V: BvhVector> BvhTraverse<V> for BoxSelector<V> {
    fn reject_node(&self, min: &V, max: &V, metric: &mut V::Scalar) -> bool {
        *metric = V::Scalar::ZERO;
        self.query.is_out_corners(min, max)
    }
    fn accept(&mut self, index: i32, _metric: &V::Scalar) -> Result<bool, Failure> {
        write!(self.log, "{index} ").unwrap();
        Ok(true)
    }
}

/// `ProbePairSelector`: closer box pairs first, pruned beyond `pair_sq`.
pub(super) struct PairSelector<V: BvhVector> {
    pub pair_sq: V::Scalar,
    pub stop_count: i32,
    pub count: i32,
    pub log: RefCell<String>,
}
impl<V: BvhVector> BvhBaseTraverse<V::Scalar> for PairSelector<V> {
    fn is_metric_better(&self, left: &V::Scalar, right: &V::Scalar) -> bool {
        left < right
    }
    fn reject_metric(&self, metric: &V::Scalar) -> bool {
        *metric > self.pair_sq
    }
    fn stop(&self) -> bool {
        self.stop_count > 0 && self.count >= self.stop_count
    }
}
impl<V: BvhVector> BvhPairTraverse<V> for PairSelector<V> {
    fn reject_node(&self, min1: &V, max1: &V, min2: &V, max2: &V, metric: &mut V::Scalar) -> bool {
        *metric = box_box_square_distance_by_corners(min1, max1, min2, max2);
        write!(self.log.borrow_mut(), "r {} ", bits(*metric)).unwrap();
        *metric > self.pair_sq
    }
    fn accept(&mut self, first: i32, second: i32) -> Result<bool, Failure> {
        self.count += 1;
        write!(self.log.borrow_mut(), "a {first} {second} ").unwrap();
        Ok((first + second) % 2 == 0)
    }
}
