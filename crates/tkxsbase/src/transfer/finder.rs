// Rust port of DataExchange/TKXSBase/Transfer/Transfer_Finder (.hxx, .cxx) and
// Transfer_FindHasher.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Transfer_Finder`: maps any object as a key of a transfer process, with the hash code
//! of the real key and a list of attributes; `Transfer_FindHasher` hashes finders by that
//! code and compares them with `Equates`.
use crate::attr_core::{self, AttributeBoxes, AttributeKind, AttributeMap};
use crate::interface::{InterfaceIntVal, InterfaceParamType};
use rustycad_tkernel::{
    impl_transient_interface,
    ncollection::HashPolicy,
    standard::{Handle, RuntimeType, RuntimeTyped, SharedState, StandardTransient, Transient},
    tcollection::{AsciiString, HAsciiString},
};
use rustycad_tkg2d::geom2d::{Geom2dCartesianPoint, Geom2dPoint};
use std::sync::RwLockWriteGuard;

/// The private fields of `Transfer_Finder`: the hash code and the attributes.
#[derive(Default)]
pub struct TransferFinderBase {
    thecode: SharedState<usize>,
    theattrib: SharedState<AttributeMap>,
}

/// The Transfer boxes of integer and real attributes: `Interface_IntVal` and (natively)
/// a `Geom2d_CartesianPoint` whose X is the real.
struct TransferBoxes;

impl AttributeBoxes for TransferBoxes {
    fn new_integer(value: i32) -> Handle<StandardTransient> {
        let ival = InterfaceIntVal::new();
        *ival.c_value() = value;
        Handle::new(ival).cast()
    }
    fn integer_of(atr: &Handle<StandardTransient>) -> Option<i32> {
        atr.cast::<InterfaceIntVal>()
            .get()
            .map(InterfaceIntVal::value)
    }
    fn integer_type() -> &'static RuntimeType {
        InterfaceIntVal::static_type()
    }
    fn new_real(value: f64) -> Handle<StandardTransient> {
        Handle::new(Geom2dCartesianPoint::new(value, 0.0)).cast()
    }
    fn real_of(atr: &Handle<StandardTransient>) -> Option<f64> {
        atr.cast::<Geom2dCartesianPoint>().get().map(|p| p.x())
    }
    fn real_type() -> &'static RuntimeType {
        Geom2dCartesianPoint::static_type()
    }
}

/// `Transfer_Finder`, abstract: `Equates` is required; the attributes are provided.
///
/// ```
/// use rustycad_tkxsbase::transfer::{TransferFinder, TransferTransientMapper};
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let mapper = TransferTransientMapper::new(&Handle::new(StandardTransient::default()));
/// mapper.set_integer_attribute("n", 4);
/// assert_eq!(mapper.integer_attribute("n"), 4);
/// ```
pub trait TransferFinder: Transient {
    /// The embedded private fields.
    fn finder_base(&self) -> &TransferFinderBase;
    /// `Equates(other)`: same true type and same value.
    fn equates(&self, other: &Handle<dyn TransferFinder>) -> bool;
    /// `ValueType`: by default the dynamic type of the finder.
    fn value_type(&self) -> &'static RuntimeType {
        self.dynamic_type()
    }
    /// `ValueTypeName`: by default "(finder)".
    fn value_type_name(&self) -> String {
        String::from("(finder)")
    }
    /// `GetHashCode`.
    fn get_hash_code(&self) -> usize {
        *self.finder_base().thecode.read()
    }
    /// `SetHashCode(code)` (protected).
    fn set_hash_code(&self, code: usize) {
        *self.finder_base().thecode.write() = code;
    }
    /// `SetAttribute(name, val)`.
    fn set_attribute(&self, name: &str, val: &Handle<StandardTransient>) {
        self.finder_base()
            .theattrib
            .write()
            .bind(AsciiString::new(name), val.clone());
    }
    /// `RemoveAttribute(name)`.
    fn remove_attribute(&self, name: &str) -> bool {
        attr_core::remove_attribute(&mut self.finder_base().theattrib.write(), name)
    }
    /// `GetAttribute(name, type, val)`.
    fn get_attribute(
        &self,
        name: &str,
        atype: &RuntimeType,
        val: &mut Handle<StandardTransient>,
    ) -> bool {
        attr_core::get_attribute(&self.finder_base().theattrib.read(), name, atype, val)
    }
    /// `Attribute(name)`.
    fn attribute(&self, name: &str) -> Handle<StandardTransient> {
        attr_core::attribute(&self.finder_base().theattrib.read(), name)
    }
    /// `AttributeType(name)`: integer, real (a `Geom2d_CartesianPoint`), text, ident or
    /// void, by the exact dynamic type.
    fn attribute_type(&self, name: &str) -> InterfaceParamType {
        match attr_core::attribute_kind::<TransferBoxes>(&self.finder_base().theattrib.read(), name)
        {
            AttributeKind::Void => InterfaceParamType::ParamVoid,
            AttributeKind::Integer => InterfaceParamType::ParamInteger,
            AttributeKind::Real => InterfaceParamType::ParamReal,
            AttributeKind::Text => InterfaceParamType::ParamText,
            AttributeKind::Ident => InterfaceParamType::ParamIdent,
        }
    }
    /// `SetIntegerAttribute(name, val)`.
    fn set_integer_attribute(&self, name: &str, val: i32) {
        self.set_attribute(name, &TransferBoxes::new_integer(val));
    }
    /// `GetIntegerAttribute(name, val)`.
    fn get_integer_attribute(&self, name: &str, val: &mut i32) -> bool {
        let found = TransferBoxes::integer_of(&self.attribute(name));
        *val = found.unwrap_or(0);
        found.is_some()
    }
    /// `IntegerAttribute(name)`.
    fn integer_attribute(&self, name: &str) -> i32 {
        TransferBoxes::integer_of(&self.attribute(name)).unwrap_or(0)
    }
    /// `SetRealAttribute(name, val)`: a `Geom2d_CartesianPoint` (val, 0), as natively.
    fn set_real_attribute(&self, name: &str, val: f64) {
        self.set_attribute(name, &TransferBoxes::new_real(val));
    }
    /// `GetRealAttribute(name, val)`.
    fn get_real_attribute(&self, name: &str, val: &mut f64) -> bool {
        let found = TransferBoxes::real_of(&self.attribute(name));
        *val = found.unwrap_or(0.0);
        found.is_some()
    }
    /// `RealAttribute(name)`.
    fn real_attribute(&self, name: &str) -> f64 {
        TransferBoxes::real_of(&self.attribute(name)).unwrap_or(0.0)
    }
    /// `SetStringAttribute(name, val)`.
    fn set_string_attribute(&self, name: &str, val: &str) {
        self.set_attribute(name, &Handle::new(HAsciiString::new(val)).cast());
    }
    /// `GetStringAttribute(name, val)`.
    fn get_string_attribute(&self, name: &str, val: &mut AsciiString) -> bool {
        let found = attr_core::string_attribute(&self.finder_base().theattrib.read(), name);
        *val = found.clone().unwrap_or_default();
        found.is_some()
    }
    /// `StringAttribute(name)`.
    fn string_attribute(&self, name: &str) -> AsciiString {
        attr_core::string_attribute(&self.finder_base().theattrib.read(), name).unwrap_or_default()
    }
    /// `AttrList`: the attribute map (the write guard; natively a modifiable reference).
    fn attr_list(&self) -> RwLockWriteGuard<'_, AttributeMap> {
        self.finder_base().theattrib.write()
    }
    /// `SameAttributes(other)`: a copy of the map of `other` (nothing for null).
    fn same_attributes(&self, other: &Handle<dyn TransferFinder>) {
        let Some(live) = other.get() else { return };
        let copy = live.finder_base().theattrib.read().clone();
        *self.finder_base().theattrib.write() = copy;
    }
    /// `GetAttributes(other, fromname = "", copied = true)`.
    fn get_attributes(&self, other: &Handle<dyn TransferFinder>, fromname: &str, copied: bool) {
        let Some(live) = other.get() else { return };
        let source = live.finder_base().theattrib.read().clone();
        attr_core::get_attributes::<TransferBoxes>(
            &mut self.finder_base().theattrib.write(),
            &source,
            fromname,
            copied,
        );
    }
}
impl_transient_interface!(dyn TransferFinder, "Transfer_Finder", StandardTransient);

/// `Transfer_FindHasher`: the hash code of a finder, and `Equates` (false for a null
/// first finder).
#[derive(Clone, Copy, Debug, Default)]
pub struct TransferFindHasher;

impl HashPolicy<Handle<dyn TransferFinder>> for TransferFindHasher {
    fn hash(&self, key: &Handle<dyn TransferFinder>) -> usize {
        key.get().map_or(0, |f| f.get_hash_code())
    }
    fn equal(
        &self,
        first: &Handle<dyn TransferFinder>,
        second: &Handle<dyn TransferFinder>,
    ) -> bool {
        first.get().is_some_and(|f| f.equates(second))
    }
}
