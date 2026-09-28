// Copyright (c) 2020 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! `Message_AttributeStream`: an attribute holding stream text (usually a JSON dump).
use super::attribute::{AttributeBase, BaseAttribute, dump_attribute};
use super::{Attribute, json};
use crate::standard::dump::DumpStream;
use crate::standard::{RuntimeType, RuntimeTyped};
use crate::tcollection::AsciiString;
use std::sync::Mutex;

/// Attribute storing stream content; `AttributeStream::new(b"\"x\": 1", &AsciiString::default())`.
pub struct AttributeStream {
    base: AttributeBase,
    stream: Mutex<Vec<u8>>,
}
impl AttributeStream {
    /// Creates the attribute from stream content; `AttributeStream::new(b"text", &name)`.
    pub fn new(stream: &[u8], name: &AsciiString) -> Self {
        let attribute = Self {
            base: AttributeBase::new(name),
            stream: Mutex::new(Vec::new()),
        };
        attribute.set_stream(stream);
        attribute
    }
    /// The stored content; `attribute.stream()`.
    pub fn stream(&self) -> Vec<u8> {
        self.stream
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
    /// Replaces the content with the source stream's C string (cut at a NUL); `attribute.set_stream(b"x")`.
    pub fn set_stream(&self, stream: &[u8]) {
        let text = stream.split(|&byte| byte == 0).next().unwrap_or_default();
        *self
            .stream
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = text.to_vec();
    }
}
message_runtime_type!(AttributeStream, "Message_AttributeStream", BaseAttribute);
impl Attribute for AttributeStream {
    fn attribute_base(&self) -> &AttributeBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    /// Class name, the base attribute dump, then `"Stream": {<content>}`.
    fn dump_json(&self, output: &mut DumpStream, depth: i32) {
        json::class_begin(output, Self::static_type().name());
        json::base_class(output, depth, |output, depth| {
            dump_attribute(&self.base, output, depth)
        });
        json::stream_value(output, "Stream", &self.stream());
    }
}
