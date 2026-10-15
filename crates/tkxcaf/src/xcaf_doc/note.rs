// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_Note.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_Note`: the abstract base of the note attributes (comments, balloons and binary
//! data): the author and time stamp, the orphan test and the note object kept in the
//! children of the label.
use super::object_storage::{child, generate_shape, named_shape};
use super::support::{child_labels, find};
use super::{XCAFDoc, XCAFDocGraphNode};
use crate::xcaf_note_objects::XCAFNoteObjectsNoteObject;
use rustycad_tkcaf::tdata_xtd::{TDataXtdGeometry, TDataXtdPlane, TDataXtdPoint};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::json::dump_attribute_json_base;
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeIterator, TdfAttributeState, TdfLabel, dump_attribute,
};
use rustycad_tkmath::gp::{Axis3, Plane3, Point3};
use rustycad_tkservice::json_dump::{class_begin, field_string};
use std::io;

/// The child labels of a note (`ChildLab`).
mod child_lab {
    pub const PNT_TEXT: i32 = 1;
    pub const PLANE: i32 = 2;
    pub const PNT: i32 = 3;
    pub const PRESENTATION: i32 = 4;
}
use child_lab as lab;

/// The fields of `XCAFDoc_Note`: the author and the time stamp.
#[derive(Default)]
pub struct XCAFDocNoteFields {
    user_name: SharedState<ExtendedString>,
    time_stamp: SharedState<ExtendedString>,
}

impl XCAFDocNoteFields {
    /// Sets the author and time stamp without backup (the constructors of the notes).
    pub(super) fn assign(&self, user_name: &ExtendedString, time_stamp: &ExtendedString) {
        *self.user_name.write() = user_name.clone();
        *self.time_stamp.write() = time_stamp.clone();
    }
}

/// `XCAFDoc_Note`: a note class; implementers return their embedded fields.
pub trait XCAFDocNote: TdfAttribute {
    /// The embedded note fields.
    fn note_fields(&self) -> &XCAFDocNoteFields;
}
rustycad_tkernel::impl_transient_interface!(dyn XCAFDocNote, "XCAFDoc_Note", dyn TdfAttribute);

/// `XCAFDoc_Note::IsMine(theLabel)`: the label holds a note.
pub fn xcaf_doc_note_is_mine(label: &TdfLabel) -> Result<bool, Failure> {
    Ok(!xcaf_doc_note_get(label)?.is_null())
}

/// `XCAFDoc_Note::Get(theLabel)`: the first note attribute of the label, or null.
pub fn xcaf_doc_note_get(label: &TdfLabel) -> Result<Handle<dyn XCAFDocNote>, Failure> {
    let mut iterator = TdfAttributeIterator::new(label, true)?;
    while iterator.more() {
        let note = iterator.value().cast::<dyn XCAFDocNote>();
        if !note.is_null() {
            return Ok(note);
        }
        iterator.next();
    }
    Ok(Handle::default())
}

/// The non-virtual members of `XCAFDoc_Note`.
pub trait XCAFDocNoteMembers: XCAFDocNote {
    /// `Set(theUserName, theTimeStamp)`: backed up.
    fn set_note(
        &self,
        user_name: &ExtendedString,
        time_stamp: &ExtendedString,
    ) -> Result<(), Failure> {
        self.backup()?;
        self.note_fields().assign(user_name, time_stamp);
        Ok(())
    }
    /// `UserName`.
    fn user_name(&self) -> ExtendedString {
        self.note_fields().user_name.read().clone()
    }
    /// `TimeStamp`.
    fn time_stamp(&self) -> ExtendedString {
        self.note_fields().time_stamp.read().clone()
    }
    /// `IsOrphan`: no `NoteRefGUID` node, or one without annotated items.
    fn is_orphan(&self) -> Result<bool, Failure> {
        let node = find::<XCAFDocGraphNode>(&self.label(), &XCAFDoc::note_ref_guid())?;
        Ok(node.get().is_none_or(|node| node.nb_children() == 0))
    }
    /// `GetObject`: a new note object with the stored point, plane, text point and
    /// presentation (the children are created when absent, as natively).
    fn get_object(&self) -> Result<Handle<XCAFNoteObjectsNoteObject>, Failure> {
        let label = self.label();
        let object = XCAFNoteObjectsNoteObject::new();
        if let Some(point) = stored_point(&child(&label, lab::PNT)?)? {
            object.set_point(&point);
        }
        if let Some(plane) = stored_plane(&child(&label, lab::PLANE)?)? {
            object.set_plane(&plane.position().ax2()?);
        }
        if let Some(point) = stored_point(&child(&label, lab::PNT_TEXT)?)? {
            object.set_point_text(&point);
        }
        if let Some(shape) = named_shape(&child(&label, lab::PRESENTATION)?)? {
            object.set_presentation(&shape);
        }
        Ok(Handle::new(object))
    }
    /// `SetObject(theObject)`: backed up; the set values of the object on fresh children
    /// (`Standard_NullObject` for a null object).
    fn set_object(&self, object: &Handle<XCAFNoteObjectsNoteObject>) -> Result<(), Failure> {
        self.backup()?;
        let label = self.label();
        for child_label in child_labels(&label)? {
            child_label.forget_all_attributes(true)?;
        }
        let object = rustycad_tklcaf::live(object, "XCAFDoc_Note::SetObject")?;
        if object.has_point() {
            TDataXtdPoint::set_point(&child(&label, lab::PNT)?, object.get_point())?;
        }
        if object.has_plane() {
            TDataXtdPlane::set_plane(
                &child(&label, lab::PLANE)?,
                &Plane3::from_position(Axis3::from_axis2(object.get_plane())),
            )?;
        }
        if object.has_point_text() {
            TDataXtdPoint::set_point(&child(&label, lab::PNT_TEXT)?, object.get_point_text())?;
        }
        let presentation = object.get_presentation();
        if !presentation.is_null() {
            generate_shape(&child(&label, lab::PRESENTATION)?, &presentation)?;
        }
        Ok(())
    }
}
impl<T: XCAFDocNote + ?Sized> XCAFDocNoteMembers for T {}

/// The point of the label's `TDataXtd_Point`, when it has one with a geometry.
fn stored_point(label: &TdfLabel) -> Result<Option<Point3>, Failure> {
    if !label.is_attribute(&TDataXtdPoint::get_id())? {
        return Ok(None);
    }
    let mut point = Point3::default();
    Ok(TDataXtdGeometry::point_of_label(label, &mut point)?.then_some(point))
}

/// The plane of the label's `TDataXtd_Plane`, when it has one with a geometry.
fn stored_plane(label: &TdfLabel) -> Result<Option<Plane3>, Failure> {
    if !label.is_attribute(&TDataXtdPlane::get_id())? {
        return Ok(None);
    }
    let mut plane = Plane3::default();
    Ok(TDataXtdGeometry::plane_of_label(label, &mut plane)?.then_some(plane))
}

/// `XCAFDoc_Note::Restore`: the author and time stamp of the other note
/// (`Standard_NullObject` when it is not a note).
pub fn xcaf_doc_note_restore<A: XCAFDocNote + ?Sized>(
    note: &A,
    with: &Handle<dyn TdfAttribute>,
) -> Result<(), Failure> {
    let other = with.cast::<dyn XCAFDocNote>();
    let other = rustycad_tklcaf::live(&other, "XCAFDoc_Note::Restore")?;
    note.note_fields()
        .assign(&other.user_name(), &other.time_stamp());
    Ok(())
}

/// `XCAFDoc_Note::Paste`: sets the author and time stamp of the target note
/// (`Standard_NullObject` when it is not a note).
pub fn xcaf_doc_note_paste<A: XCAFDocNote + ?Sized>(
    note: &A,
    into: &Handle<dyn TdfAttribute>,
) -> Result<(), Failure> {
    let target = into.cast::<dyn XCAFDocNote>();
    rustycad_tklcaf::live(&target, "XCAFDoc_Note::Paste")?
        .set_note(&note.user_name(), &note.time_stamp())
}

/// `XCAFDoc_Note::Dump`: the attribute dump, then the author and time stamp (the native
/// conditions are inverted: an empty author prints itself, a set one `<anonymous>`).
pub fn xcaf_doc_note_dump<A: XCAFDocNote + ?Sized>(
    note: &A,
    stream: &mut dyn io::Write,
) -> io::Result<()> {
    dump_attribute(note, stream)?;
    let (user, stamp) = (note.user_name(), note.time_stamp());
    let user = if user.is_empty() {
        user.to_utf8()
    } else {
        "<anonymous>".to_owned()
    };
    let stamp = if stamp.is_empty() {
        stamp.to_utf8()
    } else {
        "<unknown>".to_owned()
    };
    write!(stream, "Note : {user} on {stamp}")
}

/// `XCAFDoc_Note::DumpJson`: the class of the dumped object, the attribute base, the author
/// and the time stamp.
pub fn xcaf_doc_note_dump_json<A: XCAFDocNote + ?Sized>(
    note: &A,
    class: &str,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, class)?;
    dump_attribute_json_base(note, stream, depth)?;
    field_string(stream, "myUserName", note.user_name().to_utf8().as_bytes())?;
    field_string(
        stream,
        "myTimeStamp",
        note.time_stamp().to_utf8().as_bytes(),
    )
}
