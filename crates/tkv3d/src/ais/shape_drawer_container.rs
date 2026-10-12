//! Const/mutable container adapters borrow one actual native map member without copying nodes.
use super::{AisColoredDrawer, AisShapeDrawerMap, AisShapeDrawerNodeRef, AisShapeDrawerValueMut};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
/// Live const map view; `let alias=map.as_const(); map.clear(true); assert!(alias.is_empty())`.
#[derive(Clone, Copy)]
pub struct AisShapeDrawerMapRef<'a> {
    owner: &'a AisShapeDrawerMap,
}
/// Live mutable map view; `map.as_mut().bind(&shape, &drawer)?`.
#[derive(Clone, Copy)]
pub struct AisShapeDrawerMapMut<'a> {
    owner: &'a AisShapeDrawerMap,
}
impl AisShapeDrawerMap {
    /// Borrows the owned container itself, not a snapshot; `map.as_const().len()`.
    pub fn as_const(&self) -> AisShapeDrawerMapRef<'_> {
        // Native CustomAspectsMap returns the same map member across all lifecycle transitions.
        AisShapeDrawerMapRef { owner: self }
    }
    /// Borrows the actual mutable container without retaining a lock; `map.as_mut().clear(true)`.
    pub fn as_mut(&self) -> AisShapeDrawerMapMut<'_> {
        // Native ChangeCustomAspectsMap may coexist with retained const references.
        AisShapeDrawerMapMut { owner: self }
    }
}
impl<'a> AisShapeDrawerMapRef<'a> {
    /// Reads the live extent; `reference.len()`.
    pub fn len(&self) -> usize {
        let owner = self.owner;
        owner.len()
    }
    /// Reads live emptiness; `reference.is_empty()`.
    pub fn is_empty(&self) -> bool {
        let owner = self.owner;
        owner.is_empty()
    }
    /// Compares actual container identity; `left.is_same_reference(&right)`.
    pub fn is_same_reference(&self, other: &Self) -> bool {
        let owner = self.owner;
        std::ptr::eq(owner, other.owner)
    }
    /// Looks up a live native node; `reference.node_ref(&shape)?`.
    pub fn node_ref(&self, key: &Shape) -> Result<AisShapeDrawerNodeRef<'a>, Failure> {
        let owner = self.owner;
        owner.node_ref(key)
    }
}
impl<'a> AisShapeDrawerMapMut<'a> {
    /// Reads the same container through a const view; `reference.as_const().len()`.
    pub fn as_const(&self) -> AisShapeDrawerMapRef<'a> {
        let owner = self.owner;
        AisShapeDrawerMapRef { owner }
    }
    /// Assigns a native key/value pair; `reference.bind(&shape, &drawer)?`.
    pub fn bind(&self, key: &Shape, value: &Handle<AisColoredDrawer>) -> Result<bool, Failure> {
        let owner = self.owner;
        owner.bind(key, value)
    }
    /// Borrows the actual mutable value slot; `reference.change_value_ref(&shape)?.assign(&drawer)?`.
    pub fn change_value_ref(&self, key: &Shape) -> Result<AisShapeDrawerValueMut<'a>, Failure> {
        let owner = self.owner;
        owner.change_value_ref(key)
    }
    /// Removes a key and releases node ownership; `reference.unbind(&shape)`.
    pub fn unbind(&self, key: &Shape) -> bool {
        let owner = self.owner;
        owner.unbind(key)
    }
    /// Clears the actual container; `reference.clear(false)`.
    pub fn clear(&self, release: bool) {
        let owner = self.owner;
        owner.clear(release);
    }
}
