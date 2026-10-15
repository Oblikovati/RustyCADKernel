// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Partition.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_Partition`: an object that holds a set of homogeneous objects of a model, with
//! a name prefix to name new objects.
use super::object_names::same_name;
use super::{TObjModelState, TObjObject, TObjObjectBase, TObjObjectData, TObjObjectState};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::{ExtendedString, HExtendedString},
};
use rustycad_tklcaf::{
    tdata_std::TDataStdName,
    tdf::{TdfLabel, TdfTagSource},
};

/// `TObj_Partition::DataTag`: the ranks of the partition's data labels.
pub struct TObjPartitionDataTag;
impl TObjPartitionDataTag {
    /// `DataTag_First`: `TObj_Object::DataTag_Last`.
    pub const FIRST: i32 = super::TObjObjectDataTag::LAST;
    /// `DataTag_LastIndex`: the last index used to name objects.
    pub const LAST_INDEX: i32 = Self::FIRST + 1;
    /// `DataTag_Last`: reserved up to `DataTag_First + 100`.
    pub const LAST: i32 = Self::FIRST + 100;
}

/// The private field `myPrefix` of `TObj_Partition`.
#[derive(Default)]
pub struct TObjPartitionBase {
    prefix: SharedState<Handle<HExtendedString>>,
}

/// `TObj_Partition`'s members over the embedded base; the `TObj_Object` overrides of a
/// partition are filled by [`crate::tobj_partition_object_members`].
pub trait TObjPartition: TObjObject {
    /// The embedded base with the native private field.
    fn partition_base(&self) -> &TObjPartitionBase;
    /// `Update` (virtual): does nothing in a partition.
    fn update(&self) -> bool {
        true
    }
    /// `NewLabel`: a new child of the child label (`TDF_TagSource::NewChild`).
    fn new_label(&self) -> Result<TdfLabel, Failure> {
        TdfTagSource::new_child_of(&self.get_child_label()?)
    }
    /// `SetNamePrefix(thePrefix)`.
    fn set_name_prefix(&self, prefix: &Handle<HExtendedString>) {
        *self.partition_base().prefix.write() = prefix.clone();
    }
    /// `GetNamePrefix`.
    fn get_name_prefix(&self) -> Handle<HExtendedString> {
        self.partition_base().prefix.read().clone()
    }
    /// `GetNewName(theIsToChangeCount)`: the prefix followed by the first free index after
    /// the last one; the last index moves only past names actually skipped. Null without
    /// prefix.
    fn get_new_name(&self, is_to_change_count: bool) -> Result<Handle<HExtendedString>, Failure> {
        let prefix = self.get_name_prefix();
        let Some(prefix) = prefix.get() else {
            return Ok(Handle::default());
        };
        let saved_rank = self.get_last_index()? + 1;
        let mut rank = saved_rank;
        let model = self.get_model()?;
        let model = rustycad_tkcdf::live(&model, "TObj_Partition::GetNewName")?;
        let dictionary = self.get_dictionary()?;
        let name = loop {
            let text = prefix.string().cat(ExtendedString::from_integer(rank));
            rank += 1;
            let name = Handle::new(HExtendedString::from(text));
            if !model.is_registered_name(&name, &dictionary)? {
                break name;
            }
        };
        rank -= 1;
        if is_to_change_count && rank > saved_rank {
            self.set_last_index(rank)?;
        }
        Ok(name)
    }
    /// `GetLastIndex`.
    fn get_last_index(&self) -> Result<i32, Failure> {
        self.get_integer(TObjPartitionDataTag::LAST_INDEX, 0)
    }
    /// `SetLastIndex(theIndex)`.
    fn set_last_index(&self, index: i32) -> Result<(), Failure> {
        self.set_integer(index, TObjPartitionDataTag::LAST_INDEX, 0)
            .map(|_| ())
    }
}
impl_transient_interface!(dyn TObjPartition, "TObj_Partition", dyn TObjObject);

impl TObjStandardPartition {
    /// `Create(theLabel, theSetName)` (static): a new partition on the label, with the
    /// last index 0.
    pub fn create(label: &TdfLabel, set_name: bool) -> Result<Handle<dyn TObjPartition>, Failure> {
        let partition = TObjStandardPartition::new(label, set_name)?;
        partition.set_last_index(0)?;
        Ok(partition.cast())
    }
    /// `GetPartition(theObject)` (static): the nearest partition above the object, null
    /// when none.
    pub fn get_partition(
        object: &Handle<dyn TObjObject>,
    ) -> Result<Handle<dyn TObjPartition>, Failure> {
        let Some(object) = object.get() else {
            return Ok(Handle::default());
        };
        let mut label = object.get_label().father();
        while !label.is_null() {
            let mut found = Handle::default();
            if TObjObjectBase::get_obj(&label, &mut found, true)? {
                let partition: Handle<dyn TObjPartition> = found.cast();
                if !partition.is_null() {
                    return Ok(partition);
                }
            }
            label = label.father();
        }
        Ok(Handle::default())
    }
}

/// The body of `TObj_Partition::SetName`: the name is set without registration (a
/// partition does not check the uniqueness of its name).
pub fn partition_set_name<T: TObjPartition + ?Sized>(
    partition: &T,
    name: &Handle<HExtendedString>,
) -> Result<bool, Failure> {
    if same_name(&partition.get_name()?, name)? {
        return Ok(true);
    }
    let text = rustycad_tkcdf::live(name, "TObj_Partition::SetName")?;
    TDataStdName::set(&partition.get_label(), &text.string())?;
    Ok(true)
}

/// The body of `TObj_Partition::copyData`: the object data, then the prefix, when the
/// target is a partition.
pub fn partition_copy_data<T: TObjPartition + ?Sized>(
    partition: &T,
    target: &Handle<dyn TObjObject>,
) -> Result<bool, Failure> {
    let target_partition: Handle<dyn TObjPartition> = target.cast();
    let Some(target_partition) = target_partition.get() else {
        return Ok(false);
    };
    if !super::object_clone::object_copy_data(partition, target)? {
        return Ok(false);
    }
    target_partition.set_name_prefix(&partition.get_name_prefix());
    Ok(true)
}

/// Fills the `TObj_Object` overrides of `TObj_Partition` (`SetName`, `AfterRetrieval`,
/// `copyData`) in the `TObjObject` impl of a partition.
#[macro_export]
macro_rules! tobj_partition_object_members {
    () => {
        fn set_name(
            &self,
            name: &rustycad_tkernel::standard::Handle<
                rustycad_tkernel::tcollection::HExtendedString,
            >,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            $crate::tobj::partition_set_name(self, name)
        }
        fn after_retrieval(&self) -> Result<(), rustycad_tkernel::standard::Failure> {
            Ok(())
        }
        fn copy_data(
            &self,
            target: &rustycad_tkernel::standard::Handle<dyn $crate::tobj::TObjObject>,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            $crate::tobj::partition_copy_data(self, target)
        }
    };
}

/// `TObj_Partition` itself; `TObjStandardPartition::create(&label, true)`.
pub struct TObjStandardPartition {
    transient: StandardTransient,
    object: TObjObjectBase,
    partition: TObjPartitionBase,
}
impl_transient!(TObjStandardPartition, "TObj_Partition", dyn TObjPartition, transient,
    interfaces: [dyn TObjObject, dyn TObjPartition]);

impl TObjStandardPartition {
    /// `TObj_Partition(theLabel, theSetName)` (protected natively): a partition attached
    /// to the label.
    pub fn new(label: &TdfLabel, set_name: bool) -> Result<Handle<TObjStandardPartition>, Failure> {
        let partition = Handle::new(Self::unattached(label));
        TObjObjectBase::attach(&partition.cast(), set_name)?;
        Ok(partition)
    }
    /// The persistence constructor (`DECLARE_TOBJOCAF_PERSISTENCE`): bound, not attached.
    pub fn persistence_new(label: &TdfLabel) -> Result<Handle<dyn TObjObject>, Failure> {
        let partition: Handle<dyn TObjObject> = Handle::new(Self::unattached(label)).cast();
        let object = rustycad_tkcdf::live(&partition, "TObj_Partition::Persistence_::New")?;
        object.object_base().bind_self(&partition);
        object.init_fields();
        Ok(partition)
    }
    fn unattached(label: &TdfLabel) -> Self {
        Self {
            transient: StandardTransient::default(),
            object: TObjObjectBase::new(label),
            partition: TObjPartitionBase::default(),
        }
    }
}

impl TObjObject for TObjStandardPartition {
    fn object_base(&self) -> &TObjObjectBase {
        &self.object
    }
    crate::tobj_partition_object_members!();
}
impl TObjPartition for TObjStandardPartition {
    fn partition_base(&self) -> &TObjPartitionBase {
        &self.partition
    }
}
