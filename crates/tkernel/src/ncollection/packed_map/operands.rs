//! Destination aliases are explicit because Rust cannot borrow a mutable map as an operand.
use super::{PackedInteger, PackedMap};
use crate::ncollection::packed_map_algo as algo;

/// Identifies a native set-operation operand without overlapping Rust references.
/// Example: `PackedMapOperand::<i32>::Destination` selects the receiving map.
#[derive(Clone, Copy)]
pub enum PackedMapOperand<'a, T: PackedInteger> {
    /// Use the receiving map's current contents and native object identity.
    Destination,
    /// Use a distinct map; two operands may borrow the same source.
    Borrowed(&'a PackedMap<T>),
}

impl<T: PackedInteger> PackedMap<T> {
    /// Applies native Union, including destination aliases; `map.union_operands(Destination, Borrowed(&other))`.
    pub fn union_operands(
        &mut self,
        left: PackedMapOperand<'_, T>,
        right: PackedMapOperand<'_, T>,
    ) {
        use PackedMapOperand::{Borrowed, Destination};
        match (left, right) {
            (Borrowed(left), Borrowed(right)) => algo::union(self, left, right),
            (Destination, Borrowed(other)) | (Borrowed(other), Destination) => {
                self.union_destination(other);
            }
            (Destination, Destination) => {}
        }
    }

    /// Applies native Intersection with explicit aliases; `map.intersection_operands(Destination, Borrowed(&other))`.
    pub fn intersection_operands(
        &mut self,
        left: PackedMapOperand<'_, T>,
        right: PackedMapOperand<'_, T>,
    ) {
        use PackedMapOperand::{Borrowed, Destination};
        match (left, right) {
            (Borrowed(left), Borrowed(right)) => algo::intersection(self, left, right),
            (Destination, Borrowed(other)) | (Borrowed(other), Destination) => {
                self.intersect_destination(other);
            }
            (Destination, Destination) if self.is_empty() => self.clear(),
            (Destination, Destination) => {}
        }
    }

    /// Applies native Subtraction with ordered aliases; `map.subtraction_operands(Borrowed(&left), Destination)`.
    pub fn subtraction_operands(
        &mut self,
        left: PackedMapOperand<'_, T>,
        right: PackedMapOperand<'_, T>,
    ) {
        use PackedMapOperand::{Borrowed, Destination};
        match (left, right) {
            (Borrowed(left), Borrowed(right)) => algo::subtraction(self, left, right),
            (Destination, Borrowed(right)) => {
                algo::subtract(self, right);
            }
            (Borrowed(left), Destination) => self.subtract_from_left(left),
            (Destination, Destination) => {
                self.subtract_self();
            }
        }
    }

    /// Applies native Difference with explicit aliases; `map.difference_operands(Destination, Destination)` clears it.
    pub fn difference_operands(
        &mut self,
        left: PackedMapOperand<'_, T>,
        right: PackedMapOperand<'_, T>,
    ) {
        use PackedMapOperand::{Borrowed, Destination};
        match (left, right) {
            (Borrowed(left), Borrowed(right)) => algo::difference(self, left, right),
            (Destination, Borrowed(other)) | (Borrowed(other), Destination) => {
                algo::differ(self, other);
            }
            (Destination, Destination) => self.clear(),
        }
    }

    fn union_destination(&mut self, other: &Self) {
        if self.is_empty() {
            self.assign(other);
            return;
        }
        algo::unite(self, other);
    }

    fn intersect_destination(&mut self, other: &Self) {
        if self.is_empty() || other.is_empty() {
            self.clear();
            return;
        }
        algo::intersect(self, other);
    }

    fn subtract_from_left(&mut self, left: &Self) {
        // Native right-alias subtraction builds a default map, resetting the destination modulus.
        let mut copy = Self::default();
        algo::subtraction(&mut copy, left, self);
        self.move_assign(&mut copy);
    }
}
