use super::{BoundingNode, BoundingVolume, UBTree};
impl<T: Clone + Default, B: BoundingVolume> UBTree<T, B> {
    pub(in crate::ncollection) fn insert_leaf(&mut self, object: T, bounds: B) -> usize {
        let Some(mut branch) = self.root else {
            let index: usize = self.allocate_node(BoundingNode::new(object, bounds));
            self.root = Some(index);
            self.last = Some(index);
            return index;
        };
        let mut outside = self.node(branch).bounds.is_out(&bounds);
        loop {
            if outside || self.node(branch).is_leaf() {
                let mut union: B = bounds.clone();
                union.add(&self.node(branch).bounds);
                let inserted = self.gemmate(branch, union, object, bounds).unwrap();
                self.last = Some(inserted);
                return inserted;
            }
            self.node_mut(branch).bounds.add(&bounds);
            (branch, outside) = self.best_child(branch, &bounds);
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
        let mut first_union: B = bounds.clone();
        let mut second_union: B = bounds.clone();
        first_union.add(&self.node(first).bounds);
        second_union.add(&self.node(second).bounds);
        if first_union.square_extent() > second_union.square_extent() {
            (second, out_second)
        } else {
            (first, out_first)
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
