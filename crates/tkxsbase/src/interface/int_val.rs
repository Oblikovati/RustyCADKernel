// Rust port of DataExchange/TKXSBase/Interface/Interface_IntVal (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_IntVal`: an integer through a handle (the shared cursor of the entity
//! iterators).
use rustycad_tkernel::{
    impl_transient,
    standard::{SharedState, StandardTransient},
};
use std::sync::RwLockWriteGuard;

/// `Interface_IntVal`: an integer shared through handles, 0 at creation.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceIntVal;
/// let val = InterfaceIntVal::new();
/// *val.c_value() += 2;
/// assert_eq!(val.value(), 2);
/// ```
#[derive(Default)]
pub struct InterfaceIntVal {
    transient: StandardTransient,
    theval: SharedState<i32>,
}
impl_transient!(
    InterfaceIntVal,
    "Interface_IntVal",
    StandardTransient,
    transient
);

impl InterfaceIntVal {
    /// `Interface_IntVal()`: 0.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Value`.
    pub fn value(&self) -> i32 {
        *self.theval.read()
    }
    /// `CValue`: the modifiable value (natively `int&`); drop the guard before reading
    /// it again.
    pub fn c_value(&self) -> RwLockWriteGuard<'_, i32> {
        self.theval.write()
    }
}

#[cfg(test)]
mod tests {
    use super::InterfaceIntVal;
    use rustycad_tkernel::standard::Handle;

    #[test]
    fn shared_value() {
        let first = Handle::new(InterfaceIntVal::new());
        let second = first.clone();
        *second.c_value() = 5;
        assert_eq!(first.value(), 5);
        assert_eq!(first.dynamic_type().unwrap().name(), "Interface_IntVal");
    }
}
