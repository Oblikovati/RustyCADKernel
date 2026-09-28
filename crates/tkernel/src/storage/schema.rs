// Copyright (c) 1996-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! `Storage_Schema`: the storage algorithm and its callback registry.
//!
//! The native `ICurrentData` is a process-wide static that `Write` sets and clears and that
//! `AddPersistent`, `PersistentToAdd` and the type bindings read. Rust keeps that slot per
//! schema, so two schemas no longer share it; within one schema the lifetime is the same.
use super::{CallBack, Data, DefaultCallBack, InternalData, TypedCallBack};
use crate::ncollection::{DataMap, HSequence};
use crate::standard::{Failure, Handle, StandardTransient};
use crate::tcollection::AsciiString;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
mod binding;
mod creation_date;
mod migration;
mod write;
pub use creation_date::{CreationDateSource, NativeCreationDate};
pub use migration::{MigrationFile, MigrationSource, NativeMigrationSource, TypeMigration};

/// A storage schema; `Schema::default()` uses the local clock for creation dates.
///
/// ```
/// use rustycad_tkernel::storage::Schema;
/// use rustycad_tkernel::tcollection::AsciiString;
/// let mut schema = Schema::default();
/// schema.set_name(&AsciiString::new("Test"));
/// assert!(!schema.is_using_default_callback());
/// ```
pub struct Schema {
    parent: StandardTransient,
    callbacks: DataMap<AsciiString, Handle<TypedCallBack>>,
    callback_state: bool,
    default_callback: Option<Arc<dyn CallBack>>,
    name: AsciiString,
    version: AsciiString,
    current: Mutex<Handle<Data>>,
    clock: Arc<dyn CreationDateSource>,
}

impl Default for Schema {
    fn default() -> Self {
        Self::with_clock(Arc::new(NativeCreationDate))
    }
}

impl Schema {
    /// `Storage_Schema()` with an injected creation-date clock; `Schema::with_clock(clock)`.
    pub fn with_clock(clock: Arc<dyn CreationDateSource>) -> Self {
        let mut schema = Self {
            parent: StandardTransient::default(),
            callbacks: DataMap::default(),
            callback_state: false,
            default_callback: None,
            name: AsciiString::default(),
            version: AsciiString::default(),
            current: Mutex::new(Handle::default()),
            clock,
        };
        schema.clear();
        schema.reset_default_callback();
        schema
    }

    /// `SetVersion`; `schema.set_version(&AsciiString::new("1"))`.
    pub fn set_version(&mut self, version: &AsciiString) {
        self.version = version.clone();
    }

    /// `Version`; `schema.version()`.
    pub fn version(&self) -> AsciiString {
        self.version.clone()
    }

    /// `SetName`; `schema.set_name(&AsciiString::new("Schema"))`.
    pub fn set_name(&mut self, name: &AsciiString) {
        self.name = name.clone();
    }

    /// `Name`; `schema.name()`.
    pub fn name(&self) -> AsciiString {
        self.name.clone()
    }

    /// `ICreationDate`: the local date as `%m/%d/%Y`; `Schema::i_creation_date()`.
    pub fn i_creation_date() -> AsciiString {
        NativeCreationDate.creation_date()
    }

    /// `CheckTypeMigration` with the process-wide table loaded once from
    /// `CSF_MIGRATION_TYPES`; `Schema::check_type_migration(&old, &mut new)`.
    pub fn check_type_migration(old_name: &AsciiString, new_name: &mut AsciiString) -> bool {
        TypeMigration::process_wide().check(old_name, new_name)
    }
}

impl Schema {
    /// `AddReadUnknownTypeCallBack`: binds `callback` to `type_name`; a null callback is
    /// ignored; `schema.add_read_unknown_type_callback(&name, Some(callback))`.
    pub fn add_read_unknown_type_callback(
        &mut self,
        type_name: &AsciiString,
        callback: Option<Arc<dyn CallBack>>,
    ) {
        if callback.is_some() {
            let typed = Handle::new(TypedCallBack::new(type_name, callback));
            self.callbacks.bind(type_name.clone(), typed);
        }
    }

    /// `RemoveReadUnknownTypeCallBack`; `schema.remove_read_unknown_type_callback(&name)`.
    pub fn remove_read_unknown_type_callback(&mut self, type_name: &AsciiString) {
        self.callbacks.unbind(type_name);
    }

    /// `InstalledCallBackList`: type names in map iteration order; `schema.installed_callback_list()`.
    pub fn installed_callback_list(&self) -> Handle<HSequence<AsciiString>> {
        let names = HSequence::default();
        for (name, _) in self.callbacks.items() {
            names.append(name.clone());
        }
        Handle::new(names)
    }

    /// `ClearCallBackList`; `schema.clear_callback_list()`.
    pub fn clear_callback_list(&mut self) {
        self.callbacks.clear(false);
    }

    /// `UseDefaultCallBack`; `schema.use_default_callback()`.
    pub fn use_default_callback(&mut self) {
        self.callback_state = true;
    }

    /// `DontUseDefaultCallBack`; `schema.dont_use_default_callback()`.
    pub fn dont_use_default_callback(&mut self) {
        self.callback_state = false;
    }

    /// `IsUsingDefaultCallBack`; `schema.is_using_default_callback()`.
    pub fn is_using_default_callback(&self) -> bool {
        self.callback_state
    }

    /// `SetDefaultCallBack`; `schema.set_default_callback(Some(callback))`.
    pub fn set_default_callback(&mut self, callback: Option<Arc<dyn CallBack>>) {
        self.default_callback = callback;
    }

    /// `ResetDefaultCallBack`: back to a fresh `Storage_DefaultCallBack`.
    /// Example: `schema.reset_default_callback()`.
    pub fn reset_default_callback(&mut self) {
        self.default_callback = Some(Arc::new(DefaultCallBack));
    }

    /// `DefaultCallBack`; `schema.default_callback()`.
    pub fn default_callback(&self) -> Option<Arc<dyn CallBack>> {
        self.default_callback.clone()
    }
}

impl Schema {
    /// `ICurrentData()`.
    fn current(&self) -> MutexGuard<'_, Handle<Data>> {
        self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Runs `action` on the current data set and its internal data. The native code
    /// dereferences a null current data set; Rust reports `Standard_NullObject`.
    fn with_current<R>(
        &self,
        member: &str,
        action: impl FnOnce(&Data, &InternalData) -> R,
    ) -> Result<R, Failure> {
        let current = self.current().clone();
        let data = current.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                &format!("Storage_Schema::{member}: no current data; expected a Write in progress"),
                "",
            )
        })?;
        let internal = data.internal_data();
        let internal = internal.get().expect("Storage_Data owns its internal data");
        Ok(action(data, internal))
    }

    /// Private `Clear`: forgets the current data set.
    fn clear(&self) {
        self.current().nullify();
    }
}

crate::impl_transient!(Schema, "Storage_Schema", StandardTransient, parent);
