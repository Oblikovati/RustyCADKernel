// Rust port of CDF_Application::ReaderFromFormat, WriterFromFormat and Format
// (ApplicationFramework/TKCDF/CDF/CDF_Application.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The driver look-ups of `CDF_Application`: the registered drivers, else the legacy
//! plugins named by the `<format>.RetrievalPlugin` / `<format>.StoragePlugin` resources.
use super::CdfApplicationBase;
use crate::SharedResourceManager;
use crate::pcdm::{PcdmReader, PcdmReaderStatus, PcdmRetrievalDriver, PcdmStorageDriver};
use rustycad_tkernel::{
    osd::CadPath,
    standard::{Failure, Guid, Handle, exceptions},
    tcollection::{AsciiString, ExtendedString},
};

/// The value of a resource as UTF-8 text, if it is defined.
pub(crate) fn resource_value(
    resources: &SharedResourceManager,
    name: &str,
) -> Result<Option<ExtendedString>, Failure> {
    let manager = resources.read();
    if !manager.find(name) {
        return Ok(None);
    }
    Ok(Some(ExtendedString::from_ascii_string(
        manager.value(name)?,
        true,
    )))
}

/// The GUID of a plugin resource: its value without blanks, non-Latin-1 units as `?`.
fn plugin_id(value: &ExtendedString) -> Result<Guid, Failure> {
    let mut text = value.clone();
    if text.search([u16::from(b' ')]) != -1 {
        text.remove_all(u16::from(b' '));
    }
    let ascii = AsciiString::from_extended(&text, Some(b'?'));
    Guid::from_ascii(ascii.as_bytes())
}

/// `<format><suffix>`, converted to UTF-8.
fn resource_name(format: &ExtendedString, suffix: &str) -> (ExtendedString, String) {
    let name = format.cat(ExtendedString::new(suffix));
    let utf8 = name.to_utf8();
    (name, utf8)
}

/// `ReaderFromFormat`.
pub(crate) fn reader_from_format(
    base: &CdfApplicationBase,
    resources: &SharedResourceManager,
    format: &ExtendedString,
) -> Result<Handle<dyn PcdmReader>, Failure> {
    if let Ok(reader) = base.readers().find_from_key(format) {
        return Ok(reader.cast());
    }
    let (name, utf8) = resource_name(format, ".RetrievalPlugin");
    let Some(value) = resource_value(resources, &utf8)? else {
        base.set_retrievable_status(PcdmReaderStatus::WrongResource);
        return Err(exceptions::no_such_object(&format!(
            "Could not found the item:{}",
            name.to_utf8()
        )));
    };
    let id = plugin_id(&value)?;
    let loaded = (base.plugin_load())(&id)
        .inspect_err(|_| base.set_retrievable_status(PcdmReaderStatus::WrongResource))?;
    let reader: Handle<dyn PcdmRetrievalDriver> = loaded.cast();
    match reader.get() {
        Some(driver) => driver.set_format(format),
        None => base.set_retrievable_status(PcdmReaderStatus::WrongResource),
    }
    base.readers().add(format.clone(), reader.clone());
    Ok(reader.cast())
}

/// `WriterFromFormat`: a missing resource or a failing plugin also records a null writer.
pub(crate) fn writer_from_format(
    base: &CdfApplicationBase,
    resources: &SharedResourceManager,
    format: &ExtendedString,
) -> Result<Handle<dyn PcdmStorageDriver>, Failure> {
    if let Ok(writer) = base.writers().find_from_key(format) {
        return Ok(writer.clone());
    }
    let (name, utf8) = resource_name(format, ".StoragePlugin");
    let Some(value) = resource_value(resources, &utf8)? else {
        base.writers().add(format.clone(), Handle::default());
        return Err(exceptions::no_such_object(&format!(
            "Could not found the resource definition:{}",
            name.to_utf8()
        )));
    };
    let id = plugin_id(&value)?;
    let loaded = match (base.plugin_load())(&id) {
        Ok(loaded) => loaded,
        Err(failure) => {
            base.writers().add(format.clone(), Handle::default());
            base.set_retrievable_status(PcdmReaderStatus::WrongResource);
            return Err(failure);
        }
    };
    let writer: Handle<dyn PcdmStorageDriver> = loaded.cast();
    match writer.get() {
        Some(driver) => driver.set_format(format),
        None => base.set_retrievable_status(PcdmReaderStatus::WrongResource),
    }
    base.writers().add(format.clone(), writer.clone());
    Ok(writer)
}

/// The file-static `fileExtension`: the `OSD_Path` extension without its leading dot.
pub(crate) fn file_extension(file_name: &ExtendedString) -> AsciiString {
    let mut extension = CadPath::new(file_name.to_utf8().as_bytes()).extension();
    if !extension.is_empty() && extension.value(1).ok() == Some(b'.') {
        // A one-byte removal at position 1 of a non-empty string cannot fail.
        let _ = extension.remove(1, 1);
    }
    extension
}

/// `Format(aFileName, theFormat)`: the format of a native header (none: native formats are
/// not ported), else the `<extension>.FileFormat` resource.
pub(crate) fn format(
    resources: &SharedResourceManager,
    file_name: &ExtendedString,
    found: &mut ExtendedString,
) -> Result<bool, Failure> {
    *found = crate::pcdm::PcdmReadWriterBase::file_format(file_name);
    if found.length() != 0 {
        return Ok(true);
    }
    let extension = file_extension(file_name);
    if extension.is_empty() {
        return Ok(false);
    }
    let name = format!(
        "{}.FileFormat",
        String::from_utf8_lossy(extension.as_bytes())
    );
    match resource_value(resources, &name)? {
        Some(value) => {
            *found = value;
            Ok(true)
        }
        None => Ok(false),
    }
}
