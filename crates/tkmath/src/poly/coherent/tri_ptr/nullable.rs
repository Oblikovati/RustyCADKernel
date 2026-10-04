//! Nullable triangle-pointer construction and safe inspection on the coherent ring.
use super::{CoherentTriMember, CoherentTriPtr};
use crate::poly::coherent::tri_ptr::MemberAllocation;
use rustycad_tkernel::{ncollection::BaseAllocator, standard::Failure};
use std::collections::HashMap;

impl CoherentTriPtr {
    /// Exposes OCCT's protected pointer constructor for defined null-member lifecycle operations;
    /// `CoherentTriPtr::new_from_nullable_triangle(None)` creates a linked node without reading it.
    pub fn new_from_nullable_triangle(triangle: Option<usize>) -> Self {
        Self::new_from_nullable_triangle_in(triangle, BaseAllocator::common())
    }

    /// Allocates the protected-constructor analogue in an explicit arena;
    /// `CoherentTriPtr::new_from_nullable_triangle_in(None, arena)`.
    pub fn new_from_nullable_triangle_in(
        triangle: Option<usize>,
        allocator: BaseAllocator,
    ) -> Self {
        let mut ring = empty_ring_storage();
        let member = ring.allocate_member(triangle, allocator);
        set_member_as_singleton(&mut ring, member);
        ring
    }

    /// Reads nullable storage without dereferencing it; `ring.triangle_nullable(member)?`.
    pub fn triangle_nullable(&self, member: CoherentTriMember) -> Result<Option<usize>, Failure> {
        Ok(self.member(member)?.triangle)
    }

    /// Stores a nullable triangle pointer value; `ring.set_member_triangle_nullable(member, None)?`.
    pub fn set_member_triangle_nullable(
        &mut self,
        member: CoherentTriMember,
        triangle: Option<usize>,
    ) -> Result<(), Failure> {
        self.member_mut(member)?.triangle = triangle;
        Ok(())
    }

    /// Reads the head triangle without panicking on an empty or nullable ring;
    /// `ring.try_get_triangle()?` returns `None` for a stored null pointer.
    pub fn try_get_triangle(&self) -> Result<Option<usize>, Failure> {
        let head = self.head.ok_or_else(super::empty_ring)?;
        self.triangle_nullable(head)
    }

    /// Sets or clears the head triangle pointer; `ring.set_triangle_nullable(None)?`.
    pub fn set_triangle_nullable(&mut self, triangle: Option<usize>) -> Result<(), Failure> {
        let head = self.head.ok_or_else(super::empty_ring)?;
        self.set_member_triangle_nullable(head, triangle)
    }

    pub(super) fn append_member_value(
        &mut self,
        member: CoherentTriMember,
        triangle: Option<usize>,
        allocator: BaseAllocator,
    ) -> Result<CoherentTriMember, Failure> {
        let following = self.member(member)?.next;
        let added = self.allocate_member(triangle, allocator);
        self.link_between(member, added, following);
        Ok(added)
    }

    pub(super) fn triangles_nullable(&self) -> NullableTriangleValues<'_> {
        NullableTriangleValues {
            ring: self,
            current: self.head,
            remaining: self.length,
        }
    }
}

pub(super) struct NullableTriangleValues<'a> {
    ring: &'a CoherentTriPtr,
    current: Option<CoherentTriMember>,
    remaining: usize,
}

impl Iterator for NullableTriangleValues<'_> {
    type Item = Option<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let current = self.current?;
        let member = self.ring.member(current).ok()?;
        self.current = Some(member.next);
        self.remaining -= 1;
        Some(member.triangle)
    }
}

fn empty_ring_storage() -> CoherentTriPtr {
    CoherentTriPtr {
        head: None,
        length: 0,
        members: HashMap::<CoherentTriMember, MemberAllocation>::new(),
    }
}

fn set_member_as_singleton(ring: &mut CoherentTriPtr, member: CoherentTriMember) {
    ring.members.get_mut(&member).unwrap().member.next = member;
    ring.members.get_mut(&member).unwrap().member.previous = member;
    ring.head = Some(member);
    ring.length = 1;
}
