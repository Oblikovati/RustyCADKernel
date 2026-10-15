// Rust port of ApplicationFramework/TKTObj/TObj/TObj_HiddenPartition.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_HiddenPartition`: a partition whose type flags lack `Visible`.
use super::{TObjObject, TObjObjectBase, TObjPartition, TObjPartitionBase, TObjTypeFlags};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tklcaf::tdf::TdfLabel;

/// `TObj_HiddenPartition`; `TObjHiddenPartition::new(&label)`.
pub struct TObjHiddenPartition {
    transient: StandardTransient,
    object: TObjObjectBase,
    partition: TObjPartitionBase,
}
impl_transient!(TObjHiddenPartition, "TObj_HiddenPartition", dyn TObjPartition, transient,
    interfaces: [dyn TObjObject, dyn TObjPartition]);

impl TObjHiddenPartition {
    /// `TObj_HiddenPartition(theLabel)`: a partition attached and named.
    pub fn new(label: &TdfLabel) -> Result<Handle<TObjHiddenPartition>, Failure> {
        let partition = Handle::new(Self::unattached(label));
        TObjObjectBase::attach(&partition.cast(), true)?;
        Ok(partition)
    }
    /// The persistence constructor (`DECLARE_TOBJOCAF_PERSISTENCE`): bound, not attached.
    pub fn persistence_new(label: &TdfLabel) -> Result<Handle<dyn TObjObject>, Failure> {
        let partition: Handle<dyn TObjObject> = Handle::new(Self::unattached(label)).cast();
        let object = rustycad_tkcdf::live(&partition, "TObj_HiddenPartition::Persistence_::New")?;
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

impl TObjObject for TObjHiddenPartition {
    fn object_base(&self) -> &TObjObjectBase {
        &self.object
    }
    crate::tobj_partition_object_members!();
    /// `GetTypeFlags`: the partition's flags without `Visible`.
    fn get_type_flags(&self) -> i32 {
        TObjTypeFlags::VISIBLE & !TObjTypeFlags::VISIBLE
    }
}
impl TObjPartition for TObjHiddenPartition {
    fn partition_base(&self) -> &TObjPartitionBase {
        &self.partition
    }
}
