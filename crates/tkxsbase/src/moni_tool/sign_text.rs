// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_SignText (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MoniTool_SignText`: a text that identifies an object in a context.
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Handle, StandardTransient, Transient},
    tcollection::AsciiString,
};

/// `MoniTool_SignText`, abstract: gives a text as a signature of an object.
///
/// ```
/// use rustycad_tkernel::{impl_transient, standard::{Handle, StandardTransient}, tcollection::AsciiString};
/// use rustycad_tkxsbase::moni_tool::MoniToolSignText;
/// #[derive(Default)]
/// struct Silent { transient: StandardTransient }
/// impl_transient!(Silent, "Doc_Silent", dyn MoniToolSignText, transient, interfaces: [dyn MoniToolSignText]);
/// impl MoniToolSignText for Silent {
///     fn name(&self) -> String { "silent".into() }
///     fn text(&self, _: &Handle<StandardTransient>, _: &Handle<StandardTransient>) -> AsciiString {
///         AsciiString::default()
///     }
/// }
/// let ent: Handle<StandardTransient> = Handle::new(Silent::default()).cast();
/// assert_eq!(Silent::default().text_alone(&ent).as_bytes(), b"Doc_Silent");
/// ```
pub trait MoniToolSignText: Transient {
    /// `Name`: the identification of the signature.
    fn name(&self) -> String;
    /// `Text`: the signature of `ent` in `context` (which may be null).
    fn text(
        &self,
        ent: &Handle<StandardTransient>,
        context: &Handle<StandardTransient>,
    ) -> AsciiString;
    /// `TextAlone`: `Text` without context; when empty, "(NULL)" or the dynamic type name.
    fn text_alone(&self, ent: &Handle<StandardTransient>) -> AsciiString {
        sign_text_alone(self.text(ent, &Handle::default()), ent)
    }
}
impl_transient_interface!(dyn MoniToolSignText, "MoniTool_SignText", StandardTransient);

/// The native body of `MoniTool_SignText::TextAlone` after the call to `Text`, shared by
/// the overrides that call the base member; `sign_text_alone(text, &ent)`.
pub fn sign_text_alone(mut atext: AsciiString, ent: &Handle<StandardTransient>) -> AsciiString {
    if atext.length() != 0 {
        return atext;
    }
    match ent.dynamic_type() {
        None => atext.assign_cat("(NULL)"),
        Some(kind) => atext.assign_cat(kind.name()),
    }
    atext
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_falls_back_to_null_or_type_name() {
        let null: Handle<StandardTransient> = Handle::default();
        assert_eq!(
            sign_text_alone(AsciiString::default(), &null).as_bytes(),
            b"(NULL)"
        );
        let object = Handle::new(StandardTransient::default());
        let text = sign_text_alone(AsciiString::default(), &object);
        assert_eq!(text.as_bytes(), b"Standard_Transient");
        let kept = sign_text_alone(AsciiString::new("given"), &object);
        assert_eq!(kept.as_bytes(), b"given");
    }
}
