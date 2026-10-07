// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_ValueType.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MoniTool_ValueType`: the kinds of value a typed value or an attribute holds.

/// `MoniTool_ValueType`, with the native discriminants;
/// `MoniToolValueType::ValueInteger as i32 == 1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum MoniToolValueType {
    /// `MoniTool_ValueMisc`.
    ValueMisc = 0,
    /// `MoniTool_ValueInteger`.
    ValueInteger = 1,
    /// `MoniTool_ValueReal`.
    ValueReal = 2,
    /// `MoniTool_ValueIdent`: any object.
    ValueIdent = 3,
    /// `MoniTool_ValueVoid`: not recorded.
    ValueVoid = 4,
    /// `MoniTool_ValueText`: free text, the default of a typed value.
    #[default]
    ValueText = 5,
    /// `MoniTool_ValueEnum`.
    ValueEnum = 6,
    /// `MoniTool_ValueLogical`.
    ValueLogical = 7,
    /// `MoniTool_ValueSub`.
    ValueSub = 8,
    /// `MoniTool_ValueHexa`.
    ValueHexa = 9,
    /// `MoniTool_ValueBinary`.
    ValueBinary = 10,
}

#[cfg(test)]
mod tests {
    use super::MoniToolValueType;

    #[test]
    fn discriminants_follow_the_native_enum() {
        assert_eq!(MoniToolValueType::ValueMisc as i32, 0);
        assert_eq!(MoniToolValueType::ValueText as i32, 5);
        assert_eq!(MoniToolValueType::ValueBinary as i32, 10);
        assert_eq!(MoniToolValueType::default(), MoniToolValueType::ValueText);
    }
}
