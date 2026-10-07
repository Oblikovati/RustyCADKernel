// Rust port of DataExchange/TKXSBase/Interface/Interface_SignType (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_SignType`: a signature of entities in a model, given by `Value`.
use super::InterfaceInterfaceModel;
use crate::moni_tool::MoniToolSignText;
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Handle, StandardTransient},
    tcollection::AsciiString,
};

/// `Interface_SignType`, abstract (a `MoniTool_SignText`). Its override of `Text` is
/// [`interface_sign_type_text`], which the implementations call from their
/// `MoniToolSignText::text`.
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceSignType, InterfaceSignTypeBase};
/// assert_eq!(InterfaceSignTypeBase::class_name("IGESData_Line"), "Line");
/// ```
pub trait InterfaceSignType: MoniToolSignText {
    /// `Value(ent, model)`: the signature of `ent` in `model`.
    fn value(
        &self,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> String;
}
impl_transient_interface!(
    dyn InterfaceSignType,
    "Interface_SignType",
    dyn MoniToolSignText
);

/// `Interface_SignType::Text(ent, context)`: `Value` when the context is a model and the
/// entity is not null, else empty.
pub fn interface_sign_type_text<S: InterfaceSignType + ?Sized>(
    sign: &S,
    ent: &Handle<StandardTransient>,
    context: &Handle<StandardTransient>,
) -> AsciiString {
    let model: Handle<dyn InterfaceInterfaceModel> = context.cast();
    if ent.is_null() || model.is_null() {
        return AsciiString::default();
    }
    AsciiString::new(&sign.value(ent, &model))
}

/// The static members of `Interface_SignType`.
pub struct InterfaceSignTypeBase;

impl InterfaceSignTypeBase {
    /// `ClassName(typnam)`: the type name after its first '_' (the whole name without).
    pub fn class_name(typnam: &str) -> &str {
        typnam.split_once('_').map_or(typnam, |(_, rest)| rest)
    }
}

#[cfg(test)]
mod tests {
    use super::InterfaceSignTypeBase;

    #[test]
    fn class_name_skips_the_package() {
        assert_eq!(
            InterfaceSignTypeBase::class_name("StepBasic_Product"),
            "Product"
        );
        assert_eq!(InterfaceSignTypeBase::class_name("NoPackage"), "NoPackage");
        assert_eq!(InterfaceSignTypeBase::class_name("A_B_C"), "B_C");
    }
}
