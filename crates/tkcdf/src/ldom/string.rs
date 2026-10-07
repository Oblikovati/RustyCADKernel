// Rust port of ApplicationFramework/TKCDF/LDOM/LDOMString.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LDOMString`: an [`LdomBasicString`] that knows its document (ocaf-design.md section 9).
use super::LdomMemManager;
use super::basic_string::{LdomBasicString, LdomChars, LdomStringType, StringValue};
use crate::live;
use rustycad_tkernel::standard::{Failure, Handle};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// `LDOMString`; `LdomString::from_c_str(Some(b"tag"))`.
///
/// ```
/// use rustycad_tkcdf::ldom::{LdomString, LdomStringType};
/// let text = LdomString::from_integer(7);
/// assert_eq!(text.string_type(), LdomStringType::LdomInteger);
/// assert!(text.get_owner_document().is_err(), "a free string has no document");
/// ```
#[derive(Clone, Debug, Default)]
pub struct LdomString {
    base: LdomBasicString,
    /// `myPtrDoc`: the document that owns the string (null for a free string).
    document: Handle<LdomMemManager>,
}

impl Deref for LdomString {
    type Target = LdomBasicString;
    fn deref(&self) -> &LdomBasicString {
        &self.base
    }
}

impl DerefMut for LdomString {
    fn deref_mut(&mut self) -> &mut LdomBasicString {
        &mut self.base
    }
}

impl PartialEq for LdomString {
    /// The inherited `LDOMBasicString::operator==`: the document is not compared.
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base
    }
}

impl From<LdomBasicString> for LdomString {
    /// A string without document (the base-class part of a copy).
    fn from(base: LdomBasicString) -> Self {
        Self {
            base,
            document: Handle::default(),
        }
    }
}

impl LdomString {
    /// `LDOMString()`: the null string.
    pub fn new() -> Self {
        Self::default()
    }
    /// `LDOMString(aValue)` with an integer.
    pub fn from_integer(value: i32) -> Self {
        LdomBasicString::from_integer(value).into()
    }
    /// `LDOMString(const char*)`: a free copy, null for a null pointer.
    pub fn from_c_str(value: Option<&[u8]>) -> Self {
        LdomBasicString::from_c_str(value).into()
    }
    /// `getOwnerDocument`: the document; `Standard_NullObject` for a free string (natively
    /// a null dereference).
    pub fn get_owner_document(&self) -> Result<&LdomMemManager, Failure> {
        live(&self.document, "LDOMString::getOwnerDocument")
    }
    /// `operator=(LDOM_NullPtr*)`: the null string; the document is kept, as natively.
    pub fn set_null(&mut self) {
        self.base.set_null();
    }
    /// The base string (the native up-cast).
    pub fn basic(&self) -> &LdomBasicString {
        &self.base
    }
    /// `CreateDirectString(aValue, aDoc)`: a hashed string of the document, without copy.
    pub fn create_direct_string(value: &Arc<LdomChars>, document: &Handle<LdomMemManager>) -> Self {
        let mut base = LdomBasicString::new();
        base.set_direct(LdomStringType::LdomAsciiHashed, Arc::clone(value));
        Self {
            base,
            document: document.clone(),
        }
    }
    /// `LDOMString(anOther, const LDOM_MemManager&)`: the base string shared with `other`
    /// (a free string is copied), owned by the document.
    pub fn with_owner(other: &LdomBasicString, document: &Handle<LdomMemManager>) -> Self {
        Self {
            base: other.clone(),
            document: document.clone(),
        }
    }
    /// `LDOMString(anOther, const occ::handle<LDOM_MemManager>&)`: a copy into the
    /// document. An integer stays an integer; a free string becomes a document string; a
    /// document string is copied with its type; a hashed string is shared.
    pub fn copied_into(other: &LdomBasicString, document: &Handle<LdomMemManager>) -> Self {
        Self {
            base: other.document_copy(),
            document: document.clone(),
        }
    }
}

impl LdomBasicString {
    /// The value of `LDOMString(anOther, aDoc)`: an integer stays an integer; a free string
    /// becomes a document copy of its C string; a document string is copied with its type;
    /// a hashed string is shared; anything else is null.
    pub(crate) fn document_copy(&self) -> Self {
        let value = match &self.value {
            StringValue::Integer(value) => StringValue::Integer(*value),
            StringValue::Text(LdomStringType::LdomAsciiHashed, text) => {
                StringValue::Text(LdomStringType::LdomAsciiHashed, Arc::clone(text))
            }
            StringValue::Text(kind, _) => {
                let kind = if *kind == LdomStringType::LdomAsciiFree {
                    LdomStringType::LdomAsciiDoc
                } else {
                    *kind
                };
                StringValue::Text(kind, LdomChars::new(self.get_string(), None))
            }
            StringValue::Null => StringValue::Null,
        };
        Self { value }
    }
}
