//! The native public const-char pointer is borrowed; copying options must not copy its bytes.
use rustycad_tkernel::{standard::Failure, tcollection::AsciiString};
use std::{
    ffi::{CStr, c_char},
    marker::PhantomData,
};

/// A nullable, non-owning light-name pointer; `V3dLightName::from_c_str(c"sun")`.
/// Safe borrowed options cannot outlive the actual immutable buffer:
/// ```compile_fail
/// use rustycad_tkv3d::v3d::V3dLightName;
/// let name = {
///     let owner = std::ffi::CString::new("sun").unwrap();
///     V3dLightName::from_c_str(&owner)
/// };
/// name.to_ascii_string().unwrap();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)] // Keep the nullable pointer field's native physical representation.
pub struct V3dLightName<'anchor> {
    pointer: *const c_char,
    anchor: PhantomData<&'anchor CStr>,
}
impl<'anchor> V3dLightName<'anchor> {
    /// Borrow the actual immutable C string; `V3dLightName::from_c_str(c"sun")`.
    pub fn from_c_str(name: &'anchor CStr) -> Self {
        let pointer = name.as_ptr();
        Self {
            pointer,
            anchor: PhantomData,
        }
    }
    /// Preserve native null independently of empty text; `V3dLightName::null()`.
    pub fn null() -> Self {
        let pointer = std::ptr::null();
        Self {
            pointer,
            anchor: PhantomData,
        }
    }
    /// Borrow externally maintained C storage; `unsafe { V3dLightName::from_ptr(pointer) }`.
    ///
    /// # Safety
    /// Null is allowed. Otherwise storage must remain allocated for `'anchor` and be
    /// readable/NUL-terminated whenever consumed. Synchronize external edits with reads;
    /// no write, reallocation or release may overlap `to_ascii_string`. Reads happen at
    /// that call, including the ShadowMap boundary after Redraw, never at construction.
    pub unsafe fn from_ptr(pointer: *const c_char) -> Self {
        Self {
            pointer,
            anchor: PhantomData,
        }
    }
    /// The actual pointer, without dereferencing it; `name.as_ptr().is_null()`.
    pub fn as_ptr(self) -> *const c_char {
        // Pointer identity comparisons must not trigger a foreign-buffer read.
        self.pointer
    }
    /// Copy bytes at native consumption, preserving non-UTF8; `name.to_ascii_string()?`.
    pub fn to_ascii_string(self) -> Result<AsciiString, Failure> {
        if self.pointer.is_null() {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "TCollection_AsciiString(): NULL pointer passed to constructor; LightName=null, expected a NUL-terminated C string",
            ));
        }
        // The lifetime/foreign-storage contract is established at the two construction seams.
        let bytes = unsafe { CStr::from_ptr(self.pointer) }.to_bytes();
        Ok(AsciiString::from_raw(bytes))
    }
}
impl Default for V3dLightName<'_> {
    /// Native default is a static empty string, not null; `V3dLightName::default()`.
    fn default() -> Self {
        let empty = c"";
        Self::from_c_str(empty)
    }
}
