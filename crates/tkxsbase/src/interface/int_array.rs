//! The shared integer arrays (`occ::handle<NCollection_HArray1<int>>`) of the Interface
//! classes, with the native failures of their checked accessors.
use rustycad_tkernel::{
    ncollection::HArray1,
    standard::{Failure, Handle, exceptions},
};

/// `occ::handle<NCollection_HArray1<int>>`: shared through every holder, as natively.
pub type InterfaceIntArray = Handle<HArray1<i32>>;

/// `new NCollection_HArray1<int>(lower, upper)` followed by `Init(0)`;
/// `Standard_RangeError` for `upper < lower - 1`.
pub(crate) fn new_int_array(lower: i32, upper: i32) -> Result<InterfaceIntArray, Failure> {
    let array = HArray1::with_bounds(lower, upper)
        .map_err(|e| e.into_failure("NCollection_HArray1<int>"))?;
    Ok(Handle::new(array))
}

/// `Value(index)`: `Standard_OutOfRange` out of the bounds, `Standard_NullObject` for a
/// null handle.
pub(crate) fn int_at(array: &InterfaceIntArray, index: i32) -> Result<i32, Failure> {
    let live = live_array(array)?;
    let values = live.array1();
    values
        .value(index)
        .copied()
        .map_err(|e| e.into_failure("NCollection_HArray1<int>::Value"))
}

/// `SetValue(index, value)`: the failures of [`int_at`].
pub(crate) fn set_int(array: &InterfaceIntArray, index: i32, value: i32) -> Result<(), Failure> {
    let live = live_array(array)?;
    let mut values = live.change_array1();
    values
        .set_value(index, value)
        .map_err(|e| e.into_failure("NCollection_HArray1<int>::SetValue"))
}

/// `Upper()`: `Standard_NullObject` for a null handle.
pub(crate) fn int_upper(array: &InterfaceIntArray) -> Result<i32, Failure> {
    Ok(live_array(array)?.array1().upper())
}

/// `Lower()`: `Standard_NullObject` for a null handle.
pub(crate) fn int_lower(array: &InterfaceIntArray) -> Result<i32, Failure> {
    Ok(live_array(array)?.array1().lower())
}

/// A new array `lower..=upper` (zeroed) holding the values of `source` at `from..=to`.
pub(crate) fn regrown_int_array(
    source: &InterfaceIntArray,
    lower: i32,
    upper: i32,
    copied: std::ops::RangeInclusive<i32>,
) -> Result<InterfaceIntArray, Failure> {
    let grown = new_int_array(lower, upper)?;
    for index in copied {
        set_int(&grown, index, int_at(source, index)?)?;
    }
    Ok(grown)
}

fn live_array(array: &InterfaceIntArray) -> Result<&HArray1<i32>, Failure> {
    array.get().ok_or_else(|| {
        exceptions::null_object("NCollection_HArray1<int>: null handle; expected a live array")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_access_and_growth() {
        let array = new_int_array(0, 3).unwrap();
        assert_eq!(int_at(&array, 3).unwrap(), 0);
        set_int(&array, 2, 7).unwrap();
        let grown = regrown_int_array(&array, 0, 6, 1..=3).unwrap();
        assert_eq!(
            (int_at(&grown, 2).unwrap(), int_upper(&grown).unwrap()),
            (7, 6)
        );
        assert_eq!(int_lower(&grown).unwrap(), 0);
        let error = int_at(&array, 4).err().unwrap();
        assert_eq!(error.exception_type(), "Standard_OutOfRange");
        let null = InterfaceIntArray::default();
        assert_eq!(
            int_at(&null, 0).err().unwrap().exception_type(),
            "Standard_NullObject"
        );
    }
}
