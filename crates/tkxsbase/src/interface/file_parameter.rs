// Rust port of DataExchange/TKXSBase/Interface/Interface_FileParameter (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_FileParameter`: one parameter read from a file: its text, its type and the
//! number of the entity it designates.
use super::InterfaceParamType;
use rustycad_tkernel::tcollection::AsciiString;

/// `Interface_FileParameter`. Natively the text is a pointer into the buffer of the
/// `Interface_ParamSet` that holds the parameter; here it is the parameter's own copy
/// (`None` before `Init` and after `Clear`, where the native pointer is null or unset).
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceFileParameter, InterfaceParamType};
/// let mut param = InterfaceFileParameter::new();
/// param.init_text("#12", InterfaceParamType::ParamIdent);
/// param.set_entity_number(12);
/// assert_eq!(param.c_value(), b"#12");
/// assert_eq!(param.entity_number(), 12);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InterfaceFileParameter {
    thetype: InterfaceParamType,
    theval: Option<AsciiString>,
    thenum: i32,
}

impl InterfaceFileParameter {
    /// `Interface_FileParameter()`: misc type, entity 0, no text.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Init(const TCollection_AsciiString&, typ)`: a copy of the text; entity 0.
    pub fn init(&mut self, val: &AsciiString, typ: InterfaceParamType) {
        self.theval = Some(val.clone());
        self.thetype = typ;
        self.thenum = 0;
    }
    /// `Init(const char*, typ)`.
    pub fn init_text(&mut self, val: impl AsRef<[u8]>, typ: InterfaceParamType) {
        self.init(&AsciiString::from_bytes(val), typ);
    }
    /// `CValue`: the text (empty when unset).
    pub fn c_value(&self) -> &[u8] {
        self.theval.as_ref().map_or(b"", AsciiString::as_bytes)
    }
    /// Whether the parameter has a text (natively a non-null `CValue`).
    pub fn has_value(&self) -> bool {
        self.theval.is_some()
    }
    /// `ParamType`.
    pub fn param_type(&self) -> InterfaceParamType {
        self.thetype
    }
    /// `SetEntityNumber(num)`.
    pub fn set_entity_number(&mut self, num: i32) {
        self.thenum = num;
    }
    /// `EntityNumber`.
    pub fn entity_number(&self) -> i32 {
        self.thenum
    }
    /// `Clear`: no text (the type and number are kept, as natively).
    pub fn clear(&mut self) {
        self.theval = None;
    }
    /// `Destroy`: nothing (the native destructor's work).
    pub fn destroy(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_resets_the_entity_and_clear_keeps_the_type() {
        let mut param = InterfaceFileParameter::new();
        assert!(!param.has_value() && param.param_type() == InterfaceParamType::ParamMisc);
        param.set_entity_number(4);
        param.init(&AsciiString::new("1.5"), InterfaceParamType::ParamReal);
        assert_eq!(param.entity_number(), 0);
        param.clear();
        param.destroy();
        assert_eq!(param.c_value(), b"");
        assert_eq!(param.param_type(), InterfaceParamType::ParamReal);
    }
}
