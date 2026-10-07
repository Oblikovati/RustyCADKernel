// Rust port of DataExchange/TKXSBase/Interface/Interface_Static (.hxx, .cxx: the class
// members) and Interface_StaticSatisfies.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_Static`: a typed value recorded by name in the process-wide map of static
//! values (`MoniTool_TypedValue::Stats`), with its family, a wild-card and an update
//! status. The static functions on the map are in `static_registry`.
use super::{InterfaceParamType, InterfaceTypedValue};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState},
    tcollection::{AsciiString, HAsciiString},
};
use std::io::{self, Write};
use std::ops::Deref;

/// `Interface_StaticSatisfies`: an additional check of a static value.
pub type InterfaceStaticSatisfies = fn(&Handle<HAsciiString>) -> bool;

/// The own fields of a static value.
struct StaticState {
    thefamily: AsciiString,
    thewild: Handle<InterfaceStatic>,
    theupdate: bool,
}

/// `Interface_Static`: embeds its `Interface_TypedValue` (registered as its RTTI parent)
/// and dereferences to it.
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceParamType, InterfaceStatic};
/// let value = InterfaceStatic::new("XSTEP", "doc.static", InterfaceParamType::ParamReal, "0.5").unwrap();
/// assert_eq!(value.family().as_bytes(), b"XSTEP");
/// assert_eq!(value.real_value(), 0.5);
/// ```
pub struct InterfaceStatic {
    base: InterfaceTypedValue,
    state: SharedState<StaticState>,
}
impl_transient!(
    InterfaceStatic,
    "Interface_Static",
    InterfaceTypedValue,
    base
);

impl Deref for InterfaceStatic {
    type Target = InterfaceTypedValue;
    fn deref(&self) -> &InterfaceTypedValue {
        &self.base
    }
}

impl InterfaceStatic {
    /// `Interface_Static(family, name, type = ParamText, init = "")`: up to date.
    pub fn new(
        family: &str,
        name: &str,
        kind: InterfaceParamType,
        init: &str,
    ) -> Result<Self, Failure> {
        Ok(Self::with_base(
            family,
            InterfaceTypedValue::new(name, kind, init)?,
        ))
    }
    /// `Interface_Static(family, name, other)`: the specification of `other` (limits,
    /// unit, enumeration, object type) and its value. Native quirk kept: the interpret
    /// and satisfies functions and the label are not copied.
    pub fn from_other(family: &str, name: &str, other: &InterfaceStatic) -> Result<Self, Failure> {
        let created = Self::with_base(
            family,
            InterfaceTypedValue::new(name, other.param_type(), "")?,
        );
        created.copy_specification(other)?;
        if other.is_set_value() {
            let text = String::from_utf8_lossy(other.c_string_value().as_bytes()).into_owned();
            created.set_c_string_value(&text)?;
        }
        Ok(created)
    }
    fn with_base(family: &str, base: InterfaceTypedValue) -> Self {
        Self {
            base,
            state: SharedState::new(StaticState {
                thefamily: AsciiString::new(family),
                thewild: Handle::default(),
                theupdate: true,
            }),
        }
    }
    /// The specification part of the copy constructor, by type.
    fn copy_specification(&self, other: &InterfaceStatic) -> Result<(), Failure> {
        match self.param_type() {
            InterfaceParamType::ParamInteger => {
                for max in [true, false] {
                    if let (true, lim) = other.integer_limit(max) {
                        self.set_integer_limit(max, lim)?;
                    }
                }
            }
            InterfaceParamType::ParamReal => {
                for max in [true, false] {
                    if let (true, lim) = other.real_limit(max) {
                        self.set_real_limit(max, lim)?;
                    }
                }
                self.set_unit_def(&String::from_utf8_lossy(other.unit_def().as_bytes()));
            }
            InterfaceParamType::ParamEnum => self.copy_enumeration(other)?,
            InterfaceParamType::ParamIdent => self.set_object_type(&other.object_type())?,
            _ => {}
        }
        Ok(())
    }
    fn copy_enumeration(&self, other: &InterfaceStatic) -> Result<(), Failure> {
        let (e0, e1, matched) = other.enum_def().unwrap_or((0, -1, false));
        self.start_enum(e0, matched)?;
        for case in e0..=e1 {
            let text = String::from_utf8_lossy(other.enum_val(case).as_bytes()).into_owned();
            self.add_enum(&[&text])?;
        }
        Ok(())
    }
    /// `PrintStatic(S)`: name, family, the typed value print and the wild-card. Native
    /// quirks kept: the value is not printed after "Value : ", and the satisfies line of
    /// the static's own (never set) function is not printed.
    pub fn print_static(&self, s: &mut dyn Write) -> io::Result<()> {
        s.write_all(b"--- Static Value : ")?;
        s.write_all(self.name().as_bytes())?;
        s.write_all(b"  Family:")?;
        s.write_all(self.family().as_bytes())?;
        self.print(s)?;
        let (wild, update) = {
            let state = self.state.read();
            (state.thewild.clone(), state.theupdate)
        };
        if let Some(card) = wild.get() {
            s.write_all(b" -- Attached to wild-card : ")?;
            s.write_all(card.name().as_bytes())?;
            s.write_all(b"\n")?;
        }
        write!(
            s,
            "--- Actual status : {}  Value : ",
            if update { "" } else { "original" }
        )
    }
    /// `Family`.
    pub fn family(&self) -> AsciiString {
        self.state.read().thefamily.clone()
    }
    /// `SetWild(wildcard)`.
    pub fn set_wild(&self, wildcard: &Handle<InterfaceStatic>) {
        self.state.write().thewild = wildcard.clone();
    }
    /// `Wild`.
    pub fn wild(&self) -> Handle<InterfaceStatic> {
        self.state.read().thewild.clone()
    }
    /// `SetUptodate`.
    pub fn set_uptodate(&self) {
        self.state.write().theupdate = true;
    }
    /// `UpdatedStatus`.
    pub fn updated_status(&self) -> bool {
        self.state.read().theupdate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moni_tool::MoniToolTypedValue;
    use rustycad_tkernel::standard::RuntimeTyped;

    #[test]
    fn copies_the_specification() {
        let source = InterfaceStatic::new("F", "src", InterfaceParamType::ParamEnum, "").unwrap();
        source.start_enum(1, true).unwrap();
        source.add_enum(&["One", "Two"]).unwrap();
        source.set_c_string_value("Two").unwrap();
        let copy = InterfaceStatic::from_other("G", "dst", &source).unwrap();
        assert_eq!(copy.enum_def(), Some((1, 2, true)));
        assert_eq!(copy.integer_value(), 2);
        let real = InterfaceStatic::new("F", "r", InterfaceParamType::ParamReal, "").unwrap();
        real.set_real_limit(true, 4.0).unwrap();
        real.set_unit_def("mm");
        let real_copy = InterfaceStatic::from_other("F", "r2", &real).unwrap();
        assert_eq!(real_copy.real_limit(true), (true, 4.0));
        assert_eq!(real_copy.unit_def().as_bytes(), b"mm");
        let int = InterfaceStatic::new("F", "i", InterfaceParamType::ParamInteger, "").unwrap();
        int.set_integer_limit(false, -1).unwrap();
        assert_eq!(
            InterfaceStatic::from_other("F", "i2", &int)
                .unwrap()
                .integer_limit(false),
            (true, -1)
        );
        let object = InterfaceStatic::new("F", "o", InterfaceParamType::ParamIdent, "").unwrap();
        assert!(
            InterfaceStatic::from_other("F", "o2", &object)
                .unwrap()
                .object_type()
                .is_some()
        );
    }

    #[test]
    fn wild_card_update_and_print() {
        let value = Handle::new(
            InterfaceStatic::new("F", "p", InterfaceParamType::ParamText, "v").unwrap(),
        );
        let wild =
            Handle::new(InterfaceStatic::new("F", "w", InterfaceParamType::ParamText, "").unwrap());
        value.set_wild(&wild);
        assert!(value.wild() == wild);
        value.set_uptodate();
        assert!(value.updated_status());
        let mut out = Vec::new();
        value.print_static(&mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(
            text.starts_with("--- Static Value : p  Family:F--- Typed Value : p\n"),
            "{text}"
        );
        assert!(
            text.ends_with(" -- Attached to wild-card : w\n--- Actual status :   Value : "),
            "{text}"
        );
        assert!(value.is_kind(MoniToolTypedValue::static_type()));
        assert!(value.cast::<InterfaceTypedValue>().is_some());
    }
}
