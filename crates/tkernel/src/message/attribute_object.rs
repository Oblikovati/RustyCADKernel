//! `Message_AttributeObject`: an attribute holding a transient object.
use super::attribute::{AttributeBase, BaseAttribute, dump_attribute};
use super::{Attribute, json};
use crate::standard::dump::DumpStream;
use crate::standard::{Handle, RuntimeType, RuntimeTyped, StandardTransient};
use crate::tcollection::AsciiString;
use std::sync::Mutex;

/// Attribute storing an object handle; `AttributeObject::new(object, &AsciiString::default())`.
pub struct AttributeObject {
    base: AttributeBase,
    object: Mutex<Handle<StandardTransient>>,
}
impl AttributeObject {
    /// Creates the attribute; `AttributeObject::new(Handle::default(), &AsciiString::default())`.
    pub fn new(object: Handle<StandardTransient>, name: &AsciiString) -> Self {
        Self {
            base: AttributeBase::new(name),
            object: Mutex::new(object),
        }
    }
    /// The stored object (possibly null); `attribute.object()`.
    pub fn object(&self) -> Handle<StandardTransient> {
        self.object
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
    /// Replaces the stored object; `attribute.set_object(object)`.
    pub fn set_object(&self, object: Handle<StandardTransient>) {
        *self
            .object
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = object;
    }
}
message_runtime_type!(AttributeObject, "Message_AttributeObject", BaseAttribute);
impl Attribute for AttributeObject {
    fn attribute_base(&self) -> &AttributeBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    /// Class name, the base attribute dump, then `"Object": "<pointer>"`.
    fn dump_json(&self, output: &mut DumpStream, depth: i32) {
        json::class_begin(output, Self::static_type().name());
        json::base_class(output, depth, |output, depth| {
            dump_attribute(&self.base, output, depth)
        });
        let pointer = json::pointer_info(self.object().as_ptr().cast());
        json::field_string(output, "Object", pointer.as_bytes());
    }
}
