// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_Name.hxx and the accessors, Paste
// and DumpJson of TNaming_Name.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_Name`: the recipe that re-finds a selected shape (its algorithm, arguments and
//! context). `Solve` is in `name_solve.rs`.
use super::{TNamingNameType, TNamingNamedShape};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkcdf::json_dump::{class_begin, field_dumped, field_integer, field_string};
use rustycad_tkernel::standard::{Handle, dump::DumpStream};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tklcaf::json::dump_label_entry;
use rustycad_tklcaf::tdf::{TdfAttribute, TdfLabel, TdfRelocationTable};
use std::io;

/// `TNaming_Name`. The native constructor leaves the shape type and the orientation
/// uninitialized; the OCCT allocator clears new memory (`MMGT_CLEAR`), so they read as
/// `TopAbs_COMPOUND` and `TopAbs_FORWARD`, which the port uses.
///
/// ```
/// use rustycad_tkcaf::tnaming::{TNamingName, TNamingNameType};
/// let mut name = TNamingName::new();
/// assert_eq!(name.name_type(), TNamingNameType::Unknown);
/// assert_eq!(name.index(), -1);
/// name.set_type(TNamingNameType::Identity);
/// assert_eq!(name.name_type(), TNamingNameType::Identity);
/// ```
#[derive(Clone)]
pub struct TNamingName {
    name_type: TNamingNameType,
    shape_type: ShapeEnum,
    args: Vec<Handle<TNamingNamedShape>>,
    stop: Handle<TNamingNamedShape>,
    index: i32,
    shape: Shape,
    context_label: TdfLabel,
    orientation: Orientation,
}

impl Default for TNamingName {
    fn default() -> Self {
        Self::new()
    }
}

impl TNamingName {
    /// `TNaming_Name()`: type `UNKNOWN`, index -1.
    pub fn new() -> Self {
        Self {
            name_type: TNamingNameType::Unknown,
            shape_type: ShapeEnum::Compound,
            args: Vec::new(),
            stop: Handle::default(),
            index: -1,
            shape: Shape::default(),
            context_label: TdfLabel::default(),
            orientation: Orientation::Forward,
        }
    }
    /// `Type(aType)`.
    pub fn set_type(&mut self, name_type: TNamingNameType) {
        self.name_type = name_type;
    }
    /// `ShapeType(aType)`.
    pub fn set_shape_type(&mut self, shape_type: ShapeEnum) {
        self.shape_type = shape_type;
    }
    /// `Shape(theShape)`.
    pub fn set_shape(&mut self, shape: &Shape) {
        self.shape = shape.clone();
    }
    /// `Append(arg)`.
    pub fn append(&mut self, argument: &Handle<TNamingNamedShape>) {
        self.args.push(argument.clone());
    }
    /// `StopNamedShape(arg)`.
    pub fn set_stop_named_shape(&mut self, stop: &Handle<TNamingNamedShape>) {
        self.stop = stop.clone();
    }
    /// `Index(I)`.
    pub fn set_index(&mut self, index: i32) {
        self.index = index;
    }
    /// `ContextLabel(theLab)`.
    pub fn set_context_label(&mut self, label: &TdfLabel) {
        self.context_label = label.clone();
    }
    /// `Orientation(theOrientation)`.
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
    /// `Type()`.
    pub fn name_type(&self) -> TNamingNameType {
        self.name_type
    }
    /// `ShapeType()`.
    pub fn shape_type(&self) -> ShapeEnum {
        self.shape_type
    }
    /// `Shape()`.
    pub fn shape(&self) -> Shape {
        self.shape.clone()
    }
    /// `Arguments()`.
    pub fn arguments(&self) -> &[Handle<TNamingNamedShape>] {
        &self.args
    }
    /// `StopNamedShape()`.
    pub fn stop_named_shape(&self) -> Handle<TNamingNamedShape> {
        self.stop.clone()
    }
    /// `Index()`.
    pub fn index(&self) -> i32 {
        self.index
    }
    /// `ContextLabel()`.
    pub fn context_label(&self) -> &TdfLabel {
        &self.context_label
    }
    /// `Orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }
    /// `Paste(into, RT)`: the copy with relocated arguments, stop and context. The
    /// orientation is not copied. An argument without relocation is appended as the
    /// previous relocated one (the native template keeps its output unchanged when the
    /// relocation fails).
    pub fn paste(&self, into: &mut TNamingName, relocation_table: &TdfRelocationTable) {
        into.name_type = self.name_type;
        into.shape_type = self.shape_type;
        into.shape = self.shape.clone();
        into.index = self.index;
        into.args.clear();
        let mut relocated: Handle<TNamingNamedShape> = Handle::default();
        for argument in &self.args {
            relocation_table.has_attribute_relocation_as(&argument.cast(), &mut relocated);
            into.args.push(relocated.clone());
        }
        if self.stop.is_some() {
            relocation_table.has_attribute_relocation_as(&self.stop.cast(), &mut relocated);
            into.stop = relocated;
        }
        if !self.context_label.is_null() {
            relocation_table.has_label_relocation(&self.context_label, &mut into.context_label);
        }
    }
    /// `DumpJson`: the type and shape type (as integers), each argument, the index, the
    /// shape, the context label entry and the orientation.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TNaming_Name")?;
        field_string(stream, "myType", &(self.name_type as i32).to_string())?;
        field_string(stream, "myShapeType", &(self.shape_type as i32).to_string())?;
        for argument in &self.args {
            let child = argument
                .get()
                .map(|named| move |s: &mut DumpStream, d: i32| named.dump_json(s, d));
            let child = child
                .as_ref()
                .map(|f| f as &dyn Fn(&mut DumpStream, i32) -> io::Result<()>);
            field_dumped(stream, depth, "anArg.get()", child)?;
        }
        field_integer(stream, "myIndex", self.index)?;
        let shape = |s: &mut DumpStream, d: i32| self.shape.dump_json(s, d);
        field_dumped(stream, depth, "&myShape", Some(&shape))?;
        dump_label_entry(stream, "aLabel", &self.context_label)?;
        field_string(
            stream,
            "myOrientation",
            &(self.orientation as i32).to_string(),
        )
    }
}
