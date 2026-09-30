//! `Storage_InternalData`: the bookkeeping `Storage_Schema` keeps while writing and reading.
use super::{BucketOfPersistent, TypedCallBack};
use crate::ncollection::{DataMap, HArray1};
use crate::standard::{Handle, StandardPersistent, StandardTransient};
use crate::tcollection::AsciiString;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// A handle to the array of objects read so far, indexed by reference number.
pub type ReadArray = Handle<HArray1<Handle<StandardPersistent>>>;

/// The fields `Storage_Schema` (a friend class) edits.
pub(crate) struct InternalFields {
    pub(crate) objects_to_add: BucketOfPersistent,
    pub(crate) object_id: i32,
    pub(crate) type_id: i32,
    pub(crate) read_array: ReadArray,
    pub(crate) type_binding: DataMap<AsciiString, Handle<TypedCallBack>>,
}

/// Write/read bookkeeping of a set of data: object and type counters start at 1.
///
/// ```
/// use rustycad_tkernel::ncollection::HArray1;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkernel::storage::InternalData;
/// let internal = InternalData::default();
/// internal.read_array(|array| *array = Handle::new(HArray1::with_bounds(1, 2).unwrap()));
/// internal.clear();
/// assert!(internal.read_array(|array| array.is_null()));
/// ```
pub struct InternalData {
    parent: StandardTransient,
    fields: Mutex<InternalFields>,
}

impl Default for InternalData {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: Mutex::new(InternalFields {
                objects_to_add: BucketOfPersistent::default(),
                object_id: 1,
                type_id: 1,
                read_array: ReadArray::default(),
                type_binding: DataMap::default(),
            }),
        }
    }
}

impl InternalData {
    pub(crate) fn lock(&self) -> MutexGuard<'_, InternalFields> {
        self.fields.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// `ReadArray()`: lends the handle itself, which callers may replace, as the native
    /// reference return allows; `internal.read_array(|array| array.is_null())`.
    pub fn read_array<R>(&self, edit: impl FnOnce(&mut ReadArray) -> R) -> R {
        edit(&mut self.lock().read_array)
    }

    /// `Clear`: counters back to 1, no read array, no collected objects or type bindings.
    /// Example: `internal.clear()`.
    pub fn clear(&self) {
        let mut fields = self.lock();
        fields.type_id = 1;
        fields.object_id = 1;
        fields.read_array.nullify();
        fields.objects_to_add.clear();
        fields.type_binding.clear(false);
    }
}

crate::impl_transient!(
    InternalData,
    "Storage_InternalData",
    StandardTransient,
    parent
);
