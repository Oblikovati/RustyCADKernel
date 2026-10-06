//! Compute native destinations before rewiring, so a Rust hash panic leaves both old chains intact.
use super::{DoubleMapNode, DoubleStorage, MapLinkNode};
impl<A, B> DoubleStorage<A, B> {
    pub(in crate::ncollection::double_map) fn resize(
        &mut self,
        requested: usize,
        hash: impl Fn(&DoubleMapNode<A, B>, usize) -> (usize, usize),
    ) {
        let mut plan = self
            .base()
            .begin_resize(requested)
            .expect("DoubleMap resize requested valid native allocation");
        if !plan.is_resize() {
            return;
        }
        let routes = self.destinations(plan.bucket_count(), hash);
        Self::rewire(&mut plan, routes);
        self.topology.edit(|base| base.end_resize(plan));
    }
    fn rewire(
        plan: &mut crate::ncollection::BaseDoubleMapResize,
        routes: Vec<(*mut MapLinkNode, usize, usize)>,
    ) {
        for (node, first, second) in routes {
            unsafe {
                Self::set_next(node, plan.bucket_slots_mut()[first], false);
                Self::set_next(node, plan.second_slots_mut()[second], true);
            }
            plan.bucket_slots_mut()[first] = node;
            plan.second_slots_mut()[second] = node;
        }
    }
    fn destinations(
        &self,
        count: usize,
        hash: impl Fn(&DoubleMapNode<A, B>, usize) -> (usize, usize),
    ) -> Vec<(*mut MapLinkNode, usize, usize)> {
        let mut routes = Vec::with_capacity(self.len());
        for &head in self.base().bucket_slots() {
            let mut node = head;
            while !node.is_null() {
                let (first, second) = hash(unsafe { self.node(node) }, count);
                routes.push((node, first, second));
                node = unsafe { Self::next(node, false) };
            }
        }
        routes
    }
}
