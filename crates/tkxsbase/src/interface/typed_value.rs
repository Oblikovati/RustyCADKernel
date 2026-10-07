// Rust port of DataExchange/TKXSBase/Interface/Interface_TypedValue (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_TypedValue`: a `MoniTool_TypedValue` whose type is an
//! `Interface_ParamType` (the two enumerations have the same values).
use super::InterfaceParamType;
use crate::moni_tool::{MoniToolTypedValue, MoniToolValueType};
use rustycad_tkernel::{impl_transient, standard::Failure};
use std::ops::Deref;

/// `Interface_TypedValue`: embeds its `MoniTool_TypedValue` (registered as its RTTI
/// parent) and dereferences to it. The native class redeclares private fields of its base
/// that it never uses; they have no counterpart.
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceParamType, InterfaceTypedValue};
/// let value = InterfaceTypedValue::new("count", InterfaceParamType::ParamInteger, "3").unwrap();
/// assert_eq!(value.param_type(), InterfaceParamType::ParamInteger);
/// assert_eq!(value.integer_value(), 3);
/// ```
pub struct InterfaceTypedValue {
    base: MoniToolTypedValue,
}
impl_transient!(
    InterfaceTypedValue,
    "Interface_TypedValue",
    MoniToolTypedValue,
    base
);

impl Deref for InterfaceTypedValue {
    type Target = MoniToolTypedValue;
    fn deref(&self) -> &MoniToolTypedValue {
        &self.base
    }
}

impl InterfaceTypedValue {
    /// `Interface_TypedValue(name, type = ParamText, init = "")`.
    pub fn new(name: &str, kind: InterfaceParamType, init: &str) -> Result<Self, Failure> {
        let base = MoniToolTypedValue::new(name, Self::param_type_to_value_type(kind), init)?;
        Ok(Self { base })
    }
    /// `Type`: the value type as a parameter type.
    pub fn param_type(&self) -> InterfaceParamType {
        Self::value_type_to_param_type(self.value_type())
    }
    /// `ParamTypeToValueType(typ)`: the same value.
    pub fn param_type_to_value_type(typ: InterfaceParamType) -> MoniToolValueType {
        use InterfaceParamType as P;
        use MoniToolValueType as V;
        match typ {
            P::ParamMisc => V::ValueMisc,
            P::ParamInteger => V::ValueInteger,
            P::ParamReal => V::ValueReal,
            P::ParamIdent => V::ValueIdent,
            P::ParamVoid => V::ValueVoid,
            P::ParamText => V::ValueText,
            P::ParamEnum => V::ValueEnum,
            P::ParamLogical => V::ValueLogical,
            P::ParamSub => V::ValueSub,
            P::ParamHexa => V::ValueHexa,
            P::ParamBinary => V::ValueBinary,
        }
    }
    /// `ValueTypeToParamType(typ)`: the same value.
    pub fn value_type_to_param_type(typ: MoniToolValueType) -> InterfaceParamType {
        use InterfaceParamType as P;
        use MoniToolValueType as V;
        match typ {
            V::ValueMisc => P::ParamMisc,
            V::ValueInteger => P::ParamInteger,
            V::ValueReal => P::ParamReal,
            V::ValueIdent => P::ParamIdent,
            V::ValueVoid => P::ParamVoid,
            V::ValueText => P::ParamText,
            V::ValueEnum => P::ParamEnum,
            V::ValueLogical => P::ParamLogical,
            V::ValueSub => P::ParamSub,
            V::ValueHexa => P::ParamHexa,
            V::ValueBinary => P::ParamBinary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::standard::{Handle, RuntimeTyped};

    #[test]
    fn conversions_keep_the_value() {
        for value in 0..=10 {
            let typ: InterfaceParamType = param_of(value);
            let back = InterfaceTypedValue::param_type_to_value_type(typ);
            assert_eq!(back as i32, value);
            assert_eq!(InterfaceTypedValue::value_type_to_param_type(back), typ);
        }
        let bad = InterfaceTypedValue::new("x", InterfaceParamType::ParamHexa, "")
            .err()
            .unwrap();
        assert_eq!(bad.exception_type(), "Standard_ConstructionError");
        let value =
            Handle::new(InterfaceTypedValue::new("t", InterfaceParamType::ParamText, "").unwrap());
        assert!(value.cast::<MoniToolTypedValue>().is_some());
        assert!(value.is_kind(MoniToolTypedValue::static_type()));
    }

    fn param_of(value: i32) -> InterfaceParamType {
        use InterfaceParamType as P;
        [
            P::ParamMisc,
            P::ParamInteger,
            P::ParamReal,
            P::ParamIdent,
            P::ParamVoid,
            P::ParamText,
            P::ParamEnum,
            P::ParamLogical,
            P::ParamSub,
            P::ParamHexa,
            P::ParamBinary,
        ][value as usize]
    }
}
