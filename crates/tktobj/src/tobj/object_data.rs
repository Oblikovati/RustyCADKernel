// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Object.cxx (protected data accessors)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The protected `TObj_Object` tools that read and write values on the sub-labels of the
//! Data and Reference labels.
use super::{TObjObject, TObjObjectState, TObjTReference};
use rustycad_tkernel::{
    standard::{Failure, Guid, Handle},
    tcollection::{HAsciiString, HExtendedString},
};
use rustycad_tklcaf::{
    tdata_std::{
        TDataStdAsciiString, TDataStdGenericExtString, TDataStdInteger, TDataStdName, TDataStdReal,
    },
    tdf::{TdfLabel, TdfTagSource},
};

/// The label `theRank2` under `theRank1` under `root` (`theRank1` alone when `theRank2`
/// is 0), created when absent; a null label when `theRank1` is not positive.
fn ranked_label(root: TdfLabel, rank1: i32, rank2: i32) -> Result<TdfLabel, Failure> {
    if rank1 <= 0 {
        return Ok(TdfLabel::default());
    }
    let label = root.find_child(rank1, true)?;
    if rank2 > 0 {
        return label.find_child(rank2, true);
    }
    Ok(label)
}

/// `TObj_Object`'s protected tools on Data and Reference sub-labels, for every
/// [`TObjObject`]. The array tools are in `object_arrays.rs`.
pub trait TObjObjectData: TObjObject {
    /// `getDataLabel(theRank1, theRank2)`.
    fn get_data_label_at(&self, rank1: i32, rank2: i32) -> Result<TdfLabel, Failure> {
        if rank1 <= 0 {
            return Ok(TdfLabel::default());
        }
        ranked_label(self.get_data_label()?, rank1, rank2)
    }
    /// `getReferenceLabel(theRank1, theRank2)`.
    fn get_reference_label_at(&self, rank1: i32, rank2: i32) -> Result<TdfLabel, Failure> {
        if rank1 <= 0 {
            return Ok(TdfLabel::default());
        }
        ranked_label(self.get_reference_label()?, rank1, rank2)
    }
    /// `isDataAttribute(theGUID, theRank1, theRank2)`.
    fn is_data_attribute(&self, guid: &Guid, rank1: i32, rank2: i32) -> Result<bool, Failure> {
        self.get_data_label_at(rank1, rank2)?.is_attribute(guid)
    }
    /// `getReal(theRank1, theRank2)`: 0.0 when no real is stored.
    fn get_real(&self, rank1: i32, rank2: i32) -> Result<f64, Failure> {
        let label = self.get_data_label_at(rank1, rank2)?;
        let mut real: Handle<TDataStdReal> = Handle::default();
        label.find_attribute_as(&TDataStdReal::get_id(), &mut real)?;
        Ok(real.get().map_or(0.0, TDataStdReal::get))
    }
    /// `setReal(theValue, theRank1, theRank2, theTolerance)`: true when the value changed
    /// by more than the tolerance.
    fn set_real(
        &self,
        value: f64,
        rank1: i32,
        rank2: i32,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let label = self.get_data_label_at(rank1, rank2)?;
        let mut real: Handle<TDataStdReal> = Handle::default();
        if label.find_attribute_as(&TDataStdReal::get_id(), &mut real)?
            && (TDataStdReal::get(&real) - value).abs() <= tolerance
        {
            return Ok(false);
        }
        TDataStdReal::set(&label, value)?;
        Ok(true)
    }
    /// `getInteger(theRank1, theRank2)`: 0 when no integer is stored.
    fn get_integer(&self, rank1: i32, rank2: i32) -> Result<i32, Failure> {
        let label = self.get_data_label_at(rank1, rank2)?;
        let mut integer: Handle<TDataStdInteger> = Handle::default();
        label.find_attribute_as(&TDataStdInteger::get_id(), &mut integer)?;
        Ok(integer.get().map_or(0, TDataStdInteger::get))
    }
    /// `setInteger(theValue, theRank1, theRank2)`: true when the value changed.
    fn set_integer(&self, value: i32, rank1: i32, rank2: i32) -> Result<bool, Failure> {
        let label = self.get_data_label_at(rank1, rank2)?;
        let mut integer: Handle<TDataStdInteger> = Handle::default();
        if label.find_attribute_as(&TDataStdInteger::get_id(), &mut integer)?
            && TDataStdInteger::get(&integer) == value
        {
            return Ok(false);
        }
        TDataStdInteger::set(&label, value)?;
        Ok(true)
    }
    /// `setExtString(theValue, theRank1, theRank2)`: a null value forgets the name.
    fn set_ext_string(
        &self,
        value: &Handle<HExtendedString>,
        rank1: i32,
        rank2: i32,
    ) -> Result<(), Failure> {
        let label = self.get_data_label_at(rank1, rank2)?;
        match value.get() {
            Some(text) => TDataStdName::set(&label, &text.string()).map(|_| ()),
            None => label
                .forget_attribute_id(&TDataStdName::get_id())
                .map(|_| ()),
        }
    }
    /// `getExtString(theRank1, theRank2)`: null when no name is stored.
    fn get_ext_string(&self, rank1: i32, rank2: i32) -> Result<Handle<HExtendedString>, Failure> {
        let label = self.get_data_label_at(rank1, rank2)?;
        let mut name: Handle<TDataStdName> = Handle::default();
        label.find_attribute_as(&TDataStdName::get_id(), &mut name)?;
        Ok(match name.get() {
            Some(name) => Handle::new(HExtendedString::from(TDataStdGenericExtString::get(name))),
            None => Handle::default(),
        })
    }
    /// `setAsciiString(theValue, theRank1, theRank2)`: a null value forgets the string.
    fn set_ascii_string(
        &self,
        value: &Handle<HAsciiString>,
        rank1: i32,
        rank2: i32,
    ) -> Result<(), Failure> {
        let label = self.get_data_label_at(rank1, rank2)?;
        match value.get() {
            Some(text) => TDataStdAsciiString::set(&label, &text.string()).map(|_| ()),
            None => label
                .forget_attribute_id(&TDataStdAsciiString::get_id())
                .map(|_| ()),
        }
    }
    /// `getAsciiString(theRank1, theRank2)`: null when no string is stored.
    fn get_ascii_string(&self, rank1: i32, rank2: i32) -> Result<Handle<HAsciiString>, Failure> {
        let label = self.get_data_label_at(rank1, rank2)?;
        let mut text: Handle<TDataStdAsciiString> = Handle::default();
        label.find_attribute_as(&TDataStdAsciiString::get_id(), &mut text)?;
        Ok(match text.get() {
            Some(text) => Handle::new(HAsciiString::from(text.get())),
            None => Handle::default(),
        })
    }
    /// `getReference(theRank1, theRank2)`: the referred object, null when none.
    fn get_reference(&self, rank1: i32, rank2: i32) -> Result<Handle<dyn TObjObject>, Failure> {
        let label = self.get_reference_label_at(rank1, rank2)?;
        let mut reference: Handle<TObjTReference> = Handle::default();
        label.find_attribute_as(&TObjTReference::get_id(), &mut reference)?;
        Ok(match reference.get() {
            Some(reference) => reference.get_object()?,
            None => Handle::default(),
        })
    }
    /// `setReference(theObject, theRank1, theRank2)`: true when the reference changed; a
    /// null object forgets the reference.
    fn set_reference(
        &self,
        object: &Handle<dyn TObjObject>,
        rank1: i32,
        rank2: i32,
    ) -> Result<bool, Failure> {
        let label = self.get_reference_label_at(rank1, rank2)?;
        if object.is_null() {
            return label.forget_attribute_id(&TObjTReference::get_id());
        }
        let mut reference: Handle<TObjTReference> = Handle::default();
        if label.find_attribute_as(&TObjTReference::get_id(), &mut reference)?
            && reference.get_object()? == *object
        {
            return Ok(false);
        }
        // 27.07.05, PTv: the old reference is removed before the new one, for Undo/Redo.
        label.forget_attribute_id(&TObjTReference::get_id())?;
        TObjTReference::set(&label, object, &self.object_base().this()?)?;
        Ok(true)
    }
    /// `addReference(theRank1, theObject)`: a reference on the next free sub-label.
    fn add_reference(
        &self,
        rank1: i32,
        object: &Handle<dyn TObjObject>,
    ) -> Result<TdfLabel, Failure> {
        let mut root = self.get_reference_label()?;
        if rank1 > 0 {
            root = root.find_child(rank1, true)?;
        }
        let label = TdfTagSource::new_child_of(&root)?;
        TObjTReference::set(&label, object, &self.object_base().this()?)?;
        Ok(label)
    }
}
impl<T: TObjObject + ?Sized> TObjObjectData for T {}
