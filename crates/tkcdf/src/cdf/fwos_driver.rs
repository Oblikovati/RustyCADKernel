// Rust port of ApplicationFramework/TKCDF/CDF/CDF_FWOSDriver.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CDF_FWOSDriver`: the meta-data driver over the file system (folder + name = path).
use super::CdfMetaDataDriver;
use crate::cdm::{CdmDocument, CdmDocumentStorage, CdmMetaData, SharedMetaDataLookUpTable};
use crate::live;
use rustycad_tkernel::{
    impl_transient,
    osd::{CadPath, Directory, Environment, File},
    standard::{Failure, Handle, StandardTransient},
    tcollection::ExtendedString,
};

/// `CDF_FWOSDriver`; `CdfFwosDriver::new(table)` shares the application's table.
///
/// ```
/// use rustycad_tkcdf::cdf::{CdfFwosDriver, CdfMetaDataDriver};
/// use rustycad_tkernel::{standard::SharedState, tcollection::ExtendedString};
/// let driver = CdfFwosDriver::new(&std::sync::Arc::new(SharedState::default()));
/// assert!(!driver.find_folder(&ExtendedString::new("/no/such/folder")));
/// ```
pub struct CdfFwosDriver {
    transient: StandardTransient,
    // Native `myLookUpTable`, a pointer to the application's table.
    look_up_table: SharedMetaDataLookUpTable,
}
impl_transient!(CdfFwosDriver, "CDF_FWOSDriver", dyn CdfMetaDataDriver, transient, interfaces: [dyn CdfMetaDataDriver]);

/// The native file-static `PutSlash`: the platform separator.
fn put_slash(text: &mut ExtendedString) {
    text.assign_cat(ExtendedString::new(if cfg!(windows) { "\\" } else { "/" }));
}

/// An `OSD_Path` of an extended string (converted to UTF-8, as `TCollection_AsciiString`).
fn path_of(text: &ExtendedString) -> CadPath {
    CadPath::new(text.to_utf8().as_bytes())
}

impl CdfFwosDriver {
    /// `CDF_FWOSDriver(theLookUpTable)`.
    pub fn new(look_up_table: &SharedMetaDataLookUpTable) -> Self {
        Self {
            transient: StandardTransient::default(),
            look_up_table: look_up_table.clone(),
        }
    }
    /// Private static `Concatenate`: `folder` + separator + `name`.
    fn concatenate(folder: &ExtendedString, name: &ExtendedString) -> ExtendedString {
        let mut path = folder.clone();
        put_slash(&mut path);
        path.assign_cat(name);
        path
    }
    /// Whether a file is not writable (the read-only flag of new meta-data).
    fn is_read_only(path: &ExtendedString) -> bool {
        !File::with_path(path_of(path)).is_writeable()
    }
}

impl CdfMetaDataDriver for CdfFwosDriver {
    /// `Find`: the folder exists and holds the file (the version is ignored).
    fn find(
        &self,
        folder: &ExtendedString,
        name: &ExtendedString,
        _version: &ExtendedString,
    ) -> bool {
        if !self.find_folder(folder) {
            return false;
        }
        File::with_path(path_of(&Self::concatenate(folder, name))).exists()
    }
    /// `HasReadPermission`: the file is readable.
    fn has_read_permission(
        &self,
        folder: &ExtendedString,
        name: &ExtendedString,
        _version: &ExtendedString,
    ) -> bool {
        File::with_path(path_of(&Self::concatenate(folder, name))).is_readable()
    }
    /// `MetaData`: looked up by the concatenated path.
    fn meta_data(
        &self,
        folder: &ExtendedString,
        name: &ExtendedString,
        _version: &ExtendedString,
    ) -> Result<Handle<CdmMetaData>, Failure> {
        let path = Self::concatenate(folder, name);
        let read_only = Self::is_read_only(&path);
        let mut table = self.look_up_table.write();
        Ok(CdmMetaData::look_up(
            &mut table, folder, name, &path, &path, read_only,
        ))
    }
    /// `CreateMetaData`: the requested folder and name of the document, stored in `file_name`.
    fn create_meta_data(
        &self,
        document: &Handle<dyn CdmDocument>,
        file_name: &ExtendedString,
    ) -> Result<Handle<CdmMetaData>, Failure> {
        let document = live(document, "CDF_FWOSDriver::CreateMetaData")?;
        let folder = document.requested_folder()?;
        let name = document.requested_name();
        let path = Self::concatenate(&folder, &name);
        let read_only = Self::is_read_only(file_name);
        let mut table = self.look_up_table.write();
        Ok(CdmMetaData::look_up(
            &mut table, &folder, &name, &path, file_name, read_only,
        ))
    }
    /// `BuildFileName`: requested folder + separator + requested name.
    fn build_file_name(
        &self,
        document: &Handle<dyn CdmDocument>,
    ) -> Result<ExtendedString, Failure> {
        let document = live(document, "CDF_FWOSDriver::BuildFileName")?;
        Ok(Self::concatenate(
            &document.requested_folder()?,
            &document.requested_name(),
        ))
    }
    /// `FindFolder`: the directory exists.
    fn find_folder(&self, folder: &ExtendedString) -> bool {
        Directory::with_path(path_of(folder)).exists()
    }
    /// `DefaultFolder`: `$HOME`, else `/tmp` (on Windows `%HOMEDRIVE%%HOMEPATH%`, else
    /// `%TEMP%`, else `.`).
    fn default_folder(&self) -> ExtendedString {
        let read = |name: &str| -> ExtendedString {
            let value = Environment::named(name)
                .map(|mut variable| variable.value())
                .unwrap_or_default();
            ExtendedString::from_ascii_string(&value, true)
        };
        if !cfg!(windows) {
            let home = read("HOME");
            return if home.length() != 0 {
                home
            } else {
                ExtendedString::new("/tmp")
            };
        }
        let drive = read("HOMEDRIVE");
        if drive.length() != 0 {
            return drive.cat(read("HOMEPATH"));
        }
        let temp = read("TEMP");
        if temp.length() == 0 {
            ExtendedString::new(".")
        } else {
            temp
        }
    }
    /// `SetName`: appends the document's file extension unless the name already ends
    /// with it (compared lower case on Windows).
    fn set_name(
        &self,
        document: &Handle<dyn CdmDocument>,
        name: &ExtendedString,
    ) -> Result<ExtendedString, Failure> {
        let extension = live(document, "CDF_FWOSDriver::SetName")?.file_extension()?;
        if extension.length() == 0 {
            return Ok(name.clone());
        }
        let dotted = ExtendedString::new(".").cat(&extension);
        let (compared_name, compared_extension) = (platform_case(name), platform_case(&dotted));
        let (name_length, extension_length) = (compared_name.length(), compared_extension.length());
        let already_there = name_length >= extension_length
            && compared_name.search_from_end(&compared_extension) + extension_length - 1
                == name_length;
        if already_there {
            return Ok(name.clone());
        }
        Ok(name.cat(&dotted))
    }
}

/// The name as `SetName` compares it: lower case (`towlower`) on Windows only.
fn platform_case(text: &ExtendedString) -> ExtendedString {
    if !cfg!(windows) {
        return text.clone();
    }
    let lowered: Vec<u16> = String::from_utf16_lossy(text.as_utf16())
        .to_lowercase()
        .encode_utf16()
        .collect();
    ExtendedString::from(lowered.as_slice())
}
