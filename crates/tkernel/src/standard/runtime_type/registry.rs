use super::{InterfaceEntry, RuntimeType, RuntimeTyped};
use crate::standard::{Handle, Transient, WeakHandle};
use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock, PoisonError},
};

type Registry = Mutex<HashMap<TypeId, RegistryEntry>>;
enum RegistryEntry {
    Owned(WeakHandle<RuntimeType>),
    LegacyBorrowed(&'static RuntimeType),
}
type StaticOwners = Mutex<HashMap<TypeId, &'static StaticDescriptorOwner>>;
struct StaticDescriptorOwner {
    class_handle: OnceLock<StaticClassHandle>,
    storage: OnceLock<Arc<dyn Transient>>,
}
enum StaticClassHandle {
    Owned(Handle<RuntimeType>),
}
impl StaticDescriptorOwner {
    fn new() -> Self {
        Self {
            class_handle: OnceLock::new(),
            storage: OnceLock::new(),
        }
    }
    fn descriptor(&'static self) -> &'static RuntimeType {
        if let Some(handle) = self.class_handle.get() {
            return handle.descriptor();
        }
        RuntimeType::project_from(self.storage.get().expect("pinned descriptor").as_ref())
            .expect("stored runtime descriptor")
    }
}
impl StaticClassHandle {
    fn descriptor(&'static self) -> &'static RuntimeType {
        match self {
            Self::Owned(handle) => handle.get().expect("owned class descriptor"),
        }
    }
}
fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(Registry::default)
}
fn static_owners() -> &'static StaticOwners {
    static OWNERS: OnceLock<StaticOwners> = OnceLock::new();
    OWNERS.get_or_init(StaticOwners::default)
}
fn static_owner<T: ?Sized + 'static>() -> &'static StaticDescriptorOwner {
    let mut owners = static_owners()
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some(owner) = owners.get(&TypeId::of::<T>()) {
        return owner;
    }
    let owner = Box::leak(Box::new(StaticDescriptorOwner::new()));
    owners.insert(TypeId::of::<T>(), owner);
    owner
}
pub(super) fn register_owned<T: ?Sized + 'static>(
    name: &str,
    size: usize,
    mut parent: Option<Handle<RuntimeType>>,
    interfaces: Vec<InterfaceEntry>,
) -> Handle<RuntimeType> {
    let type_id = TypeId::of::<T>();
    let mut entries = registry().lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(existing) = owned_or_legacy(type_id, &entries) {
        drop(entries);
        drop(parent);
        return existing;
    }
    let candidate = Handle::new(RuntimeType::new::<T>(name, size, parent.take(), interfaces));
    entries.insert(type_id, RegistryEntry::Owned(candidate.downgrade()));
    candidate
}
pub(super) fn register_legacy_borrowed<T: ?Sized + 'static>(
    name: &str,
    size: usize,
    parent: Option<&'static RuntimeType>,
    interfaces: Vec<InterfaceEntry>,
) -> &'static RuntimeType {
    let type_id = TypeId::of::<T>();
    let mut entries = registry().lock().unwrap_or_else(PoisonError::into_inner);
    let existing = legacy_or_owned(type_id, &entries);
    if let Some(existing) = existing {
        drop(entries);
        drop(interfaces);
        return resolve_legacy_existing::<T>(existing);
    }
    create_legacy::<T>(type_id, name, size, parent, interfaces, &mut entries)
}
fn resolve_legacy_existing<T: ?Sized + 'static>(existing: LegacyExisting) -> &'static RuntimeType {
    match existing {
        LegacyExisting::Borrowed(descriptor) => descriptor,
        LegacyExisting::Owned(owner) => retain_storage_owner::<T>(owner),
    }
}
enum LegacyExisting {
    Borrowed(&'static RuntimeType),
    Owned(Handle<RuntimeType>),
}
fn legacy_or_owned(
    type_id: TypeId,
    entries: &HashMap<TypeId, RegistryEntry>,
) -> Option<LegacyExisting> {
    match entries.get(&type_id)? {
        RegistryEntry::LegacyBorrowed(descriptor) => Some(LegacyExisting::Borrowed(descriptor)),
        RegistryEntry::Owned(weak) => {
            let owner = weak.upgrade();
            (!owner.is_null()).then_some(LegacyExisting::Owned(owner))
        }
    }
}
fn create_legacy<T: ?Sized + 'static>(
    type_id: TypeId,
    name: &str,
    size: usize,
    parent: Option<&'static RuntimeType>,
    interfaces: Vec<InterfaceEntry>,
    entries: &mut HashMap<TypeId, RegistryEntry>,
) -> &'static RuntimeType {
    let descriptor = Box::leak(Box::new(RuntimeType::new::<T>(
        name,
        size,
        parent.map(Handle::from_static),
        interfaces,
    )));
    entries.insert(type_id, RegistryEntry::LegacyBorrowed(descriptor));
    descriptor
}
fn owned_or_legacy(
    type_id: TypeId,
    entries: &HashMap<TypeId, RegistryEntry>,
) -> Option<Handle<RuntimeType>> {
    match entries.get(&type_id)? {
        RegistryEntry::LegacyBorrowed(descriptor) => Some(Handle::from_static(*descriptor)),
        RegistryEntry::Owned(weak) => {
            let handle = weak.upgrade();
            (!handle.is_null()).then_some(handle)
        }
    }
}
fn retain_storage_owner<T: ?Sized + 'static>(handle: Handle<RuntimeType>) -> &'static RuntimeType {
    let owner = static_owner::<T>();
    if owner.class_handle.get().is_none() {
        let storage = handle
            .clone_storage_owner()
            .expect("owned descriptor storage");
        let _ = owner.storage.set(storage);
    }
    drop(handle);
    owner.descriptor()
}
pub(super) fn retain_class_handle<T: ?Sized + 'static>(
    handle: Handle<RuntimeType>,
) -> &'static RuntimeType {
    let owner = static_owner::<T>();
    let _ = owner.class_handle.set(StaticClassHandle::Owned(handle));
    owner.descriptor()
}
pub(super) fn retain_class_descriptor<T: ?Sized + 'static>(
    descriptor: &'static RuntimeType,
) -> &'static RuntimeType {
    let handle = registered_handle::<T>()
        .unwrap_or_else(|| panic!("no registered descriptor for {}", descriptor.name()));
    assert!(std::ptr::eq(
        handle.get().expect("registered descriptor"),
        descriptor
    ));
    retain_class_handle::<T>(handle)
}
fn registered_handle<T: ?Sized + 'static>() -> Option<Handle<RuntimeType>> {
    let entries = registry().lock().unwrap_or_else(PoisonError::into_inner);
    match entries.get(&TypeId::of::<T>())? {
        RegistryEntry::LegacyBorrowed(descriptor) => Some(Handle::from_static(*descriptor)),
        RegistryEntry::Owned(weak) => {
            let handle = weak.upgrade();
            (!handle.is_null()).then_some(handle)
        }
    }
}
pub(super) fn remove_owned(type_id: TypeId, address: *const ()) {
    let mut entries = registry().lock().unwrap_or_else(PoisonError::into_inner);
    let matches = match entries.get(&type_id) {
        Some(RegistryEntry::Owned(weak)) => weak.object_address() == address,
        _ => false,
    };
    if matches {
        entries.remove(&type_id);
    }
}
