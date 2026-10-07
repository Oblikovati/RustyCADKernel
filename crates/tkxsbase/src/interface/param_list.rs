// Rust port of DataExchange/TKXSBase/Interface/Interface_ParamList (.hxx, .lxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_ParamList`: a growable list of file parameters, numbered from 1.
use super::InterfaceFileParameter;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, SharedState, StandardTransient, exceptions},
};

/// `Interface_ParamList`: a `NCollection_DynamicArray` of parameters natively; a write
/// beyond the end grows the list with default parameters.
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceFileParameter, InterfaceParamList};
/// let list = InterfaceParamList::new(256);
/// list.set_value(3, &InterfaceFileParameter::new());
/// assert_eq!((list.lower(), list.upper()), (1, 3));
/// ```
#[derive(Default)]
pub struct InterfaceParamList {
    transient: StandardTransient,
    my_vector: SharedState<Vec<InterfaceFileParameter>>,
}
impl_transient!(
    InterfaceParamList,
    "Interface_ParamList",
    StandardTransient,
    transient
);

impl InterfaceParamList {
    /// `Interface_ParamList(theIncrement = 256)`: empty (the increment is the growth step
    /// of the native blocks, not observable).
    pub fn new(_increment: i32) -> Self {
        Self::default()
    }
    /// `Length`.
    pub fn length(&self) -> i32 {
        self.my_vector.read().len() as i32
    }
    /// `Lower`: 1.
    pub fn lower(&self) -> i32 {
        1
    }
    /// `Upper`: the length.
    pub fn upper(&self) -> i32 {
        self.length()
    }
    /// `SetValue(index, value)`: grows the list up to `index` when needed;
    /// `Standard_OutOfRange` below 1.
    pub fn set_value(&self, index: i32, value: &InterfaceFileParameter) -> Result<(), Failure> {
        let slot = self.slot(index)?;
        let mut vector = self.my_vector.write();
        if slot >= vector.len() {
            vector.resize(slot + 1, InterfaceFileParameter::new());
        }
        vector[slot] = value.clone();
        Ok(())
    }
    /// `Value(index)` (and `operator()`): `Standard_OutOfRange` out of 1..Length.
    pub fn value(&self, index: i32) -> Result<InterfaceFileParameter, Failure> {
        let slot = self.slot(index)?;
        let vector = self.my_vector.read();
        vector
            .get(slot)
            .cloned()
            .ok_or_else(|| out_of_range(index, vector.len()))
    }
    /// `ChangeValue(index)`: edits the parameter, created (default) beyond the end.
    pub fn change_value<R>(
        &self,
        index: i32,
        edit: impl FnOnce(&mut InterfaceFileParameter) -> R,
    ) -> Result<R, Failure> {
        let slot = self.slot(index)?;
        let mut vector = self.my_vector.write();
        if slot >= vector.len() {
            vector.resize(slot + 1, InterfaceFileParameter::new());
        }
        Ok(edit(&mut vector[slot]))
    }
    /// `Clear`.
    pub fn clear(&self) {
        self.my_vector.write().clear();
    }
    fn slot(&self, index: i32) -> Result<usize, Failure> {
        usize::try_from(index - 1).map_err(|_| out_of_range(index, self.my_vector.read().len()))
    }
}

fn out_of_range(index: i32, length: usize) -> Failure {
    exceptions::out_of_range(&format!(
        "Interface_ParamList: parameter {index}; expected [1, {length}]"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::InterfaceParamType;

    #[test]
    fn grows_on_write() {
        let list = InterfaceParamList::new(4);
        assert!(list.value(1).is_err());
        list.change_value(2, |p| p.init_text("x", InterfaceParamType::ParamText))
            .unwrap();
        assert_eq!(list.length(), 2);
        assert_eq!(list.value(2).unwrap().c_value(), b"x");
        assert!(list.set_value(0, &InterfaceFileParameter::new()).is_err());
        list.clear();
        assert_eq!(list.upper(), 0);
    }
}
