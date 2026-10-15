// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_NoteComment, XCAFDoc_NoteBalloon and
// XCAFDoc_NoteBinData (.hxx and .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The concrete notes: a text comment, a balloon (a comment shown as a balloon) and a
//! binary data note with its title and MIME type.
use super::note::{
    XCAFDocNote, XCAFDocNoteFields, xcaf_doc_note_dump, xcaf_doc_note_dump_json,
    xcaf_doc_note_paste, xcaf_doc_note_restore,
};
use super::support::{find, live};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::osd::File;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Guid, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable,
};
use std::io;

/// `XCAFDoc_NoteComment::GetID()`.
const NOTE_COMMENT_ID: Guid = guid_from_text("FDEA4C52-0F54-484c-B590-579E18F7B5D4");
/// `XCAFDoc_NoteBalloon::GetID()`.
const NOTE_BALLOON_ID: Guid = guid_from_text("1127951D-87D5-4ecc-89D5-D1406576C43F");
/// `XCAFDoc_NoteBinData::GetID()`.
const NOTE_BIN_DATA_ID: Guid = guid_from_text("E9055501-F0FC-4864-BE4B-284FDA7DDEAC");

/// `XCAFDoc_NoteComment`: a note with a text comment.
///
/// ```
/// use rustycad_tkernel::tcollection::ExtendedString;
/// use rustycad_tklcaf::tdf::TdfData;
/// use rustycad_tkxcaf::xcaf_doc::{XCAFDocNoteComment, XCAFDocNoteMembers};
/// let data = TdfData::new();
/// let text = ExtendedString::from_utf8;
/// let note = XCAFDocNoteComment::set(&data.root(), &text("me"), &text("now"), &text("hi"))?;
/// let note = note.get().unwrap();
/// assert_eq!(note.comment().to_utf8(), "hi");
/// assert_eq!(note.user_name().to_utf8(), "me");
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct XCAFDocNoteComment {
    transient: StandardTransient,
    base: TdfAttributeBase,
    note: XCAFDocNoteFields,
    comment: SharedState<ExtendedString>,
}
rustycad_tkernel::impl_transient!(XCAFDocNoteComment, "XCAFDoc_NoteComment", dyn XCAFDocNote, transient,
    interfaces: [dyn TdfAttribute, dyn XCAFDocNote]);
impl XCAFDocNote for XCAFDocNoteComment {
    fn note_fields(&self) -> &XCAFDocNoteFields {
        &self.note
    }
}

impl XCAFDocNoteComment {
    /// `XCAFDoc_NoteComment()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        NOTE_COMMENT_ID
    }
    /// `Get(theLabel)`: the comment of the label, or null.
    pub fn get(label: &TdfLabel) -> Result<Handle<XCAFDocNoteComment>, Failure> {
        find::<XCAFDocNoteComment>(label, &NOTE_COMMENT_ID)
    }
    /// `Set(theLabel, theUserName, theTimeStamp, theComment)`: a new comment on a label
    /// without one; null for a null label or one that already has a comment.
    pub fn set(
        label: &TdfLabel,
        user: &ExtendedString,
        stamp: &ExtendedString,
        comment: &ExtendedString,
    ) -> Result<Handle<XCAFDocNoteComment>, Failure> {
        if label.is_null() || label.is_attribute(&NOTE_COMMENT_ID)? {
            return Ok(Handle::default());
        }
        let note = Self::new();
        note.note.assign(user, stamp);
        *note.comment.write() = comment.clone();
        let note = Handle::new(note);
        label.add_attribute(&note.clone().cast(), false)?;
        Ok(note)
    }
    /// `Set(theComment)`: backed up.
    pub fn set_comment(&self, comment: &ExtendedString) -> Result<(), Failure> {
        self.backup()?;
        *self.comment.write() = comment.clone();
        Ok(())
    }
    /// `Comment`.
    pub fn comment(&self) -> ExtendedString {
        self.comment.read().clone()
    }
}

impl TdfAttribute for XCAFDocNoteComment {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        NOTE_COMMENT_ID
    }
    /// `Restore`: the note, then the comment of another comment.
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        xcaf_doc_note_restore(self, with)?;
        if let Some(other) = with.cast::<XCAFDocNoteComment>().get() {
            *self.comment.write() = other.comment();
        }
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: the note, then the comment into another comment.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        xcaf_doc_note_paste(self, into)?;
        if let Some(other) = into.cast::<XCAFDocNoteComment>().get() {
            other.set_comment(&self.comment())?;
        }
        Ok(())
    }
    /// `Dump`: the note, then `Comment : <comment>` (`<empty>` without one).
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        xcaf_doc_note_dump(self, stream)?;
        let comment = self.comment();
        let text = if comment.is_empty() {
            "<empty>".to_owned()
        } else {
            comment.to_utf8()
        };
        write!(stream, "\nComment : {text}")
    }
    /// No `DumpJson` override natively: the `XCAFDoc_Note` dump.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        xcaf_doc_note_dump_json(self, "XCAFDoc_NoteComment", stream, depth)
    }
}

/// `XCAFDoc_NoteBalloon`: a comment shown as a balloon (derived from `XCAFDoc_NoteComment`,
/// with its own GUID).
pub struct XCAFDocNoteBalloon {
    comment: XCAFDocNoteComment,
}
rustycad_tkernel::impl_transient!(XCAFDocNoteBalloon, "XCAFDoc_NoteBalloon", XCAFDocNoteComment, comment,
    interfaces: [dyn TdfAttribute, dyn XCAFDocNote]);
impl XCAFDocNote for XCAFDocNoteBalloon {
    fn note_fields(&self) -> &XCAFDocNoteFields {
        &self.comment.note
    }
}

impl Default for XCAFDocNoteBalloon {
    fn default() -> Self {
        super::registration::register_derived_attributes();
        Self {
            comment: XCAFDocNoteComment::new(),
        }
    }
}

impl XCAFDocNoteBalloon {
    /// `XCAFDoc_NoteBalloon()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        NOTE_BALLOON_ID
    }
    /// `Get(theLabel)`: the balloon of the label, or null.
    pub fn get(label: &TdfLabel) -> Result<Handle<XCAFDocNoteBalloon>, Failure> {
        find::<XCAFDocNoteBalloon>(label, &NOTE_BALLOON_ID)
    }
    /// `Set(theLabel, theUserName, theTimeStamp, theComment)`: a new balloon on a label
    /// without one; null for a null label or one that already has a balloon.
    pub fn set(
        label: &TdfLabel,
        user: &ExtendedString,
        stamp: &ExtendedString,
        comment: &ExtendedString,
    ) -> Result<Handle<XCAFDocNoteBalloon>, Failure> {
        if label.is_null() || label.is_attribute(&NOTE_BALLOON_ID)? {
            return Ok(Handle::default());
        }
        let balloon = Self::new();
        balloon.comment.note.assign(user, stamp);
        *balloon.comment.comment.write() = comment.clone();
        let balloon = Handle::new(balloon);
        label.add_attribute(&balloon.clone().cast(), false)?;
        Ok(balloon)
    }
    /// The embedded `XCAFDoc_NoteComment` (the native base class).
    pub fn note_comment(&self) -> &XCAFDocNoteComment {
        &self.comment
    }
    /// `Comment`.
    pub fn comment(&self) -> ExtendedString {
        self.comment.comment()
    }
    /// `Set(theComment)`: backed up.
    pub fn set_comment(&self, comment: &ExtendedString) -> Result<(), Failure> {
        self.backup()?;
        *self.comment.comment.write() = comment.clone();
        Ok(())
    }
}

impl TdfAttribute for XCAFDocNoteBalloon {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.comment.base
    }
    fn id(&self) -> Guid {
        NOTE_BALLOON_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        self.comment.restore(with)
    }
    /// `NewEmpty` (from `IMPLEMENT_DERIVED_ATTRIBUTE`).
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        self.comment.paste(into, table)
    }
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        self.comment.dump(stream)
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        xcaf_doc_note_dump_json(self, "XCAFDoc_NoteBalloon", stream, depth)
    }
}

/// The values of `XCAFDoc_NoteBinData`.
#[derive(Clone, Default)]
struct BinDataFields {
    title: ExtendedString,
    mime_type: AsciiString,
    data: Handle<HArray1<u8>>,
}

/// `XCAFDoc_NoteBinData`: a note with binary data, its title and MIME type.
#[derive(Default)]
pub struct XCAFDocNoteBinData {
    transient: StandardTransient,
    base: TdfAttributeBase,
    note: XCAFDocNoteFields,
    fields: SharedState<BinDataFields>,
}
rustycad_tkernel::impl_transient!(XCAFDocNoteBinData, "XCAFDoc_NoteBinData", dyn XCAFDocNote, transient,
    interfaces: [dyn TdfAttribute, dyn XCAFDocNote]);
impl XCAFDocNote for XCAFDocNoteBinData {
    fn note_fields(&self) -> &XCAFDocNoteFields {
        &self.note
    }
}

impl XCAFDocNoteBinData {
    /// `XCAFDoc_NoteBinData()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        NOTE_BIN_DATA_ID
    }
    /// `Get(theLabel)`: the binary data note of the label, or null.
    pub fn get(label: &TdfLabel) -> Result<Handle<XCAFDocNoteBinData>, Failure> {
        find::<XCAFDocNoteBinData>(label, &NOTE_BIN_DATA_ID)
    }
    /// A detached note with the author and time stamp, null for a null label or one that
    /// already has a binary data note.
    fn detached(
        label: &TdfLabel,
        user: &ExtendedString,
        stamp: &ExtendedString,
    ) -> Result<Option<Self>, Failure> {
        if label.is_null() || label.is_attribute(&NOTE_BIN_DATA_ID)? {
            return Ok(None);
        }
        let note = Self::new();
        note.note.assign(user, stamp);
        Ok(Some(note))
    }
    /// `Set(theLabel, theUserName, theTimeStamp, theTitle, theMIMEtype, theFile)`: a new note
    /// with the contents of an open readable file; null when it cannot be read.
    pub fn set_from_file(
        label: &TdfLabel,
        user: &ExtendedString,
        stamp: &ExtendedString,
        title: &ExtendedString,
        mime_type: &AsciiString,
        file: &mut File,
    ) -> Result<Handle<XCAFDocNoteBinData>, Failure> {
        let Some(note) = Self::detached(label, user, stamp)? else {
            return Ok(Handle::default());
        };
        let note = Handle::new(note);
        if !live(&note, "XCAFDoc_NoteBinData::Set")?.set_file(title, mime_type, file)? {
            return Ok(Handle::default());
        }
        label.add_attribute(&note.clone().cast(), false)?;
        Ok(note)
    }
    /// `Set(theLabel, theUserName, theTimeStamp, theTitle, theMIMEtype, theData)`: a new note
    /// with the data.
    pub fn set(
        label: &TdfLabel,
        user: &ExtendedString,
        stamp: &ExtendedString,
        title: &ExtendedString,
        mime_type: &AsciiString,
        data: &Handle<HArray1<u8>>,
    ) -> Result<Handle<XCAFDocNoteBinData>, Failure> {
        let Some(note) = Self::detached(label, user, stamp)? else {
            return Ok(Handle::default());
        };
        *note.fields.write() = BinDataFields {
            title: title.clone(),
            mime_type: mime_type.clone(),
            data: data.clone(),
        };
        let note = Handle::new(note);
        label.add_attribute(&note.clone().cast(), false)?;
        Ok(note)
    }
}

/// The values of a binary data note.
impl XCAFDocNoteBinData {
    /// `Set(theTitle, theMIMEtype, theFile)`: backed up; the bytes of an open readable file
    /// (false when it is closed, unreadable, too large or read short).
    pub fn set_file(
        &self,
        title: &ExtendedString,
        mime_type: &AsciiString,
        file: &mut File,
    ) -> Result<bool, Failure> {
        if !file.is_open() || !file.is_readable() {
            return Ok(false);
        }
        self.backup()?;
        let size = file.size()?;
        if size > i32::MAX as usize {
            return Ok(false);
        }
        let array = HArray1::with_bounds(1, size as i32)
            .map_err(|_| Failure::out_of_range("NCollection_Array1"))?;
        let read = file.read_bytes(array.change_array1().as_mut_slice(), size as i32)?;
        let length = array.array1().length();
        self.fields.write().data = Handle::new(array);
        if read < length {
            return Ok(false);
        }
        let mut fields = self.fields.write();
        fields.title = title.clone();
        fields.mime_type = mime_type.clone();
        Ok(true)
    }
    /// `Set(theTitle, theMIMEtype, theData)`: backed up.
    pub fn set_data(
        &self,
        title: &ExtendedString,
        mime_type: &AsciiString,
        data: &Handle<HArray1<u8>>,
    ) -> Result<(), Failure> {
        self.backup()?;
        *self.fields.write() = BinDataFields {
            title: title.clone(),
            mime_type: mime_type.clone(),
            data: data.clone(),
        };
        Ok(())
    }
    /// `Title`.
    pub fn title(&self) -> ExtendedString {
        self.fields.read().title.clone()
    }
    /// `MIMEtype`.
    pub fn mime_type(&self) -> AsciiString {
        self.fields.read().mime_type.clone()
    }
    /// `Size`: the byte count, 0 without data.
    pub fn size(&self) -> i32 {
        self.fields
            .read()
            .data
            .get()
            .map_or(0, |data| data.array1().length())
    }
    /// `Data`.
    pub fn data(&self) -> Handle<HArray1<u8>> {
        self.fields.read().data.clone()
    }
}

impl TdfAttribute for XCAFDocNoteBinData {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        NOTE_BIN_DATA_ID
    }
    /// `Restore`: the note, then the values of another binary data note.
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        xcaf_doc_note_restore(self, with)?;
        if let Some(other) = with.cast::<XCAFDocNoteBinData>().get() {
            *self.fields.write() = other.fields.read().clone();
        }
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: the note, then the values into another binary data note.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        xcaf_doc_note_paste(self, into)?;
        if let Some(other) = into.cast::<XCAFDocNoteBinData>().get() {
            let fields = self.fields.read().clone();
            other.set_data(&fields.title, &fields.mime_type, &fields.data)?;
        }
        Ok(())
    }
    /// `Dump`: the note, the title (the native prints the MIME type for a set title), the MIME
    /// type, the size and the bytes as numbers.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        xcaf_doc_note_dump(self, stream)?;
        let fields = self.fields.read().clone();
        let mime = String::from_utf8_lossy(fields.mime_type.as_bytes()).into_owned();
        let title = if fields.title.is_empty() {
            "<untitled>".to_owned()
        } else {
            mime.clone()
        };
        let mime = if mime.is_empty() {
            "<none>".to_owned()
        } else {
            mime
        };
        write!(
            stream,
            "\nTitle : {title}\nMIME type : {mime}\nSize : {} bytes\n",
            self.size()
        )?;
        if let Some(data) = fields.data.get() {
            stream.write_all(data.array1().as_slice())?;
        }
        Ok(())
    }
    /// No `DumpJson` override natively: the `XCAFDoc_Note` dump.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        xcaf_doc_note_dump_json(self, "XCAFDoc_NoteBinData", stream, depth)
    }
}
