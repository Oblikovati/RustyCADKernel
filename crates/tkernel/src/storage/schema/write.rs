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

//! `Storage_Schema::Write`: every section of a data set, in native order.
use super::Schema;
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle, StandardPersistent};
use crate::storage::{
    BaseDriver, BucketIterator, CallBackTable, Data, InternalData, OpenMode, Root, StorageError,
    StreamException, TypedCallBack, version,
};
use crate::tcollection::AsciiString;

/// The step `Write` is in, reported as the error extension of a write error.
struct WriteStep(&'static str);

impl Schema {
    /// `Write(theDriver, aData)`: numbers the roots' objects, fills the header and writes
    /// every section. A `Storage_StreamWriteError` becomes the data's `WriteError` status
    /// with the failing step; any other failure propagates without the final clean-up,
    /// as the native exception does. A null data handle does nothing.
    /// Example: `schema.write(&mut driver, &data)?`.
    pub fn write(&self, driver: &mut dyn BaseDriver, data: &Handle<Data>) -> Result<(), Failure> {
        let Some(set) = data.get() else {
            return Ok(());
        };
        *self.current() = data.clone();
        let internal = set.internal_data();
        set.clear();
        set.clear_error_status();
        let roots = set.roots();
        for root in root_list(&roots) {
            self.persistent_to_add(&root.get().map(Root::object).unwrap_or_default())?;
        }
        self.fill_header(set, &internal);
        if matches!(driver.open_mode(), OpenMode::Write | OpenMode::ReadWrite) {
            let mut step = WriteStep("AddPersistent");
            match self.write_sections(driver, set, &roots, &mut step) {
                Err(failure) if StreamException::Write.is(&failure) => {
                    set.set_status(StorageError::WriteError, step.0);
                }
                outcome => outcome?,
            }
        } else {
            set.set_status(StorageError::ModeError, "OpenMode");
        }
        if let Some(internal) = internal.get() {
            internal.clear();
        }
        self.clear();
        Ok(())
    }

    fn fill_header(&self, set: &Data, internal: &Handle<InternalData>) {
        let header = set.header_data();
        let Some(header) = header.get() else {
            return;
        };
        let objects = internal
            .get()
            .map_or(0, |internal| internal.lock().objects_to_add.length());
        header.set_creation_date(&self.clock.creation_date());
        header.set_storage_version(&version());
        header.set_number_of_objects(objects);
        header.set_schema_name(&self.name);
        header.set_schema_version(&self.version);
    }

    fn write_sections(
        &self,
        driver: &mut dyn BaseDriver,
        set: &Data,
        roots: &Handle<HSequence<Handle<Root>>>,
        step: &mut WriteStep,
    ) -> Result<(), Failure> {
        write_header_sections(driver, set, step)?;
        let functions = write_type_section(driver, set, step)?;
        write_root_section(driver, roots, step)?;
        let objects = collected_objects(set);
        self.write_ref_section(driver, set, &objects, step)?;
        self.write_data_section(driver, &functions, &objects, step)
    }

    fn write_ref_section(
        &self,
        driver: &mut dyn BaseDriver,
        set: &Data,
        objects: &[Handle<StandardPersistent>],
        step: &mut WriteStep,
    ) -> Result<(), Failure> {
        step.0 = "BeginWriteRefSection";
        driver.begin_write_ref_section()?;
        let next_object = set
            .internal_data()
            .get()
            .map_or(1, |internal| internal.lock().object_id);
        driver.set_ref_section_size(next_object - 1)?;
        step.0 = "WriteReferenceType";
        for object in objects.iter().filter_map(Handle::get) {
            driver.write_reference_type(object.ref_num(), object.type_num())?;
        }
        step.0 = "EndWriteRefSection";
        driver.end_write_ref_section()?;
        Ok(())
    }

    fn write_data_section(
        &self,
        driver: &mut dyn BaseDriver,
        functions: &CallBackTable,
        objects: &[Handle<StandardPersistent>],
        step: &mut WriteStep,
    ) -> Result<(), Failure> {
        step.0 = "BeginWriteDataSection";
        driver.begin_write_data_section()?;
        step.0 = "Write";
        for handle in objects {
            let Some(object) = handle.get() else {
                continue;
            };
            functions
                .value(object.type_num(), "Storage_Schema::Write")?
                .write(handle, driver, self)?;
            object.set_type_num(0);
        }
        step.0 = "EndWriteDataSection";
        driver.end_write_data_section()?;
        Ok(())
    }
}

fn root_list(roots: &Handle<HSequence<Handle<Root>>>) -> Vec<Handle<Root>> {
    roots
        .get()
        .map(|roots| roots.sequence(|items| items.iter().cloned().collect()))
        .unwrap_or_default()
}

/// The slots `Storage_BucketIterator` visits, taken before callbacks run so that a callback
/// reaching back into the schema cannot deadlock on the internal data.
fn collected_objects(set: &Data) -> Vec<Handle<StandardPersistent>> {
    let internal = set.internal_data();
    let Some(internal) = internal.get() else {
        return Vec::new();
    };
    let fields = internal.lock();
    let mut walk = BucketIterator::new(Some(&fields.objects_to_add));
    let mut objects = Vec::new();
    while walk.more() {
        objects.push(walk.value());
        walk.next();
    }
    objects
}

fn write_header_sections(
    driver: &mut dyn BaseDriver,
    set: &Data,
    step: &mut WriteStep,
) -> Result<(), Failure> {
    step.0 = "BeginWriteInfoSection";
    driver.begin_write_info_section()?;
    step.0 = "WriteInfo";
    driver.write_info(
        set.number_of_objects(),
        &set.storage_version(),
        &set.creation_date(),
        &set.schema_name(),
        &set.schema_version(),
        &set.application_name(),
        &set.application_version(),
        &set.data_type(),
        &set.user_info(),
    )?;
    step.0 = "EndWriteInfoSection";
    driver.end_write_info_section()?;
    step.0 = "BeginWriteCommentSection";
    driver.begin_write_comment_section()?;
    step.0 = "WriteComment";
    driver.write_comment(&set.comments())?;
    step.0 = "EndWriteCommentSection";
    driver.end_write_comment_section()?;
    Ok(())
}

fn write_type_section(
    driver: &mut dyn BaseDriver,
    set: &Data,
    step: &mut WriteStep,
) -> Result<CallBackTable, Failure> {
    let names = set.types();
    step.0 = "BeginWriteTypeSection";
    driver.begin_write_type_section()?;
    let length = set.number_of_types();
    let mut functions = CallBackTable::new(length)?;
    driver.set_type_section_size(length)?;
    for typed in bound_types(set) {
        if let Some(typed) = typed.get() {
            functions.set_value(typed.index(), typed.callback())?;
        }
    }
    step.0 = "WriteTypeInformations";
    let names: Vec<AsciiString> = names
        .get()
        .map(|names| names.sequence(|items| items.iter().cloned().collect()))
        .unwrap_or_default();
    for (number, name) in (1..=length).zip(names.iter()) {
        driver.write_type_informations(number, name)?;
    }
    step.0 = "EndWriteTypeSection";
    driver.end_write_type_section()?;
    Ok(functions)
}

fn bound_types(set: &Data) -> Vec<Handle<TypedCallBack>> {
    let internal = set.internal_data();
    internal.get().map_or_else(Vec::new, |internal| {
        internal.lock().type_binding.values().cloned().collect()
    })
}

fn write_root_section(
    driver: &mut dyn BaseDriver,
    roots: &Handle<HSequence<Handle<Root>>>,
    step: &mut WriteStep,
) -> Result<(), Failure> {
    let roots = root_list(roots);
    step.0 = "BeginWriteRootSection";
    driver.begin_write_root_section()?;
    driver.set_root_section_size(roots.len() as i32)?;
    step.0 = "WriteRoot";
    let document = AsciiString::new("PDocStd_Document");
    for (reference, root) in (1..).zip(roots.iter()) {
        let name = root.get().map(Root::name).unwrap_or_default();
        driver.write_root(&name, reference, &document)?;
    }
    step.0 = "EndWriteRootSection";
    driver.end_write_root_section()?;
    Ok(())
}
