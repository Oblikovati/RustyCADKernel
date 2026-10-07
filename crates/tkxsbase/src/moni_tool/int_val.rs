// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_IntVal (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MoniTool_IntVal`: an integer through a handle.
use rustycad_tkernel::{
    impl_transient,
    standard::{SharedState, StandardTransient},
};
use std::sync::RwLockWriteGuard;

/// `MoniTool_IntVal`: an integer shared through handles; the value is edited through
/// any of them (`CValue`).
///
/// ```
/// use rustycad_tkxsbase::moni_tool::MoniToolIntVal;
/// let val = MoniToolIntVal::new(3);
/// *val.c_value() = 7;
/// assert_eq!(val.value(), 7);
/// ```
#[derive(Default)]
pub struct MoniToolIntVal {
    transient: StandardTransient,
    theval: SharedState<i32>,
}
impl_transient!(
    MoniToolIntVal,
    "MoniTool_IntVal",
    StandardTransient,
    transient
);

impl MoniToolIntVal {
    /// `MoniTool_IntVal(val = 0)`.
    pub fn new(val: i32) -> Self {
        Self {
            transient: StandardTransient::default(),
            theval: SharedState::new(val),
        }
    }
    /// `Value`.
    pub fn value(&self) -> i32 {
        *self.theval.read()
    }
    /// `CValue`: the modifiable value (natively `int&`); drop the guard before reading it
    /// again.
    pub fn c_value(&self) -> RwLockWriteGuard<'_, i32> {
        self.theval.write()
    }
}

#[cfg(test)]
mod tests {
    use super::MoniToolIntVal;
    use rustycad_tkernel::standard::{Handle, StandardTransient};

    #[test]
    fn value_is_shared_through_handles() {
        let first = Handle::new(MoniToolIntVal::default());
        let second: Handle<StandardTransient> = first.cast();
        assert_eq!(first.value(), 0);
        *second.cast::<MoniToolIntVal>().c_value() = 12;
        assert_eq!(first.value(), 12);
        assert_eq!(second.dynamic_type().unwrap().name(), "MoniTool_IntVal");
    }
}
