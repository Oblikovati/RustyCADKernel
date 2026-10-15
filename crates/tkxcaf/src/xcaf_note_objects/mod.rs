// Rust port of DataExchange/TKXCAF/XCAFNoteObjects/XCAFNoteObjects_NoteObject.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The XCAFNoteObjects package: `XCAFNoteObjects_NoteObject`, the placement and
//! presentation of a note.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Axis2, Point3};

/// The fields of `XCAFNoteObjects_NoteObject`.
#[derive(Clone, Default)]
struct NoteFields {
    plane: Axis2,
    point: Point3,
    point_text: Point3,
    presentation: Shape,
    has_plane: bool,
    has_point: bool,
    has_point_text: bool,
}

/// `XCAFNoteObjects_NoteObject`.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tkxcaf::xcaf_note_objects::XCAFNoteObjectsNoteObject;
/// let note = XCAFNoteObjectsNoteObject::new();
/// note.set_point(&Point3::new(1.0, 2.0, 3.0));
/// assert!(note.has_point());
/// note.reset();
/// assert!(!note.has_point());
/// ```
#[derive(Default)]
pub struct XCAFNoteObjectsNoteObject {
    transient: StandardTransient,
    fields: SharedState<NoteFields>,
}
rustycad_tkernel::impl_transient!(
    XCAFNoteObjectsNoteObject,
    "XCAFNoteObjects_NoteObject",
    StandardTransient,
    transient
);

impl XCAFNoteObjectsNoteObject {
    /// `XCAFNoteObjects_NoteObject()`: no plane, points or presentation.
    pub fn new() -> Self {
        Self::default()
    }
    /// `XCAFNoteObjects_NoteObject(theObj)`: a copy (a null object gives a default object).
    pub fn from_object(object: &Handle<XCAFNoteObjectsNoteObject>) -> Self {
        let fields = object
            .get()
            .map(|o| o.fields.read().clone())
            .unwrap_or_default();
        Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
    /// `HasPlane`.
    pub fn has_plane(&self) -> bool {
        self.fields.read().has_plane
    }
    /// `GetPlane`.
    pub fn get_plane(&self) -> Axis2 {
        self.fields.read().plane
    }
    /// `SetPlane`.
    pub fn set_plane(&self, plane: &Axis2) {
        let mut fields = self.fields.write();
        (fields.plane, fields.has_plane) = (*plane, true);
    }
    /// `HasPoint`.
    pub fn has_point(&self) -> bool {
        self.fields.read().has_point
    }
    /// `GetPoint`.
    pub fn get_point(&self) -> Point3 {
        self.fields.read().point
    }
    /// `SetPoint`.
    pub fn set_point(&self, point: &Point3) {
        let mut fields = self.fields.write();
        (fields.point, fields.has_point) = (*point, true);
    }
    /// `HasPointText`.
    pub fn has_point_text(&self) -> bool {
        self.fields.read().has_point_text
    }
    /// `GetPointText`.
    pub fn get_point_text(&self) -> Point3 {
        self.fields.read().point_text
    }
    /// `SetPointText`.
    pub fn set_point_text(&self, point: &Point3) {
        let mut fields = self.fields.write();
        (fields.point_text, fields.has_point_text) = (*point, true);
    }
    /// `GetPresentation`.
    pub fn get_presentation(&self) -> Shape {
        self.fields.read().presentation.clone()
    }
    /// `SetPresentation`.
    pub fn set_presentation(&self, presentation: &Shape) {
        self.fields.write().presentation = presentation.clone();
    }
    /// `Reset`: no plane or points, a null presentation (the values stay).
    pub fn reset(&self) {
        let mut fields = self.fields.write();
        (fields.has_plane, fields.has_point, fields.has_point_text) = (false, false, false);
        fields.presentation = Shape::default();
    }
}
