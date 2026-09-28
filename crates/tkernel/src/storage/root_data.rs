// Copyright (c) 1997-1999 Matra Datavision
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

//! `Storage_RootData`: the root section, a map from root names to roots.
use super::{BaseDriver, OpenMode, Root, SectionStatus, StorageError, StreamException};
use crate::ncollection::{DataMap, HSequence};
use crate::standard::{Failure, Handle, StandardPersistent, StandardTransient};
use crate::tcollection::AsciiString;
use std::sync::{Mutex, MutexGuard, PoisonError};

#[derive(Default)]
struct RootFields {
    objects: DataMap<AsciiString, Handle<Root>>,
    status: SectionStatus,
}

/// Named roots of a set of data; `Roots()` walks the map in its native bucket order.
///
/// ```
/// use rustycad_tkernel::standard::{Handle, StandardPersistent};
/// use rustycad_tkernel::storage::{Root, RootData};
/// use rustycad_tkernel::tcollection::AsciiString;
/// let roots = RootData::default();
/// let name = AsciiString::new("doc");
/// roots.add_root(&Handle::new(Root::new(&name, &Handle::<StandardPersistent>::default())));
/// assert!(roots.is_root(&name));
/// ```
#[derive(Default)]
pub struct RootData {
    parent: StandardTransient,
    fields: Mutex<RootFields>,
}

impl RootData {
    fn lock(&self) -> MutexGuard<'_, RootFields> {
        self.fields.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// `Read(theDriver)`: binds one retrieved root per entry of the root section. A
    /// type mismatch in `ReadRoot` becomes the status; `RootSectionSize` failures propagate.
    /// Example: `roots.read(&mut driver)?`.
    pub fn read(&self, driver: &mut dyn BaseDriver) -> Result<bool, Failure> {
        if !matches!(driver.open_mode(), OpenMode::Read | OpenMode::ReadWrite) {
            return Ok(self.record(StorageError::ModeError, "OpenMode"));
        }
        let status = driver.begin_read_root_section()?;
        if !self.record(status, "BeginReadRootSection") {
            return Ok(false);
        }
        let (mut name, mut type_name, mut reference) = (AsciiString::default(), AsciiString::default(), 0);
        let length = driver.root_section_size()?;
        for _ in 1..=length {
            match driver.read_root(&mut name, &mut reference, &mut type_name) {
                Err(failure) if StreamException::TypeMismatch.is(&failure) => {
                    return Ok(self.record(StorageError::TypeMismatch, "ReadRoot"));
                }
                outcome => outcome?,
            }
            let root = Handle::new(Root::with_reference(&name, reference, &type_name));
            self.lock().objects.bind(name.clone(), root);
        }
        let status = driver.end_read_root_section()?;
        Ok(self.record(status, "EndReadRootSection"))
    }

    fn record(&self, error: StorageError, context: &str) -> bool {
        self.lock().status.record(error, context)
    }

    /// `NumberOfRoots`; `roots.number_of_roots()`.
    pub fn number_of_roots(&self) -> i32 {
        self.lock().objects.extent() as i32
    }

    /// `AddRoot`: binds (or rebinds) the root under its name; `roots.add_root(&root)`.
    pub fn add_root(&self, root: &Handle<Root>) {
        let name = root.get().map(Root::name).unwrap_or_default();
        self.lock().objects.bind(name, root.clone());
    }

    /// `Roots`: a new sequence in map iteration order; `roots.roots()`.
    pub fn roots(&self) -> Handle<HSequence<Handle<Root>>> {
        let sequence = HSequence::default();
        for root in self.lock().objects.values() {
            sequence.append(root.clone());
        }
        Handle::new(sequence)
    }

    /// `Find`: the root named `name`, or a null handle; `roots.find(&name)`.
    pub fn find(&self, name: &AsciiString) -> Handle<Root> {
        self.lock().objects.seek(name).cloned().unwrap_or_default()
    }

    /// `IsRoot`; `roots.is_root(&name)`.
    pub fn is_root(&self, name: &AsciiString) -> bool {
        self.lock().objects.is_bound(name)
    }

    /// `RemoveRoot`: unbinds the name when bound; `roots.remove_root(&name)`.
    pub fn remove_root(&self, name: &AsciiString) {
        self.lock().objects.unbind(name);
    }

    /// `UpdateRoot`: sets the object of the bound root, which every handle holder sees;
    /// an unknown name is `Standard_NoSuchObject`; `roots.update_root(&name, &object)?`.
    pub fn update_root(
        &self,
        name: &AsciiString,
        object: &Handle<StandardPersistent>,
    ) -> Result<(), Failure> {
        let root = self.find(name);
        let Some(root) = root.get() else {
            return Err(Failure::with_type(
                "Standard_NoSuchObject",
                &format!(
                    "Storage_RootData::UpdateRoot {:?}; expected a bound root name",
                    String::from_utf8_lossy(name.as_bytes())
                ),
                "",
            ));
        };
        root.set_object(object);
        Ok(())
    }

    /// `ErrorStatus`; `roots.error_status()`.
    pub fn error_status(&self) -> StorageError {
        self.lock().status.error
    }

    /// `ErrorStatusExtension`; `roots.error_status_extension()`.
    pub fn error_status_extension(&self) -> AsciiString {
        self.lock().status.extension.clone()
    }

    /// `ClearErrorStatus`; `roots.clear_error_status()`.
    pub fn clear_error_status(&self) {
        self.lock().status.clear();
    }
}

crate::impl_transient!(RootData, "Storage_RootData", StandardTransient, parent);
