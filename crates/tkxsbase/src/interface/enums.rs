// Rust port of DataExchange/TKXSBase/Interface/Interface_CheckStatus.hxx,
// Interface_DataState.hxx and Interface_ParamType.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The enumerations of the Interface package.

/// `Interface_CheckStatus`: the state of a check, or a criterion on checks;
/// `InterfaceCheckStatus::CheckFail as i32 == 2`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum InterfaceCheckStatus {
    /// `Interface_CheckOK`: no fail, no warning.
    #[default]
    CheckOK = 0,
    /// `Interface_CheckWarning`: warnings, no fail.
    CheckWarning = 1,
    /// `Interface_CheckFail`: fails.
    CheckFail = 2,
    /// `Interface_CheckAny`: any status.
    CheckAny = 3,
    /// `Interface_CheckMessage`: warnings or fails.
    CheckMessage = 4,
    /// `Interface_CheckNoFail`: OK or warnings.
    CheckNoFail = 5,
}

/// `Interface_DataState`: the state of an entity in a model;
/// `InterfaceDataState::StateUnknown as i32 == 6`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum InterfaceDataState {
    /// `Interface_StateOK`.
    #[default]
    StateOK = 0,
    /// `Interface_LoadWarning`.
    LoadWarning = 1,
    /// `Interface_LoadFail`.
    LoadFail = 2,
    /// `Interface_DataWarning`.
    DataWarning = 3,
    /// `Interface_DataFail`.
    DataFail = 4,
    /// `Interface_StateUnloaded`.
    StateUnloaded = 5,
    /// `Interface_StateUnknown`.
    StateUnknown = 6,
}

/// `Interface_ParamType`: the type of a parameter, with the values of
/// [`MoniToolValueType`](crate::moni_tool::MoniToolValueType);
/// `InterfaceParamType::ParamText as i32 == 5`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum InterfaceParamType {
    /// `Interface_ParamMisc`.
    #[default]
    ParamMisc = 0,
    /// `Interface_ParamInteger`.
    ParamInteger = 1,
    /// `Interface_ParamReal`.
    ParamReal = 2,
    /// `Interface_ParamIdent`.
    ParamIdent = 3,
    /// `Interface_ParamVoid`.
    ParamVoid = 4,
    /// `Interface_ParamText`.
    ParamText = 5,
    /// `Interface_ParamEnum`.
    ParamEnum = 6,
    /// `Interface_ParamLogical`.
    ParamLogical = 7,
    /// `Interface_ParamSub`.
    ParamSub = 8,
    /// `Interface_ParamHexa`.
    ParamHexa = 9,
    /// `Interface_ParamBinary`.
    ParamBinary = 10,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminants_follow_the_native_enums() {
        assert_eq!(InterfaceCheckStatus::CheckNoFail as i32, 5);
        assert_eq!(InterfaceDataState::DataFail as i32, 4);
        assert_eq!(InterfaceParamType::ParamBinary as i32, 10);
    }
}
