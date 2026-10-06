use super::{PackedInteger, PackedMapNode, PackedNodeTable};
impl<T: PackedInteger> PackedNodeTable<T> {
    pub(in super::super) fn insert_value(&mut self, bucket: usize, value: T) {
        let mut head = self.head_ptr(bucket);
        let node = Box::new(PackedMapNode::from_value(value, &mut head));
        self.set_head(bucket, Box::into_raw(node));
    }
    pub(in super::super) fn insert_parts(&mut self, bucket: usize, mask: T::Word, bits: T::Word) {
        let node = Box::new(PackedMapNode::from_parts(mask, bits, self.head_ptr(bucket)));
        self.set_head(bucket, Box::into_raw(node));
    }
    pub(in super::super) fn rehashed(&mut self, modulus: usize) -> Self {
        let mut replacement = Self::allocated(
            modulus
                .checked_add(1)
                .expect("bucket count: expected space for terminal slot"),
        );
        // Public integer traits may panic. Resolve all hashes before changing any ownership.
        let destinations: Vec<usize> = self.nodes().map(|node| node.hash_code(modulus)).collect();
        let mut targets = destinations.into_iter();
        for bucket in 0..self.length {
            self.transfer_chain(bucket, &mut replacement, &mut targets);
        }
        replacement
    }
    fn transfer_chain(
        &mut self,
        bucket: usize,
        replacement: &mut Self,
        targets: &mut impl Iterator<Item = usize>,
    ) {
        let mut pointer = self.head_ptr(bucket);
        self.set_head(bucket, std::ptr::null_mut());
        while !pointer.is_null() {
            // No user callback occurs between unlinking and linking this owned node.
            let node = unsafe { &mut *pointer };
            let next = node.next();
            let target = targets.next().expect("one destination per owned node");
            node.set_next(replacement.head_ptr(target));
            replacement.set_head(target, pointer);
            pointer = next;
        }
    }
    pub(in super::super) fn remove_value(
        &mut self,
        bucket: usize,
        index: T::Word,
        value: T,
    ) -> (bool, bool) {
        let mut pointer = self.head_ptr(bucket);
        let mut previous: *mut PackedMapNode<T> = std::ptr::null_mut();
        while !pointer.is_null() {
            // Exclusive table access protects both this node and its predecessor link.
            let node = unsafe { &mut *pointer };
            if node.is_equal(index) {
                return self.remove_selected(bucket, previous, pointer, value);
            }
            previous = pointer;
            pointer = node.next();
        }
        (false, false)
    }
    fn remove_selected(
        &mut self,
        bucket: usize,
        previous: *mut PackedMapNode<T>,
        pointer: *mut PackedMapNode<T>,
        value: T,
    ) -> (bool, bool) {
        // This node was found in the owned chain while the table is exclusively borrowed.
        let node = unsafe { &mut *pointer };
        let removed = node.del_value(value);
        let destroyed = removed && !node.has_values();
        if destroyed {
            self.unlink(bucket, previous, pointer);
        }
        (removed, destroyed)
    }
    fn unlink(
        &mut self,
        bucket: usize,
        previous: *mut PackedMapNode<T>,
        pointer: *mut PackedMapNode<T>,
    ) {
        // Callers supply one live node and its actual predecessor under exclusive table access.
        let owner = unsafe { Box::from_raw(pointer) };
        if previous.is_null() {
            self.set_head(bucket, owner.next());
        } else {
            unsafe { (*previous).set_next(owner.next()) };
        }
    }
}
