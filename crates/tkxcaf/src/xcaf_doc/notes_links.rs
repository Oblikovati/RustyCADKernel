// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_NotesTool.cxx (the links of notes to
// annotated items and the deletion of notes) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The note links of `XCAFDoc_NotesTool`: the notes of an item, the annotation of items,
//! attributes and subshapes, the removal of annotations and the deletion of notes.
use super::graph_links::{father_labels, node_of};
use super::note::{XCAFDocNote, XCAFDocNoteMembers};
use super::notes_tool::{Annotated, XCAFDocNotesTool, labeled_item};
use super::support::{child_labels, find, live};
use super::{XCAFDoc, XCAFDocAssemblyItemId, XCAFDocAssemblyItemRef, XCAFDocGraphNode};
use rustycad_tkernel::standard::{Failure, Guid, Handle};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfTagSource};

/// The `NoteRefGUID` node of the label, else a new one.
fn note_node(label: &TdfLabel) -> Result<Handle<XCAFDocGraphNode>, Failure> {
    let node = node_of(label, &XCAFDoc::note_ref_guid())?;
    if !node.is_null() {
        return Ok(node);
    }
    XCAFDocGraphNode::set_with_id(label, &XCAFDoc::note_ref_guid())
}

/// The notes of an item.
impl XCAFDocNotesTool {
    /// Appends the notes of the annotated item of that kind; returns the length of `notes`.
    fn notes_of(
        &self,
        item: &XCAFDocAssemblyItemId,
        kind: Annotated,
        notes: &mut Vec<TdfLabel>,
    ) -> Result<i32, Failure> {
        let annotated = self.find_annotated(item, kind)?;
        if annotated.is_null() {
            return Ok(0);
        }
        let Some(fathers) = father_labels(&annotated, &XCAFDoc::note_ref_guid())? else {
            return Ok(0);
        };
        notes.extend(fathers);
        Ok(notes.len() as i32)
    }
    /// `GetNotes(theItemId, theNoteLabels)`: appends the notes of the whole item; returns the
    /// length of the sequence.
    pub fn get_item_notes(
        &self,
        item: &XCAFDocAssemblyItemId,
        notes: &mut Vec<TdfLabel>,
    ) -> Result<i32, Failure> {
        self.notes_of(item, Annotated::Item, notes)
    }
    /// `GetNotes(theItemLabel, theNoteLabels)`.
    pub fn get_item_notes_label(
        &self,
        label: &TdfLabel,
        notes: &mut Vec<TdfLabel>,
    ) -> Result<i32, Failure> {
        self.get_item_notes(&labeled_item(label), notes)
    }
    /// `GetAttrNotes(theItemId, theGUID, theNoteLabels)`.
    pub fn get_attr_notes(
        &self,
        item: &XCAFDocAssemblyItemId,
        guid: &Guid,
        notes: &mut Vec<TdfLabel>,
    ) -> Result<i32, Failure> {
        self.notes_of(item, Annotated::Attribute(*guid), notes)
    }
    /// `GetAttrNotes(theItemLabel, theGUID, theNoteLabels)`.
    pub fn get_attr_notes_label(
        &self,
        label: &TdfLabel,
        guid: &Guid,
        notes: &mut Vec<TdfLabel>,
    ) -> Result<i32, Failure> {
        self.get_attr_notes(&labeled_item(label), guid, notes)
    }
    /// `GetSubshapeNotes(theItemId, theSubshapeIndex, theNoteLabels)`.
    pub fn get_subshape_notes(
        &self,
        item: &XCAFDocAssemblyItemId,
        index: i32,
        notes: &mut Vec<TdfLabel>,
    ) -> Result<i32, Failure> {
        self.notes_of(item, Annotated::Subshape(index), notes)
    }
}

/// The annotation of items.
impl XCAFDocNotesTool {
    /// The body of the `AddNote*` members: links the note to the annotated item of that kind
    /// (created with a reference to the item when absent); null when the label holds no note.
    fn add_annotation(
        &self,
        note: &TdfLabel,
        item: &XCAFDocAssemblyItemId,
        kind: Annotated,
    ) -> Result<Handle<XCAFDocAssemblyItemRef>, Failure> {
        if !<dyn XCAFDocNote>::is_mine(note)? {
            return Ok(Handle::default());
        }
        let mut annotated = self.find_annotated(item, kind)?;
        if annotated.is_null() {
            annotated = TdfTagSource::new_child_of(&self.get_annotated_items_label()?)?;
        }
        let child = note_node(&annotated)?;
        let mut reference =
            find::<XCAFDocAssemblyItemRef>(&annotated, &XCAFDocAssemblyItemRef::get_id())?;
        if reference.is_null() {
            reference = XCAFDocAssemblyItemRef::set(&annotated, item)?;
        }
        let father = note_node(note)?;
        live(&child, "XCAFDoc_NotesTool::AddNote")?.set_father(&father)?;
        live(&father, "XCAFDoc_NotesTool::AddNote")?.set_child(&child)?;
        Ok(reference)
    }
    /// `AddNote(theNoteLabel, theItemId)`.
    pub fn add_note(
        &self,
        note: &TdfLabel,
        item: &XCAFDocAssemblyItemId,
    ) -> Result<Handle<XCAFDocAssemblyItemRef>, Failure> {
        self.add_annotation(note, item, Annotated::Item)
    }
    /// `AddNote(theNoteLabel, theItemLabel)`.
    pub fn add_note_label(
        &self,
        note: &TdfLabel,
        label: &TdfLabel,
    ) -> Result<Handle<XCAFDocAssemblyItemRef>, Failure> {
        self.add_note(note, &labeled_item(label))
    }
    /// `AddNoteToAttr(theNoteLabel, theItemId, theGUID)`: also sets the reference's GUID.
    pub fn add_note_to_attr(
        &self,
        note: &TdfLabel,
        item: &XCAFDocAssemblyItemId,
        guid: &Guid,
    ) -> Result<Handle<XCAFDocAssemblyItemRef>, Failure> {
        let reference = self.add_annotation(note, item, Annotated::Attribute(*guid))?;
        if let Some(r) = reference.get() {
            r.set_guid(guid)?;
        }
        Ok(reference)
    }
    /// `AddNoteToAttr(theNoteLabel, theItemLabel, theGUID)`.
    pub fn add_note_to_attr_label(
        &self,
        note: &TdfLabel,
        label: &TdfLabel,
        guid: &Guid,
    ) -> Result<Handle<XCAFDocAssemblyItemRef>, Failure> {
        self.add_note_to_attr(note, &labeled_item(label), guid)
    }
    /// `AddNoteToSubshape(theNoteLabel, theItemId, theSubshapeIndex)`: also sets the
    /// reference's subshape index.
    pub fn add_note_to_subshape(
        &self,
        note: &TdfLabel,
        item: &XCAFDocAssemblyItemId,
        index: i32,
    ) -> Result<Handle<XCAFDocAssemblyItemRef>, Failure> {
        let reference = self.add_annotation(note, item, Annotated::Subshape(index))?;
        if let Some(r) = reference.get() {
            r.set_subshape_index(index)?;
        }
        Ok(reference)
    }
    /// `AddNoteToSubshape(theNoteLabel, theItemLabel, theSubshapeIndex)`.
    pub fn add_note_to_subshape_label(
        &self,
        note: &TdfLabel,
        label: &TdfLabel,
        index: i32,
    ) -> Result<Handle<XCAFDocAssemblyItemRef>, Failure> {
        self.add_note_to_subshape(note, &labeled_item(label), index)
    }
}

/// The removal of annotations.
impl XCAFDocNotesTool {
    /// The body of the `Remove*Note` members: unlinks the note from the annotated item of that
    /// kind (forgetting the item without notes left), then deletes an orphan note on request.
    fn remove_annotation(
        &self,
        note: &TdfLabel,
        item: &XCAFDocAssemblyItemId,
        kind: Annotated,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        let attribute = <dyn XCAFDocNote>::get(note)?;
        let Some(note_attribute) = attribute.get() else {
            return Ok(false);
        };
        let father = node_of(note, &XCAFDoc::note_ref_guid())?;
        let annotated = self.find_annotated(item, kind)?;
        if father.is_null() || annotated.is_null() {
            return Ok(false);
        }
        let child = node_of(&annotated, &XCAFDoc::note_ref_guid())?;
        let Some(child_node) = child.get() else {
            return Ok(false);
        };
        child_node.unset_father(&father)?;
        if child_node.nb_fathers() == 0 {
            annotated.forget_all_attributes(true)?;
        }
        if delete_orphan && note_attribute.is_orphan()? {
            self.delete_note(note)?;
        }
        Ok(true)
    }
    /// `RemoveNote(theNoteLabel, theItemId, theDelIfOrphan)`.
    pub fn remove_note(
        &self,
        note: &TdfLabel,
        item: &XCAFDocAssemblyItemId,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        self.remove_annotation(note, item, Annotated::Item, delete_orphan)
    }
    /// `RemoveNote(theNoteLabel, theItemLabel, theDelIfOrphan)`.
    pub fn remove_note_label(
        &self,
        note: &TdfLabel,
        label: &TdfLabel,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        self.remove_note(note, &labeled_item(label), delete_orphan)
    }
    /// `RemoveSubshapeNote(theNoteLabel, theItemId, theSubshapeIndex, theDelIfOrphan)`.
    pub fn remove_subshape_note(
        &self,
        note: &TdfLabel,
        item: &XCAFDocAssemblyItemId,
        index: i32,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        self.remove_annotation(note, item, Annotated::Subshape(index), delete_orphan)
    }
    /// `RemoveSubshapeNote(theNoteLabel, theItemLabel, theSubshapeIndex, theDelIfOrphan)`.
    pub fn remove_subshape_note_label(
        &self,
        note: &TdfLabel,
        label: &TdfLabel,
        index: i32,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        self.remove_subshape_note(note, &labeled_item(label), index, delete_orphan)
    }
    /// `RemoveAttrNote(theNoteLabel, theItemId, theGUID, theDelIfOrphan)`.
    pub fn remove_attr_note(
        &self,
        note: &TdfLabel,
        item: &XCAFDocAssemblyItemId,
        guid: &Guid,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        self.remove_annotation(note, item, Annotated::Attribute(*guid), delete_orphan)
    }
    /// `RemoveAttrNote(theNoteLabel, theItemLabel, theGUID, theDelIfOrphan)`.
    pub fn remove_attr_note_label(
        &self,
        note: &TdfLabel,
        label: &TdfLabel,
        guid: &Guid,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        self.remove_attr_note(note, &labeled_item(label), guid, delete_orphan)
    }
    /// The body of the `RemoveAll*Notes` members: unlinks every note of the annotated item of
    /// that kind (deleting orphans on request) and forgets the item. A father that is not a
    /// note loops forever natively; it raises `Standard_ProgramError` here.
    fn remove_all(
        &self,
        item: &XCAFDocAssemblyItemId,
        kind: Annotated,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        let annotated = self.find_annotated(item, kind)?;
        if annotated.is_null() {
            return Ok(false);
        }
        let child = node_of(&annotated, &XCAFDoc::note_ref_guid())?;
        let Some(child_node) = child.get() else {
            return Ok(false);
        };
        while child_node.nb_fathers() > 0 {
            let father = child_node.get_father(1)?;
            let father_label = live(&father, "XCAFDoc_NotesTool::RemoveAllNotes")?.label();
            let note = <dyn XCAFDocNote>::get(&father_label)?;
            let Some(note) = note.get() else {
                return Err(Failure::with_message(
                    "Standard_ProgramError",
                    "XCAFDoc_NotesTool::RemoveAllNotes: a father that is not a note",
                ));
            };
            live(&father, "XCAFDoc_NotesTool::RemoveAllNotes")?.unset_child(&child)?;
            if delete_orphan && note.is_orphan()? {
                self.delete_note(&father_label)?;
            }
        }
        annotated.forget_all_attributes(true)?;
        Ok(true)
    }
    /// `RemoveAllNotes(theItemId, theDelIfOrphan)`.
    pub fn remove_all_notes(
        &self,
        item: &XCAFDocAssemblyItemId,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        self.remove_all(item, Annotated::Item, delete_orphan)
    }
    /// `RemoveAllNotes(theItemLabel, theDelIfOrphan)`.
    pub fn remove_all_notes_label(
        &self,
        label: &TdfLabel,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        self.remove_all_notes(&labeled_item(label), delete_orphan)
    }
    /// `RemoveAllSubshapeNotes(theItemId, theSubshapeIndex, theDelIfOrphan)`.
    pub fn remove_all_subshape_notes(
        &self,
        item: &XCAFDocAssemblyItemId,
        index: i32,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        self.remove_all(item, Annotated::Subshape(index), delete_orphan)
    }
    /// `RemoveAllAttrNotes(theItemId, theGUID, theDelIfOrphan)`.
    pub fn remove_all_attr_notes(
        &self,
        item: &XCAFDocAssemblyItemId,
        guid: &Guid,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        self.remove_all(item, Annotated::Attribute(*guid), delete_orphan)
    }
    /// `RemoveAllAttrNotes(theItemLabel, theGUID, theDelIfOrphan)`.
    pub fn remove_all_attr_notes_label(
        &self,
        label: &TdfLabel,
        guid: &Guid,
        delete_orphan: bool,
    ) -> Result<bool, Failure> {
        self.remove_all_attr_notes(&labeled_item(label), guid, delete_orphan)
    }
}

/// The deletion of notes.
impl XCAFDocNotesTool {
    /// `DeleteNote(theNoteLabel)`: unlinks the note from its items (forgetting the items left
    /// without notes) and forgets every attribute of the note label.
    pub fn delete_note(&self, note: &TdfLabel) -> Result<bool, Failure> {
        if !<dyn XCAFDocNote>::is_mine(note)? {
            return Ok(false);
        }
        let father = node_of(note, &XCAFDoc::note_ref_guid())?;
        if let Some(father_node) = father.get() {
            while father_node.nb_children() > 0 {
                let child = father_node.get_child(1)?;
                father_node.unset_child(&child)?;
                let child_node = live(&child, "XCAFDoc_NotesTool::DeleteNote")?;
                if child_node.nb_fathers() == 0 {
                    child_node.label().forget_all_attributes(true)?;
                }
            }
        }
        note.forget_all_attributes(true)?;
        Ok(true)
    }
    /// `DeleteNotes(theNoteLabels)`: the number of deleted notes.
    pub fn delete_notes(&self, notes: &[TdfLabel]) -> Result<i32, Failure> {
        let mut count = 0;
        for note in notes {
            count += i32::from(self.delete_note(note)?);
        }
        Ok(count)
    }
    /// `DeleteAllNotes`: the number of deleted notes.
    pub fn delete_all_notes(&self) -> Result<i32, Failure> {
        self.delete_notes(&child_labels(&self.get_notes_label()?)?)
    }
    /// `GetOrphanNotes(theNoteLabels)`: appends the notes without annotated items.
    pub fn get_orphan_notes(&self, notes: &mut Vec<TdfLabel>) -> Result<(), Failure> {
        for label in child_labels(&self.get_notes_label()?)? {
            let note = <dyn XCAFDocNote>::get(&label)?;
            if note.get().map(|n| n.is_orphan()).transpose()? == Some(true) {
                notes.push(label);
            }
        }
        Ok(())
    }
    /// `NbOrphanNotes`.
    pub fn nb_orphan_notes(&self) -> Result<i32, Failure> {
        let mut notes = Vec::new();
        self.get_orphan_notes(&mut notes)?;
        Ok(notes.len() as i32)
    }
    /// `DeleteOrphanNotes`: the number of deleted orphan notes.
    pub fn delete_orphan_notes(&self) -> Result<i32, Failure> {
        let mut count = 0;
        for label in child_labels(&self.get_notes_label()?)? {
            let note = <dyn XCAFDocNote>::get(&label)?;
            if note.get().map(|n| n.is_orphan()).transpose()? == Some(true)
                && self.delete_note(&label)?
            {
                count += 1;
            }
        }
        Ok(count)
    }
}
