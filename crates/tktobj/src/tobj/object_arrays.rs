// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Object.cxx (getRealArray,
// getIntegerArray, getExtStringArray and the three setArray) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The protected `TObj_Object` array tools over the `TDataStd` array attributes.
use super::{TObjObject, TObjObjectData};
use rustycad_tkernel::{
    ncollection::HArray1,
    standard::{Failure, Guid, Handle, RuntimeTyped},
    tcollection::ExtendedString,
};
use rustycad_tklcaf::{
    tdata_std::{TDataStdExtStringArray, TDataStdIntegerArray, TDataStdRealArray},
    tdf::{TdfAttribute, TdfLabel},
};

/// The members of a `TDataStd` array attribute that the TObj array tools use.
pub trait TObjArrayAttribute: TdfAttribute + RuntimeTyped + Sized {
    /// The item type of the array.
    type Item: Clone + Send + Sync + 'static;
    /// `GetID`.
    fn array_id() -> Guid;
    /// `Set(label, 1, theLength)`.
    fn set_on(label: &TdfLabel, upper: i32) -> Result<Handle<Self>, Failure>;
    /// `Array()`: the shared array.
    fn shared_array(&self) -> Handle<HArray1<Self::Item>>;
    /// `Init(lower, upper)`.
    fn init_bounds(&self, lower: i32, upper: i32) -> Result<(), Failure>;
    /// `ChangeArray(theArray)` with the default item check.
    fn take_array(&self, array: &Handle<HArray1<Self::Item>>) -> Result<(), Failure>;
}

macro_rules! tobj_array_attribute {
    ($attribute:ty, $item:ty, $array:ident) => {
        impl TObjArrayAttribute for $attribute {
            type Item = $item;
            fn array_id() -> Guid {
                <$attribute>::get_id()
            }
            fn set_on(label: &TdfLabel, upper: i32) -> Result<Handle<Self>, Failure> {
                <$attribute>::set(label, 1, upper, false)
            }
            fn shared_array(&self) -> Handle<HArray1<$item>> {
                self.$array()
            }
            fn init_bounds(&self, lower: i32, upper: i32) -> Result<(), Failure> {
                self.init(lower, upper)
            }
            fn take_array(&self, array: &Handle<HArray1<$item>>) -> Result<(), Failure> {
                self.change_array(array, true)
            }
        }
    };
}
tobj_array_attribute!(TDataStdRealArray, f64, array);
tobj_array_attribute!(TDataStdIntegerArray, i32, array);
tobj_array_attribute!(TDataStdExtStringArray, ExtendedString, array);

/// The array on the label, created with bounds `1..=length` (then filled by `fill`) when
/// absent and `length` is positive; null otherwise.
fn existing_or_new_array<A: TObjArrayAttribute>(
    label: &TdfLabel,
    length: i32,
    fill: impl FnOnce(&HArray1<A::Item>),
) -> Result<Handle<HArray1<A::Item>>, Failure> {
    let mut attribute: Handle<A> = Handle::default();
    if !label.find_attribute_as(&A::array_id(), &mut attribute)? && length > 0 {
        attribute = A::set_on(label, length)?;
        if let Some(array) = attribute.shared_array().get() {
            fill(array);
        }
    }
    Ok(attribute.get().map(A::shared_array).unwrap_or_default())
}

/// The body of the three `setArray`: a null array forgets the attribute; storing the
/// attribute's own array first re-initializes it so that the change is backed up.
fn store_array<A: TObjArrayAttribute>(
    label: &TdfLabel,
    array: &Handle<HArray1<A::Item>>,
) -> Result<(), Failure> {
    let mut attribute: Handle<A> = Handle::default();
    if !label.find_attribute_as(&A::array_id(), &mut attribute)? && !array.is_null() {
        attribute = A::set_on(label, 1)?;
    }
    if array.is_null() {
        if !attribute.is_null() {
            label.forget_attribute(&attribute.cast())?;
        }
        return Ok(());
    }
    let target = rustycad_tkcdf::live(&attribute, "TObj_Object::setArray")?;
    if target.shared_array() == *array {
        // The backup would not happen, but it is wanted.
        target.init_bounds(1, 1)?;
    }
    target.take_array(array)
}

/// `TObj_Object`'s protected array tools, for every [`TObjObject`].
pub trait TObjObjectArrays: TObjObject {
    /// `getRealArray(theLength, theRank1, theRank2, theInitialValue)`.
    fn get_real_array(
        &self,
        length: i32,
        rank1: i32,
        rank2: i32,
        initial_value: f64,
    ) -> Result<Handle<HArray1<f64>>, Failure> {
        let label = self.get_data_label_at(rank1, rank2)?;
        existing_or_new_array::<TDataStdRealArray>(&label, length, |array| {
            array.change_array1().init(initial_value)
        })
    }
    /// `getIntegerArray(theLength, theRank1, theRank2, theInitialValue)`.
    fn get_integer_array(
        &self,
        length: i32,
        rank1: i32,
        rank2: i32,
        initial_value: i32,
    ) -> Result<Handle<HArray1<i32>>, Failure> {
        let label = self.get_data_label_at(rank1, rank2)?;
        existing_or_new_array::<TDataStdIntegerArray>(&label, length, |array| {
            array.change_array1().init(initial_value)
        })
    }
    /// `getExtStringArray(theLength, theRank1, theRank2)`: a new array is not initialized.
    fn get_ext_string_array(
        &self,
        length: i32,
        rank1: i32,
        rank2: i32,
    ) -> Result<Handle<HArray1<ExtendedString>>, Failure> {
        let label = self.get_data_label_at(rank1, rank2)?;
        existing_or_new_array::<TDataStdExtStringArray>(&label, length, |_| {})
    }
    /// `setArray(const occ::handle<NCollection_HArray1<double>>&, theRank1, theRank2)`.
    fn set_real_array(
        &self,
        array: &Handle<HArray1<f64>>,
        rank1: i32,
        rank2: i32,
    ) -> Result<(), Failure> {
        store_array::<TDataStdRealArray>(&self.get_data_label_at(rank1, rank2)?, array)
    }
    /// `setArray(const occ::handle<NCollection_HArray1<int>>&, theRank1, theRank2)`.
    fn set_integer_array(
        &self,
        array: &Handle<HArray1<i32>>,
        rank1: i32,
        rank2: i32,
    ) -> Result<(), Failure> {
        store_array::<TDataStdIntegerArray>(&self.get_data_label_at(rank1, rank2)?, array)
    }
    /// `setArray(const occ::handle<NCollection_HArray1<TCollection_ExtendedString>>&, ...)`.
    fn set_ext_string_array(
        &self,
        array: &Handle<HArray1<ExtendedString>>,
        rank1: i32,
        rank2: i32,
    ) -> Result<(), Failure> {
        store_array::<TDataStdExtStringArray>(&self.get_data_label_at(rank1, rank2)?, array)
    }
}
impl<T: TObjObject + ?Sized> TObjObjectArrays for T {}
