// Rust translation of FoundationClasses/TKMath/Poly/Poly_CoherentTriPtr.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Poly_CoherentTriPtr`: the round list of triangles incident to a node.
//!
//! Natively each list member is a heap node holding a triangle pointer, linked both ways
//! in a ring, and the owner keeps a pointer to one member (the head). Here the ring is a
//! vector of triangle indices ordered from the head, and a member is named by its position
//! in that order. Iteration from the head, `Append`/`Prepend` next to a member and
//! `Remove` keep the native order: prepending before the head puts the new member last.
/// Ring of triangle indices seen from its head; `CoherentTriPtr::new(0)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoherentTriPtr {
    ring: Vec<usize>,
}
impl CoherentTriPtr {
    /// A one-member ring; `CoherentTriPtr::new(triangle)`.
    pub fn new(triangle: usize) -> Self {
        Self {
            ring: vec![triangle],
        }
    }
    /// Triangle of the head member, as `GetTriangle`; `ring.get_triangle()`.
    pub fn get_triangle(&self) -> usize {
        self.ring[0]
    }
    /// Re-points the head member, as `SetTriangle`; `ring.set_triangle(3)`.
    pub fn set_triangle(&mut self, triangle: usize) {
        self.ring[0] = triangle;
    }
    /// Number of members; `ring.len()`.
    pub fn len(&self) -> usize {
        self.ring.len()
    }
    /// A ring always has a head; `ring.is_empty()` is false while it exists.
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }
    /// Position after `member`, wrapping to the head, as `Next`; `ring.next(0)`.
    pub fn next(&self, member: usize) -> usize {
        (member + 1) % self.ring.len()
    }
    /// Position before `member`, wrapping to the last, as `Previous`; `ring.previous(0)`.
    pub fn previous(&self, member: usize) -> usize {
        (member + self.ring.len() - 1) % self.ring.len()
    }
    /// Triangle stored at a position; `ring.triangle_at(1)`.
    pub fn triangle_at(&self, member: usize) -> Option<usize> {
        self.ring.get(member).copied()
    }
    /// Inserts a triangle after `member`, as `Append`; `ring.append(0, 5)`.
    pub fn append(&mut self, member: usize, triangle: usize) {
        self.ring.insert(member + 1, triangle);
    }
    /// Inserts a triangle before `member`, as `Prepend`; before the head it becomes the
    /// last member; `ring.prepend(0, 5)`.
    pub fn prepend(&mut self, member: usize, triangle: usize) {
        if member == 0 {
            self.ring.push(triangle);
        } else {
            self.ring.insert(member, triangle);
        }
    }
    /// Unlinks the member at a position, as the static `Remove`; the next member becomes
    /// the head when the head is removed; `ring.remove(1)`.
    pub fn remove(&mut self, member: usize) {
        self.ring.remove(member);
    }
    /// Triangles from the head; `ring.iter().collect::<Vec<_>>()`.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.ring.iter().copied()
    }
    /// The native cursor started at a member, going once round the ring, as
    /// `Iterator(member)`; `ring.iter_from(2)`.
    pub fn iter_from(&self, member: usize) -> CoherentTriPtrIterator<'_> {
        CoherentTriPtrIterator {
            ring: Some(self),
            first: member,
            current: (member < self.ring.len()).then_some(member),
        }
    }
    /// Drops the whole ring, as the static `RemoveList`; `ring.remove_list()`.
    pub fn remove_list(self) {}
}
/// `Poly_CoherentTriPtr::Iterator`: a cursor that stops before reaching its first member
/// again; the default cursor is empty; `ring.iter_from(0)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct CoherentTriPtrIterator<'a> {
    ring: Option<&'a CoherentTriPtr>,
    first: usize,
    current: Option<usize>,
}
impl CoherentTriPtrIterator<'_> {
    /// Triangle of the starting member, `None` for the empty cursor, as `First`; `cursor.first()`.
    pub fn first(&self) -> Option<usize> {
        self.ring.and_then(|ring| ring.triangle_at(self.first))
    }
    /// True while a member is current, as `More`; `cursor.more()`.
    pub const fn more(&self) -> bool {
        self.current.is_some()
    }
    /// Moves to the next member, ending at the starting one, as `Next`; `cursor.advance()`.
    pub fn advance(&mut self) {
        let (Some(ring), Some(current)) = (self.ring, self.current) else {
            return;
        };
        let next = ring.next(current);
        self.current = (next != self.first).then_some(next);
    }
    /// Triangle of the current member, as `Value` and `ChangeValue`; `cursor.value()`.
    pub fn value(&self) -> Option<usize> {
        self.ring
            .zip(self.current)
            .and_then(|(ring, member)| ring.triangle_at(member))
    }
    /// Position of the current member, as `PtrValue`; `cursor.ptr_value()`.
    pub const fn ptr_value(&self) -> Option<usize> {
        self.current
    }
}
impl Iterator for CoherentTriPtrIterator<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        let value = self.value()?;
        self.advance();
        Some(value)
    }
}
