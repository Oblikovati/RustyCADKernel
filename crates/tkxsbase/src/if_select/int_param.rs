// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_IntParam (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_IntParam`: an integer parameter of the selections, optionally bound to a
//! static of `Interface_Static`.
use crate::interface::InterfaceStatic;
use rustycad_tkernel::{
    impl_transient,
    standard::{SharedState, StandardTransient},
    tcollection::AsciiString,
};

/// The fields of `IFSelect_IntParam`.
#[derive(Default)]
struct IntParamState {
    theval: i32,
    thestn: AsciiString,
}

/// `IFSelect_IntParam`.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectIntParam;
/// let param = IFSelectIntParam::new();
/// param.set_value(3);
/// assert_eq!(param.value(), 3);
/// ```
#[derive(Default)]
pub struct IFSelectIntParam {
    transient: StandardTransient,
    state: SharedState<IntParamState>,
}
impl_transient!(
    IFSelectIntParam,
    "IFSelect_IntParam",
    StandardTransient,
    transient
);

impl IFSelectIntParam {
    /// `IFSelect_IntParam()`: value 0, no static.
    pub fn new() -> Self {
        Self::default()
    }
    /// `SetStaticName(statname)`: binds the value to a static.
    pub fn set_static_name(&self, statname: &str) {
        self.state.write().thestn = AsciiString::new(statname);
    }
    /// `StaticName`.
    pub fn static_name(&self) -> AsciiString {
        self.state.read().thestn.clone()
    }
    /// The static name as text (empty without one).
    fn static_text(&self) -> String {
        String::from_utf8_lossy(self.state.read().thestn.as_bytes()).into_owned()
    }
    /// `Value`: the value of the static when set, else the own value.
    pub fn value(&self) -> i32 {
        let name = self.static_text();
        if name.is_empty() || !InterfaceStatic::is_set(&name, true) {
            return self.state.read().theval;
        }
        InterfaceStatic::i_val(&name)
    }
    /// `SetValue(val)`: the own value, and the static when it exists.
    pub fn set_value(&self, val: i32) {
        self.state.write().theval = val;
        let name = self.static_text();
        if name.is_empty() || !InterfaceStatic::is_present(&name) {
            return;
        }
        // a failure of the static update is ignored, as natively (SetIVal returns false)
        let _ = InterfaceStatic::set_i_val(&name, val);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_and_statics() {
        let param = IFSelectIntParam::new();
        assert_eq!(param.value(), 0);
        param.set_static_name("ifselect.test.no.static");
        param.set_value(4);
        assert_eq!(
            (param.value(), param.static_name().as_bytes()),
            (4, &b"ifselect.test.no.static"[..])
        );
        InterfaceStatic::init_char("XSTEP", "ifselect.test.intparam", b'i', "1").unwrap();
        param.set_static_name("ifselect.test.intparam");
        param.set_value(7);
        assert_eq!(
            (
                param.value(),
                InterfaceStatic::i_val("ifselect.test.intparam")
            ),
            (7, 7)
        );
    }
}
