// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_AttrList (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MoniTool_AttrList`: a list of named attributes (integers, reals, texts or any
//! object).
use super::{MoniToolIntVal, MoniToolRealVal, MoniToolValueType};
use crate::attr_core::{self, AttributeBoxes, AttributeKind, AttributeMap};
use rustycad_tkernel::{
    standard::{Handle, RuntimeType, RuntimeTyped, StandardTransient},
    tcollection::{AsciiString, HAsciiString},
};

/// The native `NCollection_DataMap<TCollection_AsciiString, occ::handle<Standard_Transient>>`
/// of the attributes.
pub type MoniToolAttrMap = AttributeMap;

/// The MoniTool boxes of integer and real attributes.
struct MoniToolBoxes;

impl AttributeBoxes for MoniToolBoxes {
    fn new_integer(value: i32) -> Handle<StandardTransient> {
        Handle::new(MoniToolIntVal::new(value)).cast()
    }
    fn integer_of(atr: &Handle<StandardTransient>) -> Option<i32> {
        atr.cast::<MoniToolIntVal>()
            .get()
            .map(MoniToolIntVal::value)
    }
    fn integer_type() -> &'static RuntimeType {
        MoniToolIntVal::static_type()
    }
    fn new_real(value: f64) -> Handle<StandardTransient> {
        Handle::new(MoniToolRealVal::new(value)).cast()
    }
    fn real_of(atr: &Handle<StandardTransient>) -> Option<f64> {
        atr.cast::<MoniToolRealVal>()
            .get()
            .map(MoniToolRealVal::value)
    }
    fn real_type() -> &'static RuntimeType {
        MoniToolRealVal::static_type()
    }
}

/// `MoniTool_AttrList`: attributes by name. An integer is recorded as a
/// [`MoniToolIntVal`], a real as a [`MoniToolRealVal`], a text as an [`HAsciiString`].
///
/// ```
/// use rustycad_tkxsbase::moni_tool::{MoniToolAttrList, MoniToolValueType};
/// let mut list = MoniToolAttrList::new();
/// list.set_integer_attribute("count", 3);
/// assert_eq!(list.integer_attribute("count"), 3);
/// assert_eq!(list.attribute_type("count"), MoniToolValueType::ValueInteger);
/// ```
#[derive(Clone, Default)]
pub struct MoniToolAttrList {
    theattrib: MoniToolAttrMap,
}

impl MoniToolAttrList {
    /// `MoniTool_AttrList()`: empty. The copy constructor is `Clone`: it copies the map,
    /// the attribute objects are shared.
    pub fn new() -> Self {
        Self::default()
    }
    /// `SetAttribute`: adds or replaces the attribute `name`.
    pub fn set_attribute(&mut self, name: &str, val: &Handle<StandardTransient>) {
        self.theattrib.bind(AsciiString::new(name), val.clone());
    }
    /// `RemoveAttribute`: true when the attribute existed.
    pub fn remove_attribute(&mut self, name: &str) -> bool {
        attr_core::remove_attribute(&mut self.theattrib, name)
    }
    /// `GetAttribute`: the attribute `name` when it is kind of `atype`, else null and false.
    pub fn get_attribute(
        &self,
        name: &str,
        atype: &RuntimeType,
        val: &mut Handle<StandardTransient>,
    ) -> bool {
        attr_core::get_attribute(&self.theattrib, name, atype, val)
    }
    /// `Attribute`: the attribute `name`, null when not recorded.
    pub fn attribute(&self, name: &str) -> Handle<StandardTransient> {
        attr_core::attribute(&self.theattrib, name)
    }
    /// `AttributeType`: integer, real, text, ident (any other object) or void (not
    /// recorded); the exact dynamic type decides, as natively.
    pub fn attribute_type(&self, name: &str) -> MoniToolValueType {
        match attr_core::attribute_kind::<MoniToolBoxes>(&self.theattrib, name) {
            AttributeKind::Void => MoniToolValueType::ValueVoid,
            AttributeKind::Integer => MoniToolValueType::ValueInteger,
            AttributeKind::Real => MoniToolValueType::ValueReal,
            AttributeKind::Text => MoniToolValueType::ValueText,
            AttributeKind::Ident => MoniToolValueType::ValueIdent,
        }
    }
    /// `SetIntegerAttribute`.
    pub fn set_integer_attribute(&mut self, name: &str, val: i32) {
        self.set_attribute(name, &MoniToolBoxes::new_integer(val));
    }
    /// `GetIntegerAttribute`: false and 0 when absent or not an integer.
    pub fn get_integer_attribute(&self, name: &str, val: &mut i32) -> bool {
        let found = MoniToolBoxes::integer_of(&self.attribute(name));
        *val = found.unwrap_or(0);
        found.is_some()
    }
    /// `IntegerAttribute`: 0 when absent.
    pub fn integer_attribute(&self, name: &str) -> i32 {
        MoniToolBoxes::integer_of(&self.attribute(name)).unwrap_or(0)
    }
    /// `SetRealAttribute`.
    pub fn set_real_attribute(&mut self, name: &str, val: f64) {
        self.set_attribute(name, &MoniToolBoxes::new_real(val));
    }
    /// `GetRealAttribute`: false and 0.0 when absent or not a real.
    pub fn get_real_attribute(&self, name: &str, val: &mut f64) -> bool {
        let found = MoniToolBoxes::real_of(&self.attribute(name));
        *val = found.unwrap_or(0.0);
        found.is_some()
    }
    /// `RealAttribute`: 0.0 when absent.
    pub fn real_attribute(&self, name: &str) -> f64 {
        MoniToolBoxes::real_of(&self.attribute(name)).unwrap_or(0.0)
    }
    /// `SetStringAttribute`.
    pub fn set_string_attribute(&mut self, name: &str, val: &str) {
        self.set_attribute(name, &Handle::new(HAsciiString::new(val)).cast());
    }
    /// `GetStringAttribute`: false and "" when absent or not a text.
    pub fn get_string_attribute(&self, name: &str, val: &mut AsciiString) -> bool {
        let found = attr_core::string_attribute(&self.theattrib, name);
        *val = found.clone().unwrap_or_default();
        found.is_some()
    }
    /// `StringAttribute`: "" when absent (natively the C string of the shared text).
    pub fn string_attribute(&self, name: &str) -> AsciiString {
        attr_core::string_attribute(&self.theattrib, name).unwrap_or_default()
    }
    /// `AttrList`: the map of every attribute.
    pub fn attr_list(&self) -> &MoniToolAttrMap {
        &self.theattrib
    }
    /// `SameAttributes`: takes a copy of the map of `other`; the attribute objects are
    /// shared (the native assignment copies the map, xsbase-design.md section 5).
    pub fn same_attributes(&mut self, other: &MoniToolAttrList) {
        self.theattrib = other.theattrib.clone();
    }
    /// `GetAttributes(other, fromname, copied)`: adds the attributes of `other` whose name
    /// starts with `fromname`; with `copied`, integers, reals and texts are new objects.
    pub fn get_attributes(&mut self, other: &MoniToolAttrList, fromname: &str, copied: bool) {
        attr_core::get_attributes::<MoniToolBoxes>(
            &mut self.theattrib,
            &other.theattrib,
            fromname,
            copied,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_attributes_round_trip() {
        let mut list = MoniToolAttrList::new();
        list.set_real_attribute("r", 2.5);
        list.set_string_attribute("s", "text");
        let mut real = -1.0;
        assert!(list.get_real_attribute("r", &mut real));
        assert_eq!(real, 2.5);
        let mut text = AsciiString::new("x");
        assert!(list.get_string_attribute("s", &mut text));
        assert_eq!(text.as_bytes(), b"text");
        assert!(!list.get_integer_attribute("r", &mut 5));
        assert_eq!(list.attribute_type("s"), MoniToolValueType::ValueText);
        assert_eq!(list.attribute_type("none"), MoniToolValueType::ValueVoid);
        assert_eq!(list.string_attribute("r").as_bytes(), b"");
    }

    #[test]
    fn get_attribute_filters_by_kind() {
        let mut list = MoniToolAttrList::new();
        list.set_integer_attribute("i", 4);
        let mut found = Handle::default();
        assert!(list.get_attribute("i", MoniToolIntVal::static_type(), &mut found));
        assert!(!list.get_attribute("i", MoniToolRealVal::static_type(), &mut found));
        assert!(found.is_null());
        let any = Handle::new(MoniToolRealVal::new(1.0)).cast::<HAsciiString>();
        assert!(any.is_null());
    }

    #[test]
    fn remove_and_ident_attributes() {
        let mut list = MoniToolAttrList::new();
        assert!(!list.remove_attribute("a"));
        let object = Handle::new(StandardTransient::default());
        list.set_attribute("a", &object);
        assert_eq!(list.attribute_type("a"), MoniToolValueType::ValueIdent);
        assert!(list.remove_attribute("a"));
        assert!(list.attribute("a").is_null());
    }

    #[test]
    fn get_attributes_copies_basic_values_with_prefix() {
        let mut source = MoniToolAttrList::new();
        source.set_integer_attribute("x.a", 1);
        source.set_integer_attribute("y.b", 2);
        let mut copy = MoniToolAttrList::new();
        copy.get_attributes(&source, "x.", true);
        assert_eq!(copy.integer_attribute("x.a"), 1);
        assert!(copy.attribute("y.b").is_null());
        assert!(copy.attribute("x.a") != source.attribute("x.a"));
        let mut shared = MoniToolAttrList::new();
        shared.get_attributes(&source, "", false);
        assert!(shared.attribute("y.b") == source.attribute("y.b"));
        let mut same = MoniToolAttrList::new();
        same.same_attributes(&source);
        assert_eq!(same.attr_list().len(), 2);
    }
}
