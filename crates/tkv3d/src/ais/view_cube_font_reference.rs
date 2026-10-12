//! Font() aliases the HAsciiString payload chosen once, rather than the replaceable TextFont slot.
use rustycad_tkernel::{
    standard::{Handle, HandleObserver},
    tcollection::{AsciiString, HAsciiString},
};
use std::marker::PhantomData;
/// Retained native font-string alias; `let old=cube.font_ref(); cube.set_font(&next); old.value()`.
/// Replacing TextFont does not redirect this view; editing the retained original string does.
pub struct AisViewCubeFontRef<'a> {
    font: Option<HandleObserver<HAsciiString>>,
    scope: PhantomData<&'a super::AisViewCubePresentationBase>,
}
impl<'a> AisViewCubeFontRef<'a> {
    pub(super) fn new(
        font: &Handle<HAsciiString>,
        _: &'a super::AisViewCubePresentationBase,
    ) -> Self {
        Self {
            font: (!font.is_null()).then(|| font.observe()),
            scope: PhantomData,
        }
    }
    /// Copy the current bound payload or the immutable native empty string; `alias.value()`.
    pub fn value(&self) -> AsciiString {
        self.font
            .as_ref()
            .map_or_else(AsciiString::default, |font| font.string().clone())
    }
    /// Compare actual string storage, with all native static-empty aliases equal; `left.is_same_reference(&right)`.
    pub fn is_same_reference(&self, other: &AisViewCubeFontRef<'_>) -> bool {
        match (&self.font, &other.font) {
            (None, None) => true,
            (Some(left), Some(right)) => left.object_address() == right.object_address(),
            _ => false,
        }
    }
    /// Return the chosen HAsciiString address, or null for the static empty payload; `alias.object_address()`.
    pub fn object_address(&self) -> *const () {
        self.font
            .as_ref()
            .map_or(std::ptr::null(), |font| font.object_address())
    }
}
