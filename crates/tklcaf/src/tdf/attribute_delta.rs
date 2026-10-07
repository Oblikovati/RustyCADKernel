// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_AttributeDelta.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_AttributeDelta`: the abstract difference between two states of one attribute.
use super::{TdfAttribute, TdfAttributeState, TdfLabel};
use crate::json::dump_label_entry;
use rustycad_tkcdf::json_dump::{class_begin, field_dumped};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{
        Failure, Guid, Handle, RuntimeTyped, StandardTransient, Transient, WeakHandle,
        dump::DumpStream, exceptions,
    },
};
use std::{io, sync::OnceLock};

/// The private fields of `TDF_AttributeDelta` and the self handle of the delta
/// (ocaf-design.md section 2); built by the protected constructor
/// `TDF_AttributeDelta(anAttribute)`.
pub struct TdfAttributeDeltaBase {
    this: OnceLock<WeakHandle<dyn TdfAttributeDelta>>,
    attribute: Handle<dyn TdfAttribute>,
    label: TdfLabel,
}
impl TdfAttributeDeltaBase {
    /// `TDF_AttributeDelta(anAttribute)`: the attribute and its current label.
    pub fn new(attribute: &Handle<dyn TdfAttribute>) -> Self {
        Self {
            this: OnceLock::new(),
            attribute: attribute.clone(),
            label: attribute
                .get()
                .map(|attribute| attribute.label())
                .unwrap_or_default(),
        }
    }
    /// The handle `this` converts to natively; `Standard_NullObject` before binding.
    pub fn this(&self) -> Result<Handle<dyn TdfAttributeDelta>, Failure> {
        let this = self.this.get().map(WeakHandle::upgrade).unwrap_or_default();
        if this.is_null() {
            return Err(exceptions::null_object(
                "TDF_AttributeDelta: the delta has no owning handle; expected TdfAttributeDeltaBase::share",
            ));
        }
        Ok(this)
    }
    /// Puts a new delta in its handle and binds the self handle;
    /// `TdfAttributeDeltaBase::share(TdfDeltaOnAddition { .. })`.
    pub fn share<T: TdfAttributeDelta + RuntimeTyped>(delta: T) -> Handle<T> {
        let handle = Handle::new(delta);
        let shared: Handle<dyn TdfAttributeDelta> = handle.cast();
        if let Some(delta) = handle.get() {
            let _ = delta.attribute_delta_base().this.set(shared.downgrade());
        }
        handle
    }
}

/// `TDF_AttributeDelta`'s members; a concrete delta embeds a [`TdfAttributeDeltaBase`].
pub trait TdfAttributeDelta: Transient {
    /// The embedded base with the native private fields.
    fn attribute_delta_base(&self) -> &TdfAttributeDeltaBase;
    /// `Apply` (pure virtual): undoes the change on the framework.
    fn apply(&self) -> Result<(), Failure>;
    /// `Label`: the label of the attribute when the delta was made.
    fn label(&self) -> TdfLabel {
        self.attribute_delta_base().label.clone()
    }
    /// `Attribute`; `delta.attribute()`.
    fn attribute(&self) -> Handle<dyn TdfAttribute> {
        self.attribute_delta_base().attribute.clone()
    }
    /// `ID`: the attribute's ID (`Standard_NullObject` for a null attribute).
    fn id(&self) -> Result<Guid, Failure> {
        Ok(crate::live(&self.attribute(), "TDF_AttributeDelta::ID")?.id())
    }
    /// `Dump` (and `operator<<`): `<delta type> at <entry> on <attribute type>`.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        let mut entry = rustycad_tkernel::tcollection::AsciiString::default();
        super::TdfTool::entry(&self.label(), &mut entry);
        let attribute = self.attribute();
        let attribute_type = attribute.dynamic_type().map_or("", |t| t.name());
        write!(
            stream,
            "{} at {}",
            self.dynamic_type().name(),
            String::from_utf8_lossy(entry.as_bytes())
        )?;
        write!(stream, " on {attribute_type}")
    }
    /// `DumpJson`: the attribute and the label entry.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_attribute_delta(self, stream, depth)
    }
}
impl_transient_interface!(
    dyn TdfAttributeDelta,
    "TDF_AttributeDelta",
    StandardTransient
);

/// The native `TDF_AttributeDelta::DumpJson`, also called as the base-class dump.
pub fn dump_attribute_delta<D: TdfAttributeDelta + ?Sized>(
    delta: &D,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    // OCCT_DUMP_TRANSIENT_CLASS_BEGIN names the static class of the member.
    class_begin(stream, "TDF_AttributeDelta")?;
    let attribute = delta.attribute();
    if let Some(attribute) = attribute.get() {
        let child = |s: &mut DumpStream, d: i32| attribute.dump_json(s, d);
        field_dumped(stream, depth, "myAttribute.get()", Some(&child))?;
    }
    dump_label_entry(stream, "aStrForTDF_Label", &delta.label())
}
