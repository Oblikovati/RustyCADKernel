// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Object.hxx and .cxx (non-virtual public
// members and statics) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The non-virtual public members of `TObj_Object`: labels, names, flags, the father
//! object and the static `GetObj`/`Detach`.
use super::TObjObjectBase;
use super::{TObjDeletingMode, TObjObject, TObjObjectData, TObjObjectDataTag, TObjTObject};
use rustycad_tkernel::{
    standard::{Failure, Handle, RuntimeType},
    tcollection::{AsciiString, ExtendedString, HAsciiString, HExtendedString},
};
use rustycad_tklcaf::tdf::TdfLabel;

/// `TObj_Object`'s non-virtual public members, for every [`TObjObject`].
pub trait TObjObjectState: TObjObject {
    /// `GetLabel`: the label holding the object's data (null once the object is dead).
    fn get_label(&self) -> TdfLabel {
        self.object_base().label()
    }
    /// `GetChildLabel`: sub-label 4, created when absent.
    fn get_child_label(&self) -> Result<TdfLabel, Failure> {
        self.get_label().find_child(4, true)
    }
    /// `getChildLabel(theRank)`: sub-label `theRank` of the child label when positive.
    fn get_child_label_at(&self, rank: i32) -> Result<TdfLabel, Failure> {
        let label = self.get_child_label()?;
        if rank > 0 {
            return label.find_child(rank, true);
        }
        Ok(label)
    }
    /// `GetDataLabel`: sub-label 3, created when absent.
    fn get_data_label(&self) -> Result<TdfLabel, Failure> {
        self.get_label().find_child(3, true)
    }
    /// `GetReferenceLabel`: sub-label 1, created when absent.
    fn get_reference_label(&self) -> Result<TdfLabel, Failure> {
        self.get_label().find_child(1, true)
    }
    /// `GetName(TCollection_ExtendedString&)`: true when the name is not empty.
    fn get_name_extended(&self, name: &mut ExtendedString) -> Result<bool, Failure> {
        let found = self.get_name()?;
        *name = found.string().clone();
        Ok(name.length() != 0)
    }
    /// `GetName(TCollection_AsciiString&)`: true when the name is not empty.
    fn get_name_ascii(&self, name: &mut AsciiString) -> Result<bool, Failure> {
        let found = self.get_name()?;
        let Some(found) = found.get() else {
            return Ok(false);
        };
        *name = AsciiString::from(&*found.string());
        Ok(name.length() != 0)
    }
    /// `SetName(const occ::handle<TCollection_HAsciiString>&)`.
    fn set_name_h_ascii(&self, name: &Handle<HAsciiString>) -> Result<bool, Failure> {
        let ascii = rustycad_tkcdf::live(name, "TObj_Object::SetName")?;
        self.set_name(&Handle::new(HExtendedString::from_h_ascii_string(ascii)))
    }
    /// `SetName(const char*)`.
    fn set_name_c_str(&self, name: &str) -> Result<bool, Failure> {
        self.set_name_h_ascii(&Handle::new(HAsciiString::new(name)))
    }
    /// `HasBackReferences`: at least one back reference.
    fn has_back_references(&self) -> bool {
        let references = self.get_back_references(None);
        references.get().is_some_and(|iterator| iterator.more())
    }
    /// `GetFatherObject(theType)`: the nearest ancestor object of that type (any type
    /// when `None`); null when none.
    fn get_father_object(
        &self,
        kind: Option<&'static RuntimeType>,
    ) -> Result<Handle<dyn TObjObject>, Failure> {
        let mut father = Handle::default();
        if self.get_label().is_null() {
            return Ok(father);
        }
        let mut son_label = self.get_label();
        while TObjObjectBase::get_obj(&son_label.father(), &mut father, true)? {
            if kind.is_none_or(|kind| father.is_kind(kind)) {
                break;
            }
            son_label = father.get_label();
            father = Handle::default();
        }
        Ok(father)
    }
    /// `GetFlags`.
    fn get_flags(&self) -> Result<i32, Failure> {
        self.get_integer(TObjObjectDataTag::FLAGS, 0)
    }
    /// `SetFlags(theMask)`: adds the bits of the mask.
    fn set_flags(&self, mask: i32) -> Result<(), Failure> {
        let flags = self.get_integer(TObjObjectDataTag::FLAGS, 0)? | mask;
        self.set_integer(flags, TObjObjectDataTag::FLAGS, 0)?;
        Ok(())
    }
    /// `TestFlags(theMask)`: any bit of the mask is set.
    fn test_flags(&self, mask: i32) -> Result<bool, Failure> {
        Ok(self.get_integer(TObjObjectDataTag::FLAGS, 0)? & mask != 0)
    }
    /// `ClearFlags(theMask)`: removes the bits of the mask (`~0` clears all).
    fn clear_flags(&self, mask: i32) -> Result<(), Failure> {
        let flags = self.get_integer(TObjObjectDataTag::FLAGS, 0)? & !mask;
        self.set_integer(flags, TObjObjectDataTag::FLAGS, 0)?;
        Ok(())
    }
    /// `HasModifications`: the label may be modified in the open transaction.
    fn has_modifications(&self) -> Result<bool, Failure> {
        Ok(self.is_alive()? && self.get_label().may_be_modified())
    }
}
impl<T: TObjObject + ?Sized> TObjObjectState for T {}

impl TObjObjectBase {
    /// `GetObj(theLabel, theResult, isSuper)` (static): the live object of the label, or
    /// (with `is_super`) of its nearest ancestor; false when none.
    pub fn get_obj(
        label: &TdfLabel,
        result: &mut Handle<dyn TObjObject>,
        is_super: bool,
    ) -> Result<bool, Failure> {
        if label.is_null() {
            return Ok(false);
        }
        let mut attribute: Handle<TObjTObject> = Handle::default();
        *result = if label.find_attribute_as(&TObjTObject::get_id(), &mut attribute)? {
            attribute.get_object()
        } else {
            Handle::default()
        };
        if let Some(found) = result.get() {
            if !found.get_label().is_null() {
                return Ok(true);
            }
            // An object that is not alive is wrong data in the model.
            *result = Handle::default();
            return Ok(false);
        }
        if is_super {
            return Self::get_obj(&label.father(), result, is_super);
        }
        Ok(false)
    }
    /// `Detach(theLabel, theMode)` (static): detaches the object of the label; true when
    /// the label holds no object afterwards.
    pub fn detach_label(label: &TdfLabel, mode: TObjDeletingMode) -> Result<bool, Failure> {
        let mut object = Handle::default();
        if Self::get_obj(label, &mut object, false)? {
            return object.detach(mode);
        }
        Ok(true)
    }
}
