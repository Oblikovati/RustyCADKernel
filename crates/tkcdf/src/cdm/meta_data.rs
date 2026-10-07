// Rust port of ApplicationFramework/TKCDF/CDM/CDM_MetaData.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CDM_MetaData`: where a stored document lives (folder, name, version, file).
use super::{CdmApplication, CdmDocument, MetaDataLookUpTable};
use crate::json_dump::{class_begin, field_integer, field_pointer, field_string};
use rustycad_tkernel::{
    impl_transient,
    standard::{
        Failure, Handle, SharedState, StandardTransient, WeakHandle, dump::DumpStream, exceptions,
    },
    tcollection::ExtendedString,
};
use std::io;

#[derive(Clone, Default)]
struct MetaDataState {
    is_retrieved: bool,
    // Native `CDM_Document* myDocument`: a raw pointer, so a weak handle (section 6).
    document: WeakHandle<dyn CdmDocument>,
    folder: ExtendedString,
    name: ExtendedString,
    version: ExtendedString,
    has_version: bool,
    file_name: ExtendedString,
    path: ExtendedString,
    document_version: i32,
    is_read_only: bool,
}

/// `CDM_MetaData`: the database description of a stored document, shared through the
/// application's look-up table; `CdmMetaData::look_up(&mut table, &folder, &name, &path,
/// &file, false)`.
pub struct CdmMetaData {
    transient: StandardTransient,
    state: SharedState<MetaDataState>,
}
impl_transient!(CdmMetaData, "CDM_MetaData", StandardTransient, transient);

/// The fields of a new meta-data, as the two private constructors take them.
struct MetaDataFields<'a> {
    folder: &'a ExtendedString,
    name: &'a ExtendedString,
    path: &'a ExtendedString,
    version: Option<&'a ExtendedString>,
    file_name: &'a ExtendedString,
    read_only: bool,
}

impl CdmMetaData {
    fn create(fields: &MetaDataFields<'_>) -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            state: SharedState::new(MetaDataState {
                folder: fields.folder.clone(),
                name: fields.name.clone(),
                version: fields.version.cloned().unwrap_or_default(),
                has_version: fields.version.is_some(),
                file_name: fields.file_name.clone(),
                path: fields.path.clone(),
                is_read_only: fields.read_only,
                ..MetaDataState::default()
            }),
        })
    }
    /// `LookUp(table, folder, name, path, fileName, readOnly)`: the meta-data bound to the
    /// path (backslashes read as slashes), created and bound when absent.
    ///
    /// ```
    /// use rustycad_tkcdf::cdm::{CdmMetaData, MetaDataLookUpTable};
    /// use rustycad_tkernel::tcollection::ExtendedString;
    /// let mut table = MetaDataLookUpTable::default();
    /// let s = |t: &str| ExtendedString::new(t);
    /// let first = CdmMetaData::look_up(&mut table, &s("f"), &s("n"), &s("a\\b"), &s("b"), false);
    /// let again = CdmMetaData::look_up(&mut table, &s("g"), &s("m"), &s("a/b"), &s("b"), true);
    /// assert!(first == again && !first.has_version());
    /// ```
    pub fn look_up(
        table: &mut MetaDataLookUpTable,
        folder: &ExtendedString,
        name: &ExtendedString,
        path: &ExtendedString,
        file_name: &ExtendedString,
        read_only: bool,
    ) -> Handle<Self> {
        let fields = MetaDataFields {
            folder,
            name,
            path,
            version: None,
            file_name,
            read_only,
        };
        Self::look_up_fields(table, &fields)
    }
    /// `LookUp(table, folder, name, path, version, fileName, readOnly)`: as
    /// [`CdmMetaData::look_up`], with a version when the meta-data is created.
    pub fn look_up_with_version(
        table: &mut MetaDataLookUpTable,
        folder: &ExtendedString,
        name: &ExtendedString,
        path: &ExtendedString,
        version: &ExtendedString,
        file_name: &ExtendedString,
        read_only: bool,
    ) -> Handle<Self> {
        let fields = MetaDataFields {
            folder,
            name,
            path,
            version: Some(version),
            file_name,
            read_only,
        };
        Self::look_up_fields(table, &fields)
    }
    fn look_up_fields(
        table: &mut MetaDataLookUpTable,
        fields: &MetaDataFields<'_>,
    ) -> Handle<Self> {
        let mut conventional_path = fields.path.clone();
        conventional_path.change_all(u16::from(b'\\'), u16::from(b'/'));
        if let Ok(found) = table.find(&conventional_path) {
            return found.clone();
        }
        let created = Self::create(fields);
        table.bind(conventional_path, created.clone());
        created
    }
    /// `IsRetrieved`: a document in session is attached; `meta.is_retrieved()`.
    pub fn is_retrieved(&self) -> bool {
        self.state.read().is_retrieved
    }
    /// `Document`: the attached document, null when it was never set or is gone.
    pub fn document(&self) -> Handle<dyn CdmDocument> {
        self.state.read().document.upgrade()
    }
    /// Private `SetDocument` (friend `CDM_Document::SetMetaData`): attaches a document.
    pub(crate) fn set_document(&self, document: &Handle<dyn CdmDocument>) {
        let mut state = self.state.write();
        state.is_retrieved = true;
        state.document = document.downgrade();
    }
    /// `UnsetDocument`: marks the meta-data not retrieved; the pointer is kept, as natively.
    pub fn unset_document(&self) {
        self.state.write().is_retrieved = false;
    }
    /// `Folder`; `meta.folder()`.
    pub fn folder(&self) -> ExtendedString {
        self.state.read().folder.clone()
    }
    /// `Name`; `meta.name()`.
    pub fn name(&self) -> ExtendedString {
        self.state.read().name.clone()
    }
    /// `Version`: `Standard_NoSuchObject` when no version was given.
    pub fn version(&self) -> Result<ExtendedString, Failure> {
        let state = self.state.read();
        if !state.has_version {
            return Err(exceptions::no_such_object("Document has no version"));
        }
        Ok(state.version.clone())
    }
    /// `HasVersion`; `meta.has_version()`.
    pub fn has_version(&self) -> bool {
        self.state.read().has_version
    }
    /// `FileName`; `meta.file_name()`.
    pub fn file_name(&self) -> ExtendedString {
        self.state.read().file_name.clone()
    }
    /// `Path`: the path as given, not the conventional key; `meta.path()`.
    pub fn path(&self) -> ExtendedString {
        self.state.read().path.clone()
    }
    /// `Print` (and `operator<<`): `*CDM_MetaData*folder,name[,version]; Physical situation:
    /// file` and a newline.
    pub fn print(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        let state = self.state.read();
        write!(
            stream,
            "*CDM_MetaData*{},{}",
            state.folder.to_utf8(),
            state.name.to_utf8()
        )?;
        if state.has_version {
            write!(stream, ",{}", state.version.to_utf8())?;
        }
        writeln!(
            stream,
            "; Physical situation: {}",
            state.file_name.to_utf8()
        )
    }
    /// Private `DocumentVersion` (friends `CDM_Reference`, `CDM_Application`): asks the
    /// application once and caches a nonzero answer. Associated, because the native member
    /// passes `this` to the application.
    pub(crate) fn document_version(
        meta_data: &Handle<Self>,
        application: &Handle<dyn CdmApplication>,
    ) -> Result<i32, Failure> {
        let own = crate::live(meta_data, "CDM_MetaData::DocumentVersion")?;
        let cached = own.state.read().document_version;
        if cached != 0 {
            return Ok(cached);
        }
        let version = crate::live(application, "CDM_MetaData::DocumentVersion")?
            .document_version(meta_data)?;
        own.state.write().document_version = version;
        Ok(version)
    }
    /// `IsReadOnly`; `meta.is_read_only()`.
    pub fn is_read_only(&self) -> bool {
        self.state.read().is_read_only
    }
    /// `SetIsReadOnly`; `meta.set_is_read_only()`.
    pub fn set_is_read_only(&self) {
        self.state.write().is_read_only = true;
    }
    /// `UnsetIsReadOnly`; `meta.unset_is_read_only()`.
    pub fn unset_is_read_only(&self) {
        self.state.write().is_read_only = false;
    }
    /// `DumpJson`: the native fields in native order; `meta.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
        let state = self.state.snapshot();
        class_begin(stream, "CDM_MetaData")?;
        field_integer(stream, "myIsRetrieved", i32::from(state.is_retrieved))?;
        let document = state.document.upgrade();
        field_pointer(stream, "myDocument", document.object_address())?;
        field_string(stream, "myFolder", &state.folder.to_utf8())?;
        field_string(stream, "myName", &state.name.to_utf8())?;
        field_string(stream, "myVersion", &state.version.to_utf8())?;
        field_integer(stream, "myHasVersion", i32::from(state.has_version))?;
        field_string(stream, "myFileName", &state.file_name.to_utf8())?;
        field_string(stream, "myPath", &state.path.to_utf8())?;
        field_integer(stream, "myDocumentVersion", state.document_version)?;
        field_integer(stream, "myIsReadOnly", i32::from(state.is_read_only))
    }
}
