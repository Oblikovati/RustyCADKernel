use super::{BoundingNode, BoundingVolume, UBTree};
use crate::ncollection::AllocatedBox;
impl<T: Clone + Default, B: BoundingVolume> UBTree<T, B> {
    pub(in crate::ncollection) fn insert_leaf(&mut self, object: T, bounds: B) -> usize {
        let Some(branch) = self.insertion_branch(&bounds) else {
            let owner = self
                .resource
                .allocate_value(BoundingNode::new(object, bounds));
            return self.publish_root(owner);
        };
        let union = self.insertion_union(branch, &bounds);
        let inserted = self.gemmate(branch, union, object, bounds).unwrap();
        self.last = Some(inserted);
        inserted
    }
    pub(super) fn publish_root(&mut self, owner: AllocatedBox<BoundingNode<T, B>>) -> usize {
        let index = self.nodes.push_single(owner);
        self.root = Some(index);
        self.last = Some(index);
        index
    }
    pub(super) fn insertion_union(&self, branch: usize, bounds: &B) -> B {
        let mut union = bounds.clone();
        union.add(&self.node(branch).bounds);
        union
    }
    pub(super) fn insertion_branch(&mut self, bounds: &B) -> Option<usize> {
        let mut branch = self.root?;
        let mut outside = self.node(branch).bounds.is_out(bounds);
        loop {
            if outside || self.node(branch).is_leaf() {
                return Some(branch);
            }
            self.node_mut(branch).bounds.add(bounds);
            (branch, outside) = self.best_child(branch, bounds);
        }
    }
    fn best_child(&self, branch: usize, bounds: &B) -> (usize, bool) {
        let [first, second] = self.node(branch).children.unwrap();
        let out_first: bool = self.node(first).bounds.is_out(bounds);
        let out_second: bool = self.node(second).bounds.is_out(bounds);
        if out_first != out_second {
            return if out_first {
                (second, out_second)
            } else {
                (first, out_first)
            };
        }
        (self.least_extent_child(first, second, bounds), out_first)
    }
    fn least_extent_child(&self, first: usize, second: usize, bounds: &B) -> usize {
        let mut first_union = bounds.clone();
        let mut second_union = bounds.clone();
        first_union.add(&self.node(first).bounds);
        second_union.add(&self.node(second).bounds);
        if first_union.square_extent() > second_union.square_extent() {
            second
        } else {
            first
        }
    }
    pub(in crate::ncollection) fn reparent_children(&mut self, parent: usize) {
        if let Some(children) = self.node(parent).children {
            for child in children {
                self.node_mut(child).parent = Some(parent);
            }
        }
    }
    pub(in crate::ncollection) fn previous_leaf(&self, inserted: usize) -> Option<usize> {
        let parent: usize = self.node(inserted).parent?;
        let first: usize = self.node(parent).children.unwrap()[0];
        self.node(first).is_leaf().then_some(first)
    }
}
