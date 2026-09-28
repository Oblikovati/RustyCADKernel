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

//! `Storage_Data`: the picture of the data stored in, or read from, a container.
use super::{HeaderData, InternalData, Root, RootData, SectionStatus, StorageError, TypeData};
use crate::ncollection::{HSequence, Sequence};
use crate::standard::{Handle, StandardPersistent, StandardTransient};
use crate::tcollection::{AsciiString, ExtendedString};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// Roots, types, header and bookkeeping of one container, shared by handles.
///
/// The native constructor leaves `myErrorStatus` uninitialised; Rust starts at `Ok`.
///
/// ```
/// use rustycad_tkernel::standard::{Handle, StandardPersistent};
/// use rustycad_tkernel::storage::Data;
/// use rustycad_tkernel::tcollection::AsciiString;
/// let data = Data::default();
/// data.add_root(&Handle::new(StandardPersistent::default()));
/// assert!(data.is_root(&AsciiString::new("1")));
/// ```
pub struct Data {
    parent: StandardTransient,
    header: Handle<HeaderData>,
    roots: Handle<RootData>,
    types: Handle<TypeData>,
    internal: Handle<InternalData>,
    status: Mutex<SectionStatus>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            roots: Handle::new(RootData::default()),
            types: Handle::new(TypeData::default()),
            internal: Handle::new(InternalData::default()),
            header: Handle::new(HeaderData::default()),
            status: Mutex::new(SectionStatus::default()),
        }
    }
}

impl Data {
    fn header(&self) -> &HeaderData {
        self.header.get().expect("Storage_Data owns its header")
    }

    fn root_section(&self) -> &RootData {
        self.roots.get().expect("Storage_Data owns its roots")
    }

    fn type_section(&self) -> &TypeData {
        self.types.get().expect("Storage_Data owns its types")
    }

    fn status(&self) -> MutexGuard<'_, SectionStatus> {
        self.status.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// `ErrorStatus` of the last `Storage_Schema` read or write; `data.error_status()`.
    pub fn error_status(&self) -> StorageError {
        self.status().error
    }

    /// `ClearErrorStatus`: clears this, the header, root and type statuses.
    /// Example: `data.clear_error_status()`.
    pub fn clear_error_status(&self) {
        self.status().clear();
        self.header().clear_error_status();
        self.root_section().clear_error_status();
        self.type_section().clear_error_status();
    }

    /// `ErrorStatusExtension`; `data.error_status_extension()`.
    pub fn error_status_extension(&self) -> AsciiString {
        self.status().extension.clone()
    }

    /// Private `SetErrorStatus` and `SetErrorStatusExtension` of the friend `Storage_Schema`.
    pub(crate) fn set_status(&self, error: StorageError, extension: &str) {
        let mut status = self.status();
        status.error = error;
        status.extension = AsciiString::new(extension);
    }

    /// `HeaderData`; `data.header_data()`.
    pub fn header_data(&self) -> Handle<HeaderData> {
        self.header.clone()
    }

    /// `RootData`; `data.root_data()`.
    pub fn root_data(&self) -> Handle<RootData> {
        self.roots.clone()
    }

    /// `TypeData`; `data.type_data()`.
    pub fn type_data(&self) -> Handle<TypeData> {
        self.types.clone()
    }

    /// `InternalData`; `data.internal_data()`.
    pub fn internal_data(&self) -> Handle<InternalData> {
        self.internal.clone()
    }

    /// `Clear`: resets the bookkeeping and the type section; `data.clear()`.
    pub fn clear(&self) {
        if let Some(internal) = self.internal.get() {
            internal.clear();
        }
        self.type_section().clear();
    }
}

impl Data {
    /// `AddRoot(anObject)`: named by the next root number; `data.add_root(&object)`.
    pub fn add_root(&self, object: &Handle<StandardPersistent>) {
        let name = AsciiString::from_integer(self.root_section().number_of_roots() + 1);
        self.add_named_root(&name, object);
    }

    /// `AddRoot(aName, anObject)`; `data.add_named_root(&name, &object)`.
    pub fn add_named_root(&self, name: &AsciiString, object: &Handle<StandardPersistent>) {
        self.root_section()
            .add_root(&Handle::new(Root::new(name, object)));
    }

    /// `RemoveRoot`: nothing happens for an unknown name; `data.remove_root(&name)`.
    pub fn remove_root(&self, name: &AsciiString) {
        self.root_section().remove_root(name);
    }

    /// `Roots`; `data.roots()`.
    pub fn roots(&self) -> Handle<HSequence<Handle<Root>>> {
        self.root_section().roots()
    }

    /// `Find`: the named root or a null handle; `data.find(&name)`.
    pub fn find(&self, name: &AsciiString) -> Handle<Root> {
        self.root_section().find(name)
    }

    /// `NumberOfRoots`; `data.number_of_roots()`.
    pub fn number_of_roots(&self) -> i32 {
        self.root_section().number_of_roots()
    }

    /// `IsRoot`; `data.is_root(&name)`.
    pub fn is_root(&self, name: &AsciiString) -> bool {
        self.root_section().is_root(name)
    }

    /// `NumberOfTypes`; `data.number_of_types()`.
    pub fn number_of_types(&self) -> i32 {
        self.type_section().number_of_types()
    }

    /// `IsType`; `data.is_type(&name)`.
    pub fn is_type(&self, name: &AsciiString) -> bool {
        self.type_section().is_type(name)
    }

    /// `Types`; `data.types()`.
    pub fn types(&self) -> Handle<HSequence<AsciiString>> {
        self.type_section().types()
    }
}

impl Data {
    /// `CreationDate`; `data.creation_date()`.
    pub fn creation_date(&self) -> AsciiString {
        self.header().creation_date()
    }

    /// `StorageVersion`; `data.storage_version()`.
    pub fn storage_version(&self) -> AsciiString {
        self.header().storage_version()
    }

    /// `SchemaVersion`; `data.schema_version()`.
    pub fn schema_version(&self) -> AsciiString {
        self.header().schema_version()
    }

    /// `SchemaName`; `data.schema_name()`.
    pub fn schema_name(&self) -> AsciiString {
        self.header().schema_name()
    }

    /// `SetApplicationVersion`; `data.set_application_version(&version)`.
    pub fn set_application_version(&self, version: &AsciiString) {
        self.header().set_application_version(version);
    }

    /// `ApplicationVersion`; `data.application_version()`.
    pub fn application_version(&self) -> AsciiString {
        self.header().application_version()
    }

    /// `SetApplicationName`; `data.set_application_name(&name)`.
    pub fn set_application_name(&self, name: &ExtendedString) {
        self.header().set_application_name(name);
    }

    /// `ApplicationName`; `data.application_name()`.
    pub fn application_name(&self) -> ExtendedString {
        self.header().application_name()
    }

    /// `SetDataType`; `data.set_data_type(&data_type)`.
    pub fn set_data_type(&self, data_type: &ExtendedString) {
        self.header().set_data_type(data_type);
    }

    /// `DataType`; `data.data_type()`.
    pub fn data_type(&self) -> ExtendedString {
        self.header().data_type()
    }

    /// `AddToUserInfo`; `data.add_to_user_info(&line)`.
    pub fn add_to_user_info(&self, info: &AsciiString) {
        self.header().add_to_user_info(info);
    }

    /// `UserInfo`; `data.user_info()`.
    pub fn user_info(&self) -> Sequence<AsciiString> {
        self.header().user_info()
    }

    /// `AddToComments`; `data.add_to_comments(&comment)`.
    pub fn add_to_comments(&self, comment: &ExtendedString) {
        self.header().add_to_comments(comment);
    }

    /// `Comments`; `data.comments()`.
    pub fn comments(&self) -> Sequence<ExtendedString> {
        self.header().comments()
    }

    /// `NumberOfObjects`; `data.number_of_objects()`.
    pub fn number_of_objects(&self) -> i32 {
        self.header().number_of_objects()
    }
}

crate::impl_transient!(Data, "Storage_Data", StandardTransient, parent);
