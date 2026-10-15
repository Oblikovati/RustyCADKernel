// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_Location and XCAFDoc_LengthUnit (.hxx and
// .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_Location` (the location of a component) and `XCAFDoc_LengthUnit` (the length
//! unit of a document).
use super::support::{downcast, find_or_add, live};
use rustycad_tkernel::standard::dump::{DumpOutput, DumpStream, add_values_separator};
use rustycad_tkernel::standard::{Failure, Guid, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkernel::units_methods::{LengthUnit, UnitsMethods};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::json::dump_attribute_json_base;
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable, dump_attribute,
};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::json_dump::{class_begin, field_dumped};
use std::io;

/// `XCAFDoc_Location::GetID()`.
const LOCATION_ID: Guid = guid_from_text("efd212ef-6dfd-11d4-b9c8-0060b0ee281b");

/// `XCAFDoc_Location`.
///
/// ```
/// use rustycad_tklcaf::tdf::TdfData;
/// use rustycad_tkmath::toploc::Location;
/// use rustycad_tkxcaf::xcaf_doc::XCAFDocLocation;
/// let data = TdfData::new();
/// let location = XCAFDocLocation::set(&data.root(), &Location::default())?;
/// assert!(location.get().is_identity());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct XCAFDocLocation {
    transient: StandardTransient,
    base: TdfAttributeBase,
    location: SharedState<Location>,
}
rustycad_tkernel::impl_transient!(XCAFDocLocation, "XCAFDoc_Location", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl XCAFDocLocation {
    /// `XCAFDoc_Location()`: the identity.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        LOCATION_ID
    }
    /// `Set(L, Loc)`: finds or adds the attribute, then sets the location.
    pub fn set(label: &TdfLabel, location: &Location) -> Result<Handle<XCAFDocLocation>, Failure> {
        let attribute = find_or_add(label, &LOCATION_ID, Self::new)?;
        live(&attribute, "XCAFDoc_Location::Set")?.set_location(location)?;
        Ok(attribute)
    }
    /// `Set(Loc)`: backed up.
    pub fn set_location(&self, location: &Location) -> Result<(), Failure> {
        self.backup()?;
        *self.location.write() = location.clone();
        Ok(())
    }
    /// `Get`.
    pub fn get(&self) -> Location {
        self.location.read().clone()
    }
}

impl TdfAttribute for XCAFDocLocation {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        LOCATION_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = downcast::<XCAFDocLocation>(with, "XCAFDoc_Location::Restore")?;
        *self.location.write() = live(&other, "XCAFDoc_Location::Restore")?.get();
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = downcast::<XCAFDocLocation>(into, "XCAFDoc_Location::Paste")?;
        live(&target, "XCAFDoc_Location::Paste")?.set_location(&self.get())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFDoc_Location")?;
        dump_attribute_json_base(self, stream, depth)?;
        let location = self.get();
        field_dumped(stream, depth, "&myLocation", &|nested, d| {
            location.dump_json(nested, d)
        })
    }
}

/// `XCAFDoc_LengthUnit::GetID()` (the same GUID as `XCAFDoc_Material::GetID()` natively).
const LENGTH_UNIT_ID: Guid = guid_from_text("efd212f8-6dfd-11d4-b9c8-0060b0ee281b");

/// The fields of `XCAFDoc_LengthUnit`.
#[derive(Clone)]
struct LengthUnitFields {
    scale: f64,
    name: AsciiString,
}

/// `XCAFDoc_LengthUnit`: a unit name and its scale to the meter.
///
/// ```
/// use rustycad_tklcaf::tdf::TdfData;
/// use rustycad_tkxcaf::xcaf_doc::XCAFDocLengthUnit;
/// let data = TdfData::new();
/// let unit = XCAFDocLengthUnit::set_value(&data.root(), 0.001)?;
/// assert_eq!(unit.get_unit_name().to_string(), "mm");
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct XCAFDocLengthUnit {
    transient: StandardTransient,
    base: TdfAttributeBase,
    fields: SharedState<LengthUnitFields>,
}
rustycad_tkernel::impl_transient!(XCAFDocLengthUnit, "XCAFDoc_LengthUnit", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for XCAFDocLengthUnit {
    /// `XCAFDoc_LengthUnit()`: scale 1, empty name.
    fn default() -> Self {
        super::registration::register_derived_attributes();
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            fields: SharedState::new(LengthUnitFields {
                scale: 1.0,
                name: AsciiString::default(),
            }),
        }
    }
}

impl XCAFDocLengthUnit {
    /// `XCAFDoc_LengthUnit()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        LENGTH_UNIT_ID
    }
    /// `Set(theLabel, theUnitName, theUnitValue)`.
    pub fn set(
        label: &TdfLabel,
        name: &AsciiString,
        value: f64,
    ) -> Result<Handle<XCAFDocLengthUnit>, Failure> {
        Self::set_with_id(label, &LENGTH_UNIT_ID, name, value)
    }
    /// `Set(theLabel, theUnitValue)`: the name is the meter-based name of the value.
    pub fn set_value(label: &TdfLabel, value: f64) -> Result<Handle<XCAFDocLengthUnit>, Failure> {
        let name = UnitsMethods::dump_length_unit_factor(value, LengthUnit::Meter);
        Self::set_with_id(label, &LENGTH_UNIT_ID, &AsciiString::from(name), value)
    }
    /// `Set(theLabel, theGUID, theUnitName, theUnitValue)`: a new attribute gets `SetID`
    /// (which does nothing: the ID stays `GetID`, as natively).
    pub fn set_with_id(
        label: &TdfLabel,
        guid: &Guid,
        name: &AsciiString,
        value: f64,
    ) -> Result<Handle<XCAFDocLengthUnit>, Failure> {
        let attribute = find_or_add(label, guid, Self::new)?;
        live(&attribute, "XCAFDoc_LengthUnit::Set")?.set_unit(name, value)?;
        Ok(attribute)
    }
    /// `Set(theUnitName, theUnitValue)`: backed up.
    pub fn set_unit(&self, name: &AsciiString, value: f64) -> Result<(), Failure> {
        self.backup()?;
        let mut fields = self.fields.write();
        (fields.name, fields.scale) = (name.clone(), value);
        Ok(())
    }
    /// `GetUnitName`.
    pub fn get_unit_name(&self) -> AsciiString {
        self.fields.read().name.clone()
    }
    /// `GetUnitValue`.
    pub fn get_unit_value(&self) -> f64 {
        self.fields.read().scale
    }
    /// `IsEmpty`: no unit name.
    pub fn is_empty(&self) -> bool {
        self.fields.read().name.is_empty()
    }
}

impl TdfAttribute for XCAFDocLengthUnit {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        LENGTH_UNIT_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = downcast::<XCAFDocLengthUnit>(with, "XCAFDoc_LengthUnit::Restore")?;
        let other = live(&other, "XCAFDoc_LengthUnit::Restore")?
            .fields
            .read()
            .clone();
        *self.fields.write() = other;
        Ok(())
    }
    /// `NewEmpty` (from `DEFINE_DERIVED_ATTRIBUTE`).
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = downcast::<XCAFDocLengthUnit>(into, "XCAFDoc_LengthUnit::Paste")?;
        live(&target, "XCAFDoc_LengthUnit::Paste")?
            .set_unit(&self.get_unit_name(), self.get_unit_value())
    }
    /// `Dump`: the attribute line, then ` UnitName=|name| UnitScaleValue=|value|<guid>|`.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        dump_attribute(self, stream)?;
        write!(stream, " UnitName=|")?;
        stream.write_all(self.get_unit_name().as_bytes())?;
        write!(stream, "| UnitScaleValue=|")?;
        stream.write_all(AsciiString::from_real(self.get_unit_value()).as_bytes())?;
        writeln!(stream, "|{}|", self.id())
    }
    /// `DumpJson`. Natively the `OCCT_DUMP_FIELD_VALUES_*` macros receive the addresses of
    /// the fields as a `char*` and a `double` variadic argument (undefined behavior that
    /// prints the bytes of the object and a garbage real); the port writes the name and the
    /// scale they intend.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFDoc_LengthUnit")?;
        dump_attribute_json_base(self, stream, depth)?;
        let fields = self.fields.read().clone();
        add_values_separator(stream)?;
        stream.write_fragment(b"\"UnitName\": [\"")?;
        stream.write_fragment(fields.name.as_bytes())?;
        stream.write_fragment(b"\"]")?;
        rustycad_tkernel::standard::dump::dump_vector(stream, "UnitScaleValue", &[fields.scale])
    }
}
