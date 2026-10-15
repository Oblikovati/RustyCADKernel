// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_NotesTool.hxx and .cxx (the tool, the notes
// and the annotated items) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_NotesTool`: the notes of an XDE document (children of the notes label) and the
//! items they annotate (children of the annotated items label with an
//! `XCAFDoc_AssemblyItemRef`), linked by graph nodes of `NoteRefGUID` (the note the father).
use super::color_tool::entry_text;
use super::generic_empty_tool::generic_empty_attribute;
use super::note::XCAFDocNote;
use super::support::{child_labels, child_labels_with, find, find_or_add};
use super::{
    XCAFDocAssemblyItemId, XCAFDocAssemblyItemRef, XCAFDocNoteBalloon, XCAFDocNoteBinData,
    XCAFDocNoteComment,
};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::osd::File;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Guid, Handle};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfTagSource};
use std::io;

/// The children of the tool's label (`NotesTool_RootLabels`).
const NOTES_ROOT: i32 = 1;
const ANNOTATED_ITEMS_ROOT: i32 = 2;

generic_empty_attribute!(
    /// `XCAFDoc_NotesTool`.
    XCAFDocNotesTool, "XCAFDoc_NotesTool", "8F8174B1-6125-47a0-B357-61BD2D89380C", state: (),
    dump: |tool: &XCAFDocNotesTool, stream: &mut DumpStream, depth: i32| -> io::Result<()> {
        // No override natively: the `TDataStd_GenericEmpty` dump.
        rustycad_tklcaf::tdata_std::dump_generic_empty(tool, stream, depth)
    },
    text: |tool: &XCAFDocNotesTool, stream: &mut dyn io::Write| -> io::Result<()> {
        let notes = tool.nb_notes().map_err(io::Error::other)?;
        let items = tool.nb_annotated_items().map_err(io::Error::other)?;
        write!(stream, "Notes           : {notes}\nAnnotated items : {items}\n")
    }
);

/// The file-local `labeledItem(theLabel)`: the item of the label's entry.
pub(super) fn labeled_item(label: &TdfLabel) -> XCAFDocAssemblyItemId {
    XCAFDocAssemblyItemId::from_string(&AsciiString::from(entry_text(label).as_str()))
}

/// What an annotation refers to: the whole item, one of its attributes or a subshape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Annotated {
    /// The item itself (no extra reference).
    Item,
    /// An attribute of the item.
    Attribute(Guid),
    /// A subshape of the item.
    Subshape(i32),
}

impl Annotated {
    /// The test of the `FindAnnotatedItem*` members on a reference to the item.
    fn matches(self, reference: &XCAFDocAssemblyItemRef) -> bool {
        match self {
            Self::Item => !reference.has_extra_ref(),
            Self::Attribute(guid) => reference.has_extra_ref() && reference.get_guid() == guid,
            Self::Subshape(index) => {
                reference.has_extra_ref() && reference.get_subshape_index() == index
            }
        }
    }
}

impl XCAFDocNotesTool {
    /// `Set(theLabel)`: the tool of the label, added when absent (null for a null label).
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocNotesTool>, Failure> {
        if label.is_null() {
            return Ok(Handle::default());
        }
        find_or_add(label, &Self::get_id(), Self::new)
    }
    /// `GetNotesLabel`: child 1 (created when absent).
    pub fn get_notes_label(&self) -> Result<TdfLabel, Failure> {
        self.label().find_child(NOTES_ROOT, true)
    }
    /// `GetAnnotatedItemsLabel`: child 2 (created when absent).
    pub fn get_annotated_items_label(&self) -> Result<TdfLabel, Failure> {
        self.label().find_child(ANNOTATED_ITEMS_ROOT, true)
    }
    /// `GetNotes(theNoteLabels)`: appends the note labels.
    pub fn get_notes(&self, notes: &mut Vec<TdfLabel>) -> Result<(), Failure> {
        for label in child_labels(&self.get_notes_label()?)? {
            if <dyn XCAFDocNote>::is_mine(&label)? {
                notes.push(label);
            }
        }
        Ok(())
    }
    /// `NbNotes`.
    pub fn nb_notes(&self) -> Result<i32, Failure> {
        let mut notes = Vec::new();
        self.get_notes(&mut notes)?;
        Ok(notes.len() as i32)
    }
    /// `GetAnnotatedItems(theItemLabels)`: appends the annotated item labels.
    pub fn get_annotated_items(&self, items: &mut Vec<TdfLabel>) -> Result<(), Failure> {
        items.extend(child_labels_with(
            &self.get_annotated_items_label()?,
            &XCAFDocAssemblyItemRef::get_id(),
        )?);
        Ok(())
    }
    /// `NbAnnotatedItems`.
    pub fn nb_annotated_items(&self) -> Result<i32, Failure> {
        let mut items = Vec::new();
        self.get_annotated_items(&mut items)?;
        Ok(items.len() as i32)
    }
    /// The annotated item label of the item and the kind of reference, null when none.
    pub(super) fn find_annotated(
        &self,
        item: &XCAFDocAssemblyItemId,
        kind: Annotated,
    ) -> Result<TdfLabel, Failure> {
        for label in child_labels_with(
            &self.get_annotated_items_label()?,
            &XCAFDocAssemblyItemRef::get_id(),
        )? {
            let reference =
                find::<XCAFDocAssemblyItemRef>(&label, &XCAFDocAssemblyItemRef::get_id())?;
            if reference
                .get()
                .is_some_and(|r| r.get_item().is_equal(item) && kind.matches(r))
            {
                return Ok(label);
            }
        }
        Ok(TdfLabel::default())
    }
}

/// The searches of annotated items.
impl XCAFDocNotesTool {
    /// `IsAnnotatedItem(theItemId)`.
    pub fn is_annotated_item(&self, item: &XCAFDocAssemblyItemId) -> Result<bool, Failure> {
        Ok(!self.find_annotated_item(item)?.is_null())
    }
    /// `IsAnnotatedItem(theItemLabel)`.
    pub fn is_annotated_item_label(&self, label: &TdfLabel) -> Result<bool, Failure> {
        self.is_annotated_item(&labeled_item(label))
    }
    /// `FindAnnotatedItem(theItemId)`: the reference to the whole item, null when none.
    pub fn find_annotated_item(&self, item: &XCAFDocAssemblyItemId) -> Result<TdfLabel, Failure> {
        self.find_annotated(item, Annotated::Item)
    }
    /// `FindAnnotatedItem(theItemLabel)`.
    pub fn find_annotated_item_label(&self, label: &TdfLabel) -> Result<TdfLabel, Failure> {
        self.find_annotated_item(&labeled_item(label))
    }
    /// `FindAnnotatedItemAttr(theItemId, theGUID)`.
    pub fn find_annotated_item_attr(
        &self,
        item: &XCAFDocAssemblyItemId,
        guid: &Guid,
    ) -> Result<TdfLabel, Failure> {
        self.find_annotated(item, Annotated::Attribute(*guid))
    }
    /// `FindAnnotatedItemAttr(theItemLabel, theGUID)`.
    pub fn find_annotated_item_attr_label(
        &self,
        label: &TdfLabel,
        guid: &Guid,
    ) -> Result<TdfLabel, Failure> {
        self.find_annotated_item_attr(&labeled_item(label), guid)
    }
    /// `FindAnnotatedItemSubshape(theItemId, theSubshapeIndex)`.
    pub fn find_annotated_item_subshape(
        &self,
        item: &XCAFDocAssemblyItemId,
        index: i32,
    ) -> Result<TdfLabel, Failure> {
        self.find_annotated(item, Annotated::Subshape(index))
    }
    /// `FindAnnotatedItemSubshape(theItemLabel, theSubshapeIndex)`.
    pub fn find_annotated_item_subshape_label(
        &self,
        label: &TdfLabel,
        index: i32,
    ) -> Result<TdfLabel, Failure> {
        self.find_annotated_item_subshape(&labeled_item(label), index)
    }
}

/// The creation of notes.
impl XCAFDocNotesTool {
    /// A new child of the notes label.
    fn new_note_label(&self) -> Result<TdfLabel, Failure> {
        TdfTagSource::new_child_of(&self.get_notes_label()?)
    }
    /// `CreateComment(theUserName, theTimeStamp, theComment)`.
    pub fn create_comment(
        &self,
        user: &ExtendedString,
        stamp: &ExtendedString,
        comment: &ExtendedString,
    ) -> Result<Handle<dyn XCAFDocNote>, Failure> {
        Ok(XCAFDocNoteComment::set(&self.new_note_label()?, user, stamp, comment)?.cast())
    }
    /// `CreateBalloon(theUserName, theTimeStamp, theComment)`.
    pub fn create_balloon(
        &self,
        user: &ExtendedString,
        stamp: &ExtendedString,
        comment: &ExtendedString,
    ) -> Result<Handle<dyn XCAFDocNote>, Failure> {
        Ok(XCAFDocNoteBalloon::set(&self.new_note_label()?, user, stamp, comment)?.cast())
    }
    /// `CreateBinData(theUserName, theTimeStamp, theTitle, theMIMEtype, theFile)`.
    pub fn create_bin_data_from_file(
        &self,
        user: &ExtendedString,
        stamp: &ExtendedString,
        title: &ExtendedString,
        mime_type: &AsciiString,
        file: &mut File,
    ) -> Result<Handle<dyn XCAFDocNote>, Failure> {
        Ok(XCAFDocNoteBinData::set_from_file(
            &self.new_note_label()?,
            user,
            stamp,
            title,
            mime_type,
            file,
        )?
        .cast())
    }
    /// `CreateBinData(theUserName, theTimeStamp, theTitle, theMIMEtype, theData)`.
    pub fn create_bin_data(
        &self,
        user: &ExtendedString,
        stamp: &ExtendedString,
        title: &ExtendedString,
        mime_type: &AsciiString,
        data: &Handle<HArray1<u8>>,
    ) -> Result<Handle<dyn XCAFDocNote>, Failure> {
        Ok(
            XCAFDocNoteBinData::set(&self.new_note_label()?, user, stamp, title, mime_type, data)?
                .cast(),
        )
    }
}
