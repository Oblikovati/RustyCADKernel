// Rust translation of FoundationClasses/TKMath/Poly/Poly_CoherentTriPtr.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Poly_CoherentTriPtr`: a circular list of allocator-owned triangle members.
use rustycad_tkernel::ncollection::{AllocatedBox, BaseAllocator};
use rustycad_tkernel::standard::Failure;
use std::collections::HashMap;
use std::fmt;

/// Stable identity for one allocator-owned ring member; `ring.head_member()`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CoherentTriMember(usize);

#[derive(Debug)]
struct RingMember {
    triangle: usize,
    next: CoherentTriMember,
    previous: CoherentTriMember,
}

struct MemberAllocation {
    allocator: BaseAllocator,
    member: AllocatedBox<RingMember>,
}

/// Circular doubly linked members with stable identities and matching allocators.
pub struct CoherentTriPtr {
    head: Option<CoherentTriMember>,
    length: usize,
    members: HashMap<CoherentTriMember, MemberAllocation>,
}

impl CoherentTriPtr {
    /// A one-member ring; `CoherentTriPtr::new(0)`.
    pub fn new(triangle: usize) -> Self {
        Self::new_in(triangle, BaseAllocator::common())
    }

    /// A one-member ring in a selected allocator; `CoherentTriPtr::new_in(0, arena)`.
    pub fn new_in(triangle: usize, allocator: BaseAllocator) -> Self {
        let mut ring = Self {
            head: None,
            length: 0,
            members: HashMap::new(),
        };
        let member = ring.allocate_member(triangle, allocator);
        ring.members.get_mut(&member).unwrap().member.next = member;
        ring.members.get_mut(&member).unwrap().member.previous = member;
        ring.head = Some(member);
        ring.length = 1;
        ring
    }

    /// Stable identity of the head member; `ring.head_member().unwrap()`.
    pub fn head_member(&self) -> Option<CoherentTriMember> {
        self.head
    }

    /// Triangle index stored at a stable member; `ring.triangle(member)?`.
    pub fn triangle(&self, member: CoherentTriMember) -> Result<usize, Failure> {
        Ok(self.member(member)?.triangle)
    }

    /// Changes a stable member's triangle index; `ring.set_member_triangle(member, 3)?`.
    pub fn set_member_triangle(
        &mut self,
        member: CoherentTriMember,
        triangle: usize,
    ) -> Result<(), Failure> {
        self.member_mut(member)?.triangle = triangle;
        Ok(())
    }

    /// Next stable member, wrapping at the end; `ring.next_member(member)?`.
    pub fn next_member(&self, member: CoherentTriMember) -> Result<CoherentTriMember, Failure> {
        Ok(self.member(member)?.next)
    }

    /// Previous stable member, wrapping at the head; `ring.previous_member(member)?`.
    pub fn previous_member(&self, member: CoherentTriMember) -> Result<CoherentTriMember, Failure> {
        Ok(self.member(member)?.previous)
    }

    /// Inserts after a stable member using the supplied allocator; `ring.append_member(m, 4, arena)?`.
    pub fn append_member(
        &mut self,
        member: CoherentTriMember,
        triangle: usize,
        allocator: BaseAllocator,
    ) -> Result<CoherentTriMember, Failure> {
        let following = self.member(member)?.next;
        let added = self.allocate_member(triangle, allocator);
        self.link_between(member, added, following);
        Ok(added)
    }

    /// Inserts before a stable member using the supplied allocator; `ring.prepend_member(m, 4, arena)?`.
    pub fn prepend_member(
        &mut self,
        member: CoherentTriMember,
        triangle: usize,
        allocator: BaseAllocator,
    ) -> Result<CoherentTriMember, Failure> {
        let preceding = self.member(member)?.previous;
        let added = self.allocate_member(triangle, allocator);
        self.link_between(preceding, added, member);
        Ok(added)
    }

    /// Unlinks a stable member through its creating allocator; `ring.remove_member(m, &arena)?`.
    pub fn remove_member(
        &mut self,
        member: CoherentTriMember,
        allocator: &BaseAllocator,
    ) -> Result<(), Failure> {
        self.check_allocator(member, allocator)?;
        self.unlink_member(member);
        Ok(())
    }

    /// Frees every node only when all were made by the supplied allocator; `ring.remove_list_in(&arena)?`.
    pub fn remove_list_in(&mut self, allocator: &BaseAllocator) -> Result<(), Failure> {
        for (member, allocation) in &self.members {
            if allocation.allocator != *allocator {
                return Err(allocator_mismatch(*member));
            }
        }
        while self.drain_next().is_some() {}
        Ok(())
    }

    /// Triangle of the head member, as `GetTriangle`; `ring.get_triangle()`.
    pub fn get_triangle(&self) -> usize {
        self.triangle(self.head.expect("a nonempty coherent ring has a head"))
            .expect("the head member belongs to its ring")
    }

    /// Re-points the head member, as `SetTriangle`; `ring.set_triangle(3)`.
    pub fn set_triangle(&mut self, triangle: usize) {
        let head = self.head.expect("a nonempty coherent ring has a head");
        self.set_member_triangle(head, triangle)
            .expect("the head member belongs to its ring");
    }

    /// Number of members; `ring.len()`.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Checks whether the circular list has no members; `ring.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Positional successor retained for existing Rust callers; `ring.next(0)`.
    pub fn next(&self, member: usize) -> usize {
        let member = self.member_at(member).expect("member position in ring");
        self.position(self.next_member(member).unwrap()).unwrap()
    }

    /// Positional predecessor retained for existing Rust callers; `ring.previous(0)`.
    pub fn previous(&self, member: usize) -> usize {
        let member = self.member_at(member).expect("member position in ring");
        self.position(self.previous_member(member).unwrap())
            .unwrap()
    }

    /// Triangle index at a position retained for existing Rust callers; `ring.triangle_at(1)`.
    pub fn triangle_at(&self, member: usize) -> Option<usize> {
        self.member_at(member)
            .and_then(|token| self.triangle(token).ok())
    }

    /// Inserts after a position using that member's allocator; `ring.append(0, 5)`.
    pub fn append(&mut self, member: usize, triangle: usize) {
        let target = self.member_at(member).expect("member position in ring");
        let allocator = self.allocation(target).allocator.clone();
        self.append_member(target, triangle, allocator)
            .expect("target member belongs to ring");
    }

    /// Inserts before a position; before the head it remains the final member; `ring.prepend(0, 5)`.
    pub fn prepend(&mut self, member: usize, triangle: usize) {
        let target = self.member_at(member).expect("member position in ring");
        let allocator = self.allocation(target).allocator.clone();
        self.prepend_member(target, triangle, allocator)
            .expect("target member belongs to ring");
    }

    /// Removes a position and advances the head when it names the old head; `ring.remove(1)`.
    pub fn remove(&mut self, member: usize) {
        let target = self.member_at(member).expect("member position in ring");
        self.unlink_member(target);
    }

    /// Triangles from the head; `ring.iter().collect::<Vec<_>>()`.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        CoherentTriPtrIterator {
            ring: Some(self),
            first: self.head,
            current: self.head,
        }
    }

    /// Circular iteration from a stable member; `ring.iter_from_member(m).collect::<Vec<_>>()`.
    pub fn iter_from_member(&self, member: CoherentTriMember) -> CoherentTriPtrIterator<'_> {
        let valid = self.members.contains_key(&member).then_some(member);
        CoherentTriPtrIterator {
            ring: Some(self),
            first: valid,
            current: valid,
        }
    }

    /// Circular iteration from a position retained for existing Rust callers; `ring.iter_from(2)`.
    pub fn iter_from(&self, member: usize) -> CoherentTriPtrIterator<'_> {
        let token = self.member_at(member);
        CoherentTriPtrIterator {
            ring: Some(self),
            first: token,
            current: token,
        }
    }

    /// Drops every allocator-owned ring node; `ring.remove_list()`.
    pub fn remove_list(self) {}

    fn allocate_member(&mut self, triangle: usize, allocator: BaseAllocator) -> CoherentTriMember {
        let member = allocator.allocate_value(RingMember {
            triangle,
            next: CoherentTriMember(0),
            previous: CoherentTriMember(0),
        });
        let token = CoherentTriMember((&*member as *const RingMember) as usize);
        self.members
            .insert(token, MemberAllocation { allocator, member });
        token
    }

    fn link_between(
        &mut self,
        previous: CoherentTriMember,
        member: CoherentTriMember,
        next: CoherentTriMember,
    ) {
        self.member_mut(member).unwrap().previous = previous;
        self.member_mut(member).unwrap().next = next;
        self.member_mut(previous).unwrap().next = member;
        self.member_mut(next).unwrap().previous = member;
        self.length += 1;
    }

    fn unlink_member(&mut self, member: CoherentTriMember) {
        let previous = self.member(member).unwrap().previous;
        let next = self.member(member).unwrap().next;
        if self.length == 1 {
            self.head = None;
        } else {
            self.member_mut(previous).unwrap().next = next;
            self.member_mut(next).unwrap().previous = previous;
            if self.head == Some(member) {
                self.head = Some(next);
            }
        }
        self.members.remove(&member);
        self.length -= 1;
    }

    fn drain_next(&mut self) -> Option<usize> {
        let head = self.head?;
        let triangle = self.member(head).ok()?.triangle;
        self.unlink_member(head);
        Some(triangle)
    }

    fn check_allocator(
        &self,
        member: CoherentTriMember,
        allocator: &BaseAllocator,
    ) -> Result<(), Failure> {
        let allocation = self
            .members
            .get(&member)
            .ok_or_else(|| missing_member(member))?;
        if &allocation.allocator == allocator {
            return Ok(());
        }
        Err(allocator_mismatch(member))
    }

    fn member_at(&self, position: usize) -> Option<CoherentTriMember> {
        let mut current = self.head?;
        for _ in 0..position {
            current = self.member(current).ok()?.next;
        }
        (position < self.length).then_some(current)
    }

    fn position(&self, member: CoherentTriMember) -> Option<usize> {
        let mut current = self.head?;
        for position in 0..self.length {
            if current == member {
                return Some(position);
            }
            current = self.member(current).ok()?.next;
        }
        None
    }

    fn member(&self, member: CoherentTriMember) -> Result<&RingMember, Failure> {
        self.members
            .get(&member)
            .map(|allocation| &*allocation.member)
            .ok_or_else(|| missing_member(member))
    }

    fn member_mut(&mut self, member: CoherentTriMember) -> Result<&mut RingMember, Failure> {
        self.members
            .get_mut(&member)
            .map(|allocation| &mut *allocation.member)
            .ok_or_else(|| missing_member(member))
    }

    fn allocation(&self, member: CoherentTriMember) -> &MemberAllocation {
        self.members
            .get(&member)
            .expect("member token belongs to ring")
    }
}

impl Clone for CoherentTriPtr {
    fn clone(&self) -> Self {
        let Some(head) = self.head else {
            return Self {
                head: None,
                length: 0,
                members: HashMap::new(),
            };
        };
        let mut clone = Self::new_in(self.get_triangle(), self.allocation(head).allocator.clone());
        let mut current = self.next_member(head).unwrap();
        while current != head {
            let source = self.allocation(current);
            let tail = clone.member(clone.head.unwrap()).unwrap().previous;
            clone
                .append_member(tail, source.member.triangle, source.allocator.clone())
                .unwrap();
            current = self.next_member(current).unwrap();
        }
        clone
    }
}

impl PartialEq for CoherentTriPtr {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}
impl Eq for CoherentTriPtr {}
impl Drop for CoherentTriPtr {
    fn drop(&mut self) {
        while self.drain_next().is_some() {}
    }
}

impl fmt::Debug for CoherentTriPtr {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.debug_list().entries(self.iter()).finish()
    }
}

/// `Poly_CoherentTriPtr::Iterator`: a cursor that stops before revisiting its start.
#[derive(Clone, Copy, Debug, Default)]
pub struct CoherentTriPtrIterator<'a> {
    ring: Option<&'a CoherentTriPtr>,
    first: Option<CoherentTriMember>,
    current: Option<CoherentTriMember>,
}

impl CoherentTriPtrIterator<'_> {
    /// Triangle of the starting member, or `None` for an invalid cursor; `cursor.first()`.
    pub fn first(&self) -> Option<usize> {
        self.ring?.triangle(self.first?).ok()
    }

    /// True while a member is current; `cursor.more()`.
    pub const fn more(&self) -> bool {
        self.current.is_some()
    }

    /// Moves forward and stops before revisiting the start; `cursor.advance()`.
    pub fn advance(&mut self) {
        let (Some(ring), Some(current), Some(first)) = (self.ring, self.current, self.first) else {
            return;
        };
        let next = ring.member(current).map(|member| member.next).ok();
        self.current = next.filter(|member| *member != first);
    }

    /// Triangle stored at the current member; `cursor.value()`.
    pub fn value(&self) -> Option<usize> {
        self.ring?.triangle(self.current?).ok()
    }

    /// Positional token kept for compatibility; `cursor.ptr_value()`.
    pub fn ptr_value(&self) -> Option<usize> {
        self.ring?.position(self.current?)
    }

    /// Stable node identity corresponding to native `PtrValue`; `cursor.member_token()`.
    pub const fn member_token(&self) -> Option<CoherentTriMember> {
        self.current
    }
}

impl Iterator for CoherentTriPtrIterator<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.value()?;
        self.advance();
        Some(value)
    }
}

fn missing_member(member: CoherentTriMember) -> Failure {
    let message = format!("coherent ring member {member:?} is not live");
    Failure::with_type("Standard_NoSuchObject", &message, "")
}

fn allocator_mismatch(member: CoherentTriMember) -> Failure {
    let message = format!("allocator does not own coherent ring member {member:?}");
    Failure::with_type("Standard_ProgramError", &message, "")
}

#[cfg(test)]
mod tests {
    use super::CoherentTriPtr;
    use rustycad_tkernel::ncollection::BaseAllocator;

    #[test]
    fn list_drain_follows_the_native_head_then_next_order() {
        let allocator = BaseAllocator::common();
        let mut ring = CoherentTriPtr::new_in(1, allocator.clone());
        let head = ring.head_member().unwrap();
        ring.append_member(head, 2, allocator.clone()).unwrap();
        ring.prepend_member(head, 3, allocator).unwrap();
        assert_eq!(
            std::iter::from_fn(|| ring.drain_next()).collect::<Vec<_>>(),
            [1, 2, 3]
        );
    }
}
