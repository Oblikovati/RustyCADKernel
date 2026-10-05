//! Shared owning handles and non-owning projected views preserve one object identity.
//!
//! `T` is a concrete class (a struct, reached through its embedded subobjects) or
//! an abstract class written as a trait (`Handle<dyn Trait>`, reached through the
//! interface casters of the dynamic descriptor).
use super::{RuntimeType, RuntimeTyped, Transient};
use std::{any::TypeId, marker::PhantomData, sync::Arc};
mod casts;
mod traits;
mod view;
mod weak;
pub use view::HandleView;
pub use weak::WeakHandle;
/// Nullable shared ownership with runtime checked base projection; `Handle::new(value)`.
pub struct Handle<T: ?Sized + RuntimeTyped> {
    owner: Option<Owner>,
    projected: PhantomData<T>,
}
/// Keeps either heap ownership or a process-lifetime descriptor alive.
#[derive(Clone)]
pub(super) enum Owner {
    Shared(Arc<dyn Transient>),
    Static(&'static dyn Transient),
}
impl Owner {
    pub(super) fn get(&self) -> &dyn Transient {
        match self {
            Self::Shared(owner) => owner.as_ref(),
            Self::Static(owner) => *owner,
        }
    }
    pub(super) fn address(&self) -> *const () {
        std::ptr::from_ref(self.get()).cast::<()>()
    }
}
impl<T: ?Sized + RuntimeTyped> Default for Handle<T> {
    fn default() -> Self {
        Self {
            owner: None,
            projected: PhantomData,
        }
    }
}
impl<T: RuntimeTyped> Handle<T> {
    /// Create shared ownership of a fresh concrete object; `Handle::new(StandardTransient::default())`.
    pub fn new(value: T) -> Self
    where
        T: Transient,
    {
        let owner: Arc<dyn Transient> = Arc::new(value);
        Self::adopt(Some(&Owner::Shared(owner)))
    }
    /// Own a static transient descriptor while preserving its address; `Handle::from_static(RuntimeType::static_type())`.
    pub fn from_static(value: &'static T) -> Self
    where
        T: Transient,
    {
        Self::adopt(Some(&Owner::Static(value)))
    }
    /// Observe the typed address without transferring ownership; `handle.as_ptr()`.
    pub fn as_ptr(&self) -> *const T {
        self.get().map_or(std::ptr::null(), std::ptr::from_ref)
    }
}
impl<T: ?Sized + RuntimeTyped> Handle<T> {
    /// Share `owner` when it projects to `T` (one more reference), else null.
    pub(super) fn adopt(owner: Option<&Owner>) -> Self {
        let Some(owner) = owner.filter(|owner| T::project_from(owner.get()).is_some()) else {
            return Self::default();
        };
        owner.get().transient_base().increment_ref_counter();
        Self {
            owner: Some(owner.clone()),
            projected: PhantomData,
        }
    }
    /// Clone shared storage without changing the embedded OCCT reference count.
    pub(super) fn clone_storage_owner(&self) -> Option<Arc<dyn Transient>> {
        match self.owner.as_ref()? {
            Owner::Shared(owner) => Some(Arc::clone(owner)),
            Owner::Static(_) => None,
        }
    }
    /// Check for absent ownership; `handle.is_null()`.
    pub fn is_null(&self) -> bool {
        self.owner.is_none()
    }
    /// Rust's explicit boolean conversion; `handle.is_some()`.
    pub fn is_some(&self) -> bool {
        self.owner.is_some()
    }
    /// Borrow the actual projected subobject or interface view; `handle.get()`.
    pub fn get(&self) -> Option<&T> {
        T::project_from(self.owner.as_ref()?.get())
    }
    /// Observe the object address, independent of the projected type; `handle.object_address()`.
    ///
    /// Two handles to one object give the same address, whatever their `T`.
    pub fn object_address(&self) -> *const () {
        self.owner.as_ref().map_or(std::ptr::null(), Owner::address)
    }
    /// Cast ownership to a supported base/derived type, returning null on failure; `handle.cast::<Base>()`.
    pub fn cast<U: ?Sized + RuntimeTyped>(&self) -> Handle<U> {
        Handle::adopt(self.owner.as_ref())
    }
    /// Borrow a projected view without incrementing the count; `handle.view::<Base>()`.
    pub fn view<U: ?Sized + RuntimeTyped>(&self) -> HandleView<'_, U> {
        HandleView::new(self.owner.as_ref())
    }
    /// Dispatch the original object's dynamic descriptor; `handle.dynamic_type()`.
    pub fn dynamic_type(&self) -> Option<&'static RuntimeType> {
        self.owner.as_ref().map(|owner| owner.get().dynamic_type())
    }
    /// Own the dynamic descriptor; `handle.dynamic_type_handle().name()`.
    pub fn dynamic_type_handle(&self) -> Option<Handle<RuntimeType>> {
        self.owner
            .as_ref()
            .map(|owner| owner.get().dynamic_type_handle())
    }
    /// Read concrete Rust RTTI independently of OCCT descriptor inheritance; `handle.rust_type_id()`.
    pub fn rust_type_id(&self) -> Option<TypeId> {
        self.owner.as_ref().map(|owner| owner.get().type_id())
    }
    /// Check dynamic ancestry, false for null; `handle.is_kind(Base::static_type())`.
    pub fn is_kind(&self, base: &RuntimeType) -> bool {
        self.owner
            .as_ref()
            .is_some_and(|owner| owner.get().is_kind(base))
    }
    /// Check dynamic ancestry by name, false for null; `handle.is_kind_name("Standard_Transient")`.
    pub fn is_kind_name(&self, name: &str) -> bool {
        self.owner
            .as_ref()
            .is_some_and(|owner| owner.get().is_kind_name(name))
    }
    /// Check exact registered type, false for null; `handle.is_instance(Derived::static_type())`.
    pub fn is_instance(&self, exact: &RuntimeType) -> bool {
        self.owner
            .as_ref()
            .is_some_and(|owner| owner.get().is_instance(exact))
    }
    /// Check exact native name, false for null; `handle.is_instance_name("Derived")`.
    pub fn is_instance_name(&self, name: &str) -> bool {
        self.owner
            .as_ref()
            .is_some_and(|owner| owner.get().is_instance_name(name))
    }
    /// Read the embedded counter; `handle.ref_count()` is zero for null handles.
    pub fn ref_count(&self) -> i32 {
        self.owner
            .as_ref()
            .map_or(0, |owner| owner.get().transient_base().ref_count())
    }
    /// Release ownership and become null; `handle.nullify()`.
    pub fn nullify(&mut self) {
        *self = Self::default();
    }
    /// Address of the owned object, 0 for null: the identity `<` and the hash compare.
    fn address(&self) -> usize {
        self.object_address() as usize
    }
}
impl<T: ?Sized + RuntimeTyped> Clone for Handle<T> {
    fn clone(&self) -> Self {
        self.cast::<T>()
    }
    /// Assignment keeps the count untouched when both handles already share the object.
    fn clone_from(&mut self, source: &Self) {
        if *self != *source {
            *self = source.clone();
        }
    }
}
impl<T: ?Sized + RuntimeTyped> Drop for Handle<T> {
    fn drop(&mut self) {
        if let Some(owner) = &self.owner {
            owner.get().transient_base().decrement_ref_counter();
        }
    }
}
