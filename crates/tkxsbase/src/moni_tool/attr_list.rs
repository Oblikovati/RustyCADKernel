// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_AttrList (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MoniTool_AttrList`: a list of named attributes (integers, reals, texts or any
//! object).
use super::{MoniToolIntVal, MoniToolRealVal, MoniToolValueType};
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Handle, RuntimeType, RuntimeTyped, StandardTransient},
    tcollection::{AsciiString, HAsciiString},
};

/// The native `NCollection_DataMap<TCollection_AsciiString, occ::handle<Standard_Transient>>`
/// of the attributes.
pub type MoniToolAttrMap = DataMap<AsciiString, Handle<StandardTransient>>;

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
        if self.theattrib.is_empty() {
            return false;
        }
        self.theattrib.unbind(&AsciiString::new(name))
    }
    /// `GetAttribute`: the attribute `name` when it is kind of `atype`, else null and false.
    pub fn get_attribute(
        &self,
        name: &str,
        atype: &RuntimeType,
        val: &mut Handle<StandardTransient>,
    ) -> bool {
        *val = self.attribute(name);
        if val.is_null() || !val.is_kind(atype) {
            val.nullify();
            return false;
        }
        true
    }
    /// `Attribute`: the attribute `name`, null when not recorded.
    pub fn attribute(&self, name: &str) -> Handle<StandardTransient> {
        if self.theattrib.is_empty() {
            return Handle::default();
        }
        let found = self.theattrib.seek(&AsciiString::new(name));
        found.cloned().unwrap_or_default()
    }
    /// `AttributeType`: integer, real, text, ident (any other object) or void (not
    /// recorded); the exact dynamic type decides, as natively.
    pub fn attribute_type(&self, name: &str) -> MoniToolValueType {
        let atr = self.attribute(name);
        let Some(kind) = atr.dynamic_type() else {
            return MoniToolValueType::ValueVoid;
        };
        if std::ptr::eq(kind, MoniToolIntVal::static_type()) {
            return MoniToolValueType::ValueInteger;
        }
        if std::ptr::eq(kind, MoniToolRealVal::static_type()) {
            return MoniToolValueType::ValueReal;
        }
        if std::ptr::eq(kind, HAsciiString::static_type()) {
            return MoniToolValueType::ValueText;
        }
        MoniToolValueType::ValueIdent
    }
    /// `SetIntegerAttribute`.
    pub fn set_integer_attribute(&mut self, name: &str, val: i32) {
        self.set_attribute(name, &Handle::new(MoniToolIntVal::new(val)).cast());
    }
    /// `GetIntegerAttribute`: false and 0 when absent or not an integer.
    pub fn get_integer_attribute(&self, name: &str, val: &mut i32) -> bool {
        let ival = self.attribute(name).cast::<MoniToolIntVal>();
        *val = ival.get().map_or(0, MoniToolIntVal::value);
        ival.is_some()
    }
    /// `IntegerAttribute`: 0 when absent.
    pub fn integer_attribute(&self, name: &str) -> i32 {
        let ival = self.attribute(name).cast::<MoniToolIntVal>();
        ival.get().map_or(0, MoniToolIntVal::value)
    }
    /// `SetRealAttribute`.
    pub fn set_real_attribute(&mut self, name: &str, val: f64) {
        self.set_attribute(name, &Handle::new(MoniToolRealVal::new(val)).cast());
    }
    /// `GetRealAttribute`: false and 0.0 when absent or not a real.
    pub fn get_real_attribute(&self, name: &str, val: &mut f64) -> bool {
        let rval = self.attribute(name).cast::<MoniToolRealVal>();
        *val = rval.get().map_or(0.0, MoniToolRealVal::value);
        rval.is_some()
    }
    /// `RealAttribute`: 0.0 when absent.
    pub fn real_attribute(&self, name: &str) -> f64 {
        let rval = self.attribute(name).cast::<MoniToolRealVal>();
        rval.get().map_or(0.0, MoniToolRealVal::value)
    }
    /// `SetStringAttribute`.
    pub fn set_string_attribute(&mut self, name: &str, val: &str) {
        self.set_attribute(name, &Handle::new(HAsciiString::new(val)).cast());
    }
    /// `GetStringAttribute`: false and "" when absent or not a text.
    pub fn get_string_attribute(&self, name: &str, val: &mut AsciiString) -> bool {
        let hval = self.attribute(name).cast::<HAsciiString>();
        *val = hval
            .get()
            .map_or_else(AsciiString::default, |h| h.string().clone());
        hval.is_some()
    }
    /// `StringAttribute`: "" when absent (natively the C string of the shared text).
    pub fn string_attribute(&self, name: &str) -> AsciiString {
        let hval = self.attribute(name).cast::<HAsciiString>();
        hval.get()
            .map_or_else(AsciiString::default, |h| h.string().clone())
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
        for (name, atr) in other.theattrib.items() {
            if !name.starts_with(fromname) {
                continue;
            }
            let newatr = if copied { copy_basic(atr) } else { atr.clone() };
            self.theattrib.bind(name.clone(), newatr);
        }
    }
}

/// A new object for an integer, real or text attribute, else the attribute itself.
fn copy_basic(atr: &Handle<StandardTransient>) -> Handle<StandardTransient> {
    if let Some(ival) = atr.cast::<MoniToolIntVal>().get() {
        return Handle::new(MoniToolIntVal::new(ival.value())).cast();
    }
    if let Some(rval) = atr.cast::<MoniToolRealVal>().get() {
        return Handle::new(MoniToolRealVal::new(rval.value())).cast();
    }
    if let Some(hval) = atr.cast::<HAsciiString>().get() {
        return Handle::new(HAsciiString::copied(hval)).cast();
    }
    atr.clone()
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
