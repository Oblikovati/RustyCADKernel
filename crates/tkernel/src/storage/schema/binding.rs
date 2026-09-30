//! Type bindings and object registration against the current data set.
use super::Schema;
use crate::standard::{Failure, Handle, StandardPersistent};
use crate::storage::PersistentNumbers;
use crate::storage::{BaseDriver, CallBack, TypedCallBack};
use crate::tcollection::AsciiString;
use std::sync::Arc;

impl Schema {
    /// Protected `HasTypeBinding`: whether the current data set bound `type_name`.
    /// Example: `schema.has_type_binding(&name)?`.
    pub fn has_type_binding(&self, type_name: &AsciiString) -> Result<bool, Failure> {
        self.with_current("HasTypeBinding", |_, internal| {
            internal.lock().type_binding.is_bound(type_name)
        })
    }

    /// Protected `BindType`: gives an unbound type the next type number, both in the type
    /// section and in the bindings; `schema.bind_type(&name, Some(callback))?`.
    pub fn bind_type(
        &self,
        type_name: &AsciiString,
        callback: Option<Arc<dyn CallBack>>,
    ) -> Result<(), Failure> {
        if self.has_type_binding(type_name)? {
            return Ok(());
        }
        self.with_current("BindType", |data, internal| {
            let typed = TypedCallBack::new(type_name, callback);
            let mut fields = internal.lock();
            if let Some(types) = data.type_data().get() {
                types.add_type(type_name, fields.type_id);
            }
            typed.set_index(fields.type_id);
            fields.type_id += 1;
            fields
                .type_binding
                .bind(type_name.clone(), Handle::new(typed));
        })
    }

    /// Protected `TypeBinding`: the callback bound to `type_name`, `None` when unbound.
    /// Example: `schema.type_binding(&name)?`.
    pub fn type_binding(
        &self,
        type_name: &AsciiString,
    ) -> Result<Option<Arc<dyn CallBack>>, Failure> {
        self.with_current("TypeBinding", |_, internal| {
            let fields = internal.lock();
            let typed = fields.type_binding.seek(type_name)?;
            typed.get().and_then(TypedCallBack::callback)
        })
    }

    /// `AddPersistent`: numbers an object that has no type number yet with the index bound
    /// to `type_name` and the next reference; an unbound name is `Standard_NoSuchObject`
    /// (`DataMap::Find`). Returns whether the object was numbered.
    /// Example: `schema.add_persistent(&object, "PColStd_HArray1OfInteger")?`.
    pub fn add_persistent(
        &self,
        persistent: &Handle<StandardPersistent>,
        type_name: &str,
    ) -> Result<bool, Failure> {
        let Some(object) = persistent.get() else {
            return Ok(false);
        };
        self.with_current("AddPersistent", |_, internal| {
            if object.type_number() != 0 {
                return Ok(false);
            }
            let mut fields = internal.lock();
            let typed = fields.type_binding.find(&AsciiString::new(type_name))?;
            let index = typed.get().map_or(0, TypedCallBack::index);
            object.set_type_number(index);
            object.set_reference_number(fields.object_id);
            fields.object_id += 1;
            Ok(true)
        })?
    }

    /// `PersistentToAdd`: queues an object not yet queued or typed (reference set to -1).
    /// Example: `schema.persistent_to_add(&object)?`.
    pub fn persistent_to_add(
        &self,
        persistent: &Handle<StandardPersistent>,
    ) -> Result<bool, Failure> {
        let Some(object) = persistent.get() else {
            return Ok(false);
        };
        self.with_current("PersistentToAdd", |_, internal| {
            if object.type_number() != 0 || object.reference_number() == -1 {
                return false;
            }
            object.set_reference_number(-1);
            internal.lock().objects_to_add.append(persistent);
            true
        })
    }

    /// `WritePersistentObjectHeader`: the object's reference and type numbers. A null
    /// object is dereferenced natively; Rust reports `Standard_NullObject`.
    /// Example: `schema.write_persistent_object_header(&object, &mut driver)?`.
    pub fn write_persistent_object_header(
        &self,
        persistent: &Handle<StandardPersistent>,
        driver: &mut dyn BaseDriver,
    ) -> Result<(), Failure> {
        let object = persistent.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "Storage_Schema::WritePersistentObjectHeader: null object; expected a persistent",
                "",
            )
        })?;
        driver.write_persistent_object_header(object.reference_number(), object.type_number())
    }

    /// `WritePersistentReference`: the object's reference number, 0 for a null handle.
    /// Example: `schema.write_persistent_reference(&object, &mut driver)?`.
    pub fn write_persistent_reference(
        &self,
        persistent: &Handle<StandardPersistent>,
        driver: &mut dyn BaseDriver,
    ) -> Result<(), Failure> {
        let reference = persistent
            .get()
            .map_or(0, PersistentNumbers::reference_number);
        driver.put_reference(reference)
    }
}
