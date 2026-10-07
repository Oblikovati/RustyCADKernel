//! The attribute lists of `MoniTool_AttrList` and `Transfer_Finder`: the two native
//! classes have the same code over a map of named transients, and differ only in the
//! objects that box an integer and a real (`MoniTool_IntVal`/`MoniTool_RealVal` versus
//! `Interface_IntVal`/`Geom2d_CartesianPoint`). The code lives here once, over
//! [`AttributeBoxes`].
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Handle, RuntimeType, RuntimeTyped, StandardTransient},
    tcollection::{AsciiString, HAsciiString},
};

/// A map of named attributes (`NCollection_DataMap<TCollection_AsciiString,
/// occ::handle<Standard_Transient>>`).
pub type AttributeMap = DataMap<AsciiString, Handle<StandardTransient>>;

/// The kind of an attribute, by the exact dynamic type of its object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AttributeKind {
    Void,
    Integer,
    Real,
    Text,
    Ident,
}

/// The objects that box an integer and a real attribute.
pub(crate) trait AttributeBoxes {
    fn new_integer(value: i32) -> Handle<StandardTransient>;
    fn integer_of(atr: &Handle<StandardTransient>) -> Option<i32>;
    fn integer_type() -> &'static RuntimeType;
    fn new_real(value: f64) -> Handle<StandardTransient>;
    fn real_of(atr: &Handle<StandardTransient>) -> Option<f64>;
    fn real_type() -> &'static RuntimeType;
}

/// `Attribute(name)`: null when not recorded.
pub(crate) fn attribute(map: &AttributeMap, name: &str) -> Handle<StandardTransient> {
    if map.is_empty() {
        return Handle::default();
    }
    map.seek(&AsciiString::new(name))
        .cloned()
        .unwrap_or_default()
}

/// `RemoveAttribute(name)`.
pub(crate) fn remove_attribute(map: &mut AttributeMap, name: &str) -> bool {
    !map.is_empty() && map.unbind(&AsciiString::new(name))
}

/// `GetAttribute(name, type, val)`: the attribute when kind of `atype`, else null and false.
pub(crate) fn get_attribute(
    map: &AttributeMap,
    name: &str,
    atype: &RuntimeType,
    val: &mut Handle<StandardTransient>,
) -> bool {
    *val = attribute(map, name);
    if val.is_null() || !val.is_kind(atype) {
        val.nullify();
        return false;
    }
    true
}

/// `AttributeType(name)`: by the exact dynamic type, as natively.
pub(crate) fn attribute_kind<B: AttributeBoxes>(map: &AttributeMap, name: &str) -> AttributeKind {
    let atr = attribute(map, name);
    let Some(kind) = atr.dynamic_type() else {
        return AttributeKind::Void;
    };
    if std::ptr::eq(kind, B::integer_type()) {
        return AttributeKind::Integer;
    }
    if std::ptr::eq(kind, B::real_type()) {
        return AttributeKind::Real;
    }
    if std::ptr::eq(kind, HAsciiString::static_type()) {
        return AttributeKind::Text;
    }
    AttributeKind::Ident
}

/// `GetStringAttribute(name, val)`: false and "" when absent or not a text.
pub(crate) fn string_attribute(map: &AttributeMap, name: &str) -> Option<AsciiString> {
    let hval = attribute(map, name).cast::<HAsciiString>();
    hval.get().map(|h| h.string().clone())
}

/// `GetAttributes(other, fromname, copied)`: adds the attributes of `other` whose name
/// starts with `fromname`; with `copied`, integers, reals and texts are new objects.
pub(crate) fn get_attributes<B: AttributeBoxes>(
    map: &mut AttributeMap,
    other: &AttributeMap,
    fromname: &str,
    copied: bool,
) {
    for (name, atr) in other.items() {
        if !name.starts_with(fromname) {
            continue;
        }
        let newatr = if copied {
            copy_basic::<B>(atr)
        } else {
            atr.clone()
        };
        map.bind(name.clone(), newatr);
    }
}

/// A new object for an integer, real or text attribute, else the attribute itself.
fn copy_basic<B: AttributeBoxes>(atr: &Handle<StandardTransient>) -> Handle<StandardTransient> {
    if let Some(value) = B::integer_of(atr) {
        return B::new_integer(value);
    }
    if let Some(value) = B::real_of(atr) {
        return B::new_real(value);
    }
    if let Some(hval) = atr.cast::<HAsciiString>().get() {
        return Handle::new(HAsciiString::copied(hval)).cast();
    }
    atr.clone()
}
