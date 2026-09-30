// Rust translation of FoundationClasses/TKMath/BVH/BVH_Distance.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Distance<T, N, ObjectType, BVHSetType>`: minimum distance from an object
//! to the elements of a BVH set.
use super::scalar::BvhScalar;
use super::traverse::BvhSetTraverse;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;

/// Data and rules of `BVH_Distance`: the running distance (initially
/// `numeric_limits<T>::max()`), the done flag and the object.
///
/// A selector embeds it, implements `reject_node` and `accept` (which lowers
/// [`BvhDistance::distance`] through [`BvhDistance::set_distance`]) and forwards
/// the base rules to [`BvhDistance::is_metric_better`],
/// [`BvhDistance::reject_metric`] and [`BvhDistance::stop`].
/// Example: `let mut state = BvhDistance::<f64, BvhVec3d>::default(); state.set_object(point);`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BvhDistance<T, O> {
    distance: T,
    is_done: bool,
    object: O,
}
impl<T: BvhScalar, O: Default> Default for BvhDistance<T, O> {
    fn default() -> Self {
        Self {
            distance: T::MAX,
            is_done: false,
            object: O::default(),
        }
    }
}
impl<T: BvhScalar, O> BvhDistance<T, O> {
    /// Sets the object to which the distance is required; `state.set_object(point)`.
    pub fn set_object(&mut self, object: O) {
        self.object = object;
    }
    /// The object to which the distance is required (`myObject`); `state.object()`.
    pub fn object(&self) -> &O {
        &self.object
    }
    /// Returns IsDone flag; `state.is_done()`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }
    /// Returns the computed distance; `state.distance()`.
    pub fn distance(&self) -> T {
        self.distance
    }
    /// Writes the protected `myDistance`, as `Accept` does in derived selectors.
    /// Example: `state.set_distance(candidate)`.
    pub fn set_distance(&mut self, distance: T) {
        self.distance = distance;
    }
    /// Writes the protected `myIsDone`, as derived selectors do when they run their own
    /// `ComputeDistance` (BRepExtrema_ProximityDistTool). Example: `state.set_done(false)`.
    pub fn set_done(&mut self, is_done: bool) {
        self.is_done = is_done;
    }
    /// The smaller metric is better; `state.is_metric_better(&1.0, &2.0)`.
    pub fn is_metric_better(&self, left: &T, right: &T) -> bool {
        *left < *right
    }
    /// Rejects the branch whose metric exceeds the current distance.
    /// Example: `state.reject_metric(&metric)`.
    pub fn reject_metric(&self, metric: &T) -> bool {
        *metric > self.distance
    }
    /// Stops the descent once the distance is exactly zero; `state.stop()`.
    pub fn stop(&self) -> bool {
        self.distance == T::ZERO
    }
}

/// `BVH_Distance::ComputeDistance` for a selector that embeds [`BvhDistance`].
/// Example: `let squared = tool.compute_distance()?;`.
pub trait BvhDistanceTraverse<V: BvhVector>: BvhSetTraverse<V> {
    /// Type of the object to which the distance is required (`ObjectType`).
    type Object;
    /// Borrows the embedded distance data.
    fn distance_state(&self) -> &BvhDistance<V::Scalar, Self::Object>;
    /// Mutably borrows the embedded distance data.
    fn distance_state_mut(&mut self) -> &mut BvhDistance<V::Scalar, Self::Object>;
    /// Selects through the set, marks the result done when anything was
    /// accepted, and returns the distance; a failure leaves the flag unchanged.
    fn compute_distance(&mut self) -> Result<V::Scalar, Failure>
    where
        Self: Sized,
    {
        let selected = self.select()?;
        self.distance_state_mut().is_done = selected > 0;
        Ok(self.distance_state().distance)
    }
}
