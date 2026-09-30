//! A value made shareable by handle (`NCollection_Shared<T>`: `Standard_Transient` + `T`).
use crate::standard::{RuntimeType, RuntimeTyped, StandardTransient, Transient};
use std::any::{Any, TypeId};

/// `T` with a transient reference count, so it can live in a `Handle`; `Handle::new(Shared::new(v))`.
///
/// Upstream inherits from `T`; Rust embeds it and dereferences to it. The class declares no
/// RTTI of its own, so its dynamic type is `Standard_Transient`. The upstream guard against a
/// `T` that already is a transient cannot be expressed; such a `T` simply nests its own count.
#[repr(C)]
pub struct Shared<T> {
    parent: StandardTransient,
    value: T,
}

impl<T> Shared<T> {
    /// Wraps a constructed value; the native forwarding constructors (`NCollection_Shared(a)`,
    /// `NCollection_Shared(a, b)`) build `T` from their arguments, which Rust does at the call
    /// site. Example: `Shared::new(vec![0; 3]).len() == 3`.
    pub fn new(value: T) -> Self {
        Self {
            parent: StandardTransient::default(),
            value,
        }
    }

    /// Unwraps the value; `Shared::new(4).into_inner() == 4`.
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: Default> Default for Shared<T> {
    /// `NCollection_Shared()`: a default-constructed `T`.
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> std::ops::Deref for Shared<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: Send + Sync + 'static> RuntimeTyped for Shared<T> {
    // No RTTI of its own upstream: it reports Standard_Transient's name and descriptor.
    const TYPE_NAME: &'static str = StandardTransient::TYPE_NAME;
    fn static_type() -> &'static RuntimeType {
        StandardTransient::static_type() // no RTTI macro upstream
    }
    fn project_from(object: &dyn crate::standard::Transient) -> Option<&Self> {
        crate::standard::project_concrete::<Self>(object)
    }
}

impl<T: Send + Sync + 'static> Transient for Shared<T> {
    fn dynamic_type(&self) -> &'static RuntimeType {
        StandardTransient::static_type()
    }
    fn project(&self, requested: TypeId) -> Option<&dyn Any> {
        if requested == TypeId::of::<Self>() {
            return Some(self);
        }
        self.parent.project(requested)
    }
    fn transient_base(&self) -> &StandardTransient {
        &self.parent
    }
}
