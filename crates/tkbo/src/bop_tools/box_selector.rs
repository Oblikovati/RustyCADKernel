use rustycad_tkernel::{ncollection::List, standard::Failure};
use rustycad_tkmath::bvh::{
    BvhBaseTraverse, BvhBox, BvhBoxSet, BvhSet, BvhSetTraverse, BvhTraverse, BvhVector,
};
/// Source BVH box selector with retained ordered results; `BoxSelector::<BvhVec3d>::default()`.
pub struct BoxSelector<'a, V: BvhVector<Scalar = f64>> {
    bounds: BvhBox<V>,
    indices: List<i32>,
    set: Option<&'a mut BvhBoxSet<V>>,
}
impl<V: BvhVector<Scalar = f64>> Default for BoxSelector<'_, V> {
    fn default() -> Self {
        Self {
            bounds: BvhBox::default(),
            indices: List::default(),
            set: None,
        }
    }
}
impl<'a, V: BvhVector<Scalar = f64>> BoxSelector<'a, V> {
    /// Replace the traversal set, including an absent set; `selector.set_bvh_set(Some(&mut boxes));`.
    pub fn set_bvh_set(&mut self, set: Option<&'a mut BvhBoxSet<V>>) {
        self.set = set;
    }
    /// Copy the query box; `selector.set_box(bounds);`.
    pub fn set_box(&mut self, bounds: BvhBox<V>) {
        self.bounds = bounds;
    }
    /// Clear only prior indices; `selector.clear();`.
    pub fn clear(&mut self) {
        self.indices.clear();
    }
    /// Borrow traversal-order element IDs; `selector.indices()`.
    pub fn indices(&self) -> &List<i32> {
        &self.indices
    }
    /// Reject a disjoint element box; `selector.reject_element(0)?`.
    pub fn reject_element(&self, index: i32) -> Result<bool, Failure> {
        Ok(self
            .bounds
            .is_out(&self.required_set(index)?.element_box(index)?))
    }
    fn required_set(&self, index: i32) -> Result<&BvhBoxSet<V>, Failure> {
        self.set.as_deref().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                &format!(
                    "BOPTools_BoxSelector element index {index} has absent set; expected BVH_BoxSet"
                ),
                "",
            )
        })
    }
}
impl<V: BvhVector<Scalar = f64>> BvhBaseTraverse<bool> for BoxSelector<'_, V> {}
impl<V: BvhVector<Scalar = f64>> BvhTraverse<V, bool> for BoxSelector<'_, V> {
    fn reject_node(&self, minimum: &V, maximum: &V, inside: &mut bool) -> bool {
        let (contained, overlap) = self.bounds.contains_corners(minimum, maximum);
        *inside = contained;
        !overlap
    }
    fn accept_metric(&self, inside: &bool) -> bool {
        *inside
    }
    fn accept(&mut self, index: i32, inside: &bool) -> Result<bool, Failure> {
        if !inside && self.reject_element(index)? {
            return Ok(false);
        }
        let element = self.required_set(index)?.element(index)?;
        self.indices.append(element);
        Ok(true)
    }
}
impl<V: BvhVector<Scalar = f64>> BvhSetTraverse<V, bool> for BoxSelector<'_, V> {
    type Set = BvhBoxSet<V>;
    fn bvh_set_mut(&mut self) -> Option<&mut Self::Set> {
        self.set.as_deref_mut()
    }
}
