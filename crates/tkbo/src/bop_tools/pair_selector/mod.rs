use rustycad_tkernel::{ncollection::LinearVector, standard::Failure};
use rustycad_tkmath::bvh::{BvhBaseTraverse, BvhBox, BvhBoxSet, BvhPairTraverse, BvhSet, BvhVec3d};
mod sets;
use sets::PairSets;
/// Mapped element IDs with native lexicographic ordering; `PairIds::new(1,2)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PairIds {
    /// ID mapped from the first set's element.
    pub id1: i32,
    /// ID mapped from the second set's element.
    pub id2: i32,
}
impl PairIds {
    /// Store two IDs; `PairIds::new(4,7)`.
    pub const fn new(id1: i32, id2: i32) -> Self {
        Self { id1, id2 }
    }
}
impl Default for PairIds {
    fn default() -> Self {
        Self::new(-1, -1)
    }
}
/// Native 3D pair-selector rules and retained results; `PairSelector::default()`.
#[derive(Default)]
pub struct PairSelector<'a> {
    pairs: LinearVector<PairIds>,
    same: bool,
    sets: Option<PairSets<'a>>,
}
impl<'a> PairSelector<'a> {
    /// Borrow two distinct sets; `selector.set_bvh_sets(&mut first,&mut second);`.
    pub fn set_bvh_sets(
        &mut self,
        first: &'a mut BvhBoxSet<BvhVec3d>,
        second: &'a mut BvhBoxSet<BvhVec3d>,
    ) {
        self.sets = Some(PairSets::Distinct(first, second));
    }
    /// Borrow one set for both traversal inputs; `selector.set_shared_bvh_set(&mut boxes);`.
    pub fn set_shared_bvh_set(&mut self, set: &'a mut BvhBoxSet<BvhVec3d>) {
        self.sets = Some(PairSets::Shared(set));
    }
    /// Detach both set pointers; `selector.clear_bvh_sets();`.
    pub fn clear_bvh_sets(&mut self) {
        self.sets = None;
    }
    /// Filter mirrored/equal internal positions, regardless of mapped IDs; `selector.set_same(true);`.
    pub fn set_same(&mut self, same: bool) {
        self.same = same;
    }
    /// Clear pairs while retaining sets and filtering; `selector.clear();`.
    pub fn clear(&mut self) {
        self.pairs.clear(false);
    }
    /// Sort mapped pairs lexicographically; `selector.sort();`.
    pub fn sort(&mut self) {
        self.pairs.as_mut_slice().sort_unstable();
    }
    /// Borrow accumulated mapped pairs; `selector.pairs()`.
    pub fn pairs(&self) -> &LinearVector<PairIds> {
        &self.pairs
    }
    /// Build dirty trees and append accepted pairs in traversal order; `selector.select()?`.
    pub fn select(&mut self) -> Result<i32, Failure> {
        let Some(sets) = self.sets.as_mut() else {
            return Ok(0);
        };
        let (first, second) = sets.take_trees()?;
        let result = self.select_trees(Some(&first), Some(second.as_ref().unwrap_or(&first)));
        self.sets
            .as_mut()
            .expect("sets retained during traversal")
            .restore_trees(first, second);
        result
    }
    /// Apply index filtering and element-box rejection; `selector.reject_element(0,1)?`.
    pub fn reject_element(&self, first: i32, second: i32) -> Result<bool, Failure> {
        if self.same && first >= second {
            return Ok(true);
        }
        let (left, right) = self.required_sets(first, second)?.read();
        Ok(left.element_box(first)?.is_out(&right.element_box(second)?))
    }
    fn required_sets(&self, first: i32, second: i32) -> Result<&PairSets<'a>, Failure> {
        self.sets.as_ref().ok_or_else(||Failure::with_type("Standard_NullObject",&format!("BOPTools_PairSelector indices ({first},{second}) have absent sets; expected two BVH_BoxSet inputs"),""))
    }
}
impl BvhBaseTraverse<f64> for PairSelector<'_> {}
impl BvhPairTraverse<BvhVec3d> for PairSelector<'_> {
    fn reject_node(
        &self,
        min1: &BvhVec3d,
        max1: &BvhVec3d,
        min2: &BvhVec3d,
        max2: &BvhVec3d,
        _metric: &mut f64,
    ) -> bool {
        BvhBox::from_corners(*min1, *max1).is_out_corners(min2, max2)
    }
    fn accept(&mut self, first: i32, second: i32) -> Result<bool, Failure> {
        if self.reject_element(first, second)? {
            return Ok(false);
        }
        let (left, right) = self.required_sets(first, second)?.read();
        let pair = PairIds::new(left.element(first)?, right.element(second)?);
        self.pairs.append(pair);
        Ok(true)
    }
}
