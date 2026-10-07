// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_SelectAnyType,
// IFSelect_SelectType, IFSelect_SelectErrorEntities and IFSelect_SelectUnknownEntities
// (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extractions by kind of entity (a type, error entities, unknown entities).
use super::{IFSelectSelectExtract, IFSelectSelectExtractBase, impl_extract_selection};
use crate::interface::{InterfaceInterfaceModel, InterfaceModelEntities, InterfaceModelReports};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    standard::{
        Failure, Handle, RuntimeType, RuntimeTyped, SharedState, StandardTransient, exceptions,
    },
    tcollection::AsciiString,
};

/// `IFSelect_SelectAnyType`, abstract: an extraction of the entities of a kind
/// (`TypeForMatch`); its `Sort` is [`any_type_sort`].
pub trait IFSelectSelectAnyType: IFSelectSelectExtract {
    /// `TypeForMatch`.
    fn type_for_match(&self) -> &'static RuntimeType;
}
impl_transient_interface!(
    dyn IFSelectSelectAnyType,
    "IFSelect_SelectAnyType",
    dyn IFSelectSelectExtract
);

/// `IFSelect_SelectAnyType::Sort`: the entity is kind of `TypeForMatch` (a null entity
/// is natively dereferenced: `Standard_NullObject`).
pub fn any_type_sort<S: IFSelectSelectAnyType + ?Sized>(
    sel: &S,
    ent: &Handle<StandardTransient>,
) -> Result<bool, Failure> {
    let kind = ent
        .dynamic_type()
        .ok_or_else(|| exceptions::null_object("IFSelect_SelectAnyType::Sort: null entity"))?;
    Ok(kind.is_subtype(sel.type_for_match()))
}

/// `IFSelect_SelectType`: the entities of a type (`Standard_Transient` by default).
///
/// ```
/// use rustycad_tkxsbase::if_select::{IFSelectSelectType, IFSelectSelection};
/// assert_eq!(IFSelectSelectType::new().label().as_bytes(), b"Picked: Standard_Transient");
/// ```
pub struct IFSelectSelectType {
    transient: StandardTransient,
    base: IFSelectSelectExtractBase,
    thetype: SharedState<&'static RuntimeType>,
}
impl_transient!(IFSelectSelectType, "IFSelect_SelectType", dyn IFSelectSelectAnyType, transient, interfaces: [dyn super::IFSelectSelection, dyn super::IFSelectSelectDeduct, dyn IFSelectSelectExtract, dyn IFSelectSelectAnyType]);
impl_extract_selection!(IFSelectSelectType, base);

impl Default for IFSelectSelectType {
    fn default() -> Self {
        Self::with_type(StandardTransient::static_type())
    }
}

impl IFSelectSelectType {
    /// `IFSelect_SelectType()`: `Standard_Transient`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `IFSelect_SelectType(atype)`.
    pub fn with_type(atype: &'static RuntimeType) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: IFSelectSelectExtractBase::default(),
            thetype: SharedState::new(atype),
        }
    }
    /// `SetType(atype)`.
    pub fn set_type(&self, atype: &'static RuntimeType) {
        *self.thetype.write() = atype;
    }
}

impl IFSelectSelectExtract for IFSelectSelectType {
    fn extract_base(&self) -> &IFSelectSelectExtractBase {
        &self.base
    }
    fn sort(
        &self,
        _rank: i32,
        ent: &Handle<StandardTransient>,
        _model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        any_type_sort(self, ent)
    }
    /// `ExtractLabel`: the type name.
    fn extract_label(&self) -> AsciiString {
        AsciiString::new(self.type_for_match().name())
    }
}

impl IFSelectSelectAnyType for IFSelectSelectType {
    fn type_for_match(&self) -> &'static RuntimeType {
        *self.thetype.read()
    }
}

/// The live model of an extraction (a native null dereference without one).
fn live_model(
    model: &Handle<dyn InterfaceInterfaceModel>,
) -> Result<&dyn InterfaceInterfaceModel, Failure> {
    model.get().ok_or_else(|| {
        exceptions::null_object("IFSelect: extraction without model; expected a model")
    })
}

/// Declares an extraction whose only field is its extraction base.
macro_rules! plain_extract {
    ($(#[$doc:meta])* $name:ident, $native:literal) => {
        $(#[$doc])*
        #[derive(Default)]
        pub struct $name {
            transient: StandardTransient,
            base: IFSelectSelectExtractBase,
        }
        impl_transient!($name, $native, dyn IFSelectSelectExtract, transient, interfaces: [dyn super::IFSelectSelection, dyn super::IFSelectSelectDeduct, dyn IFSelectSelectExtract]);
        impl_extract_selection!($name, base);
        impl $name {
            #[doc = concat!("`", $native, "()`: direct, no input.")]
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
}

plain_extract!(
    /// `IFSelect_SelectErrorEntities`: the entities recorded in error in the model.
    IFSelectSelectErrorEntities, "IFSelect_SelectErrorEntities"
);

impl IFSelectSelectExtract for IFSelectSelectErrorEntities {
    fn extract_base(&self) -> &IFSelectSelectExtractBase {
        &self.base
    }
    /// `Sort`: the model records `ent` as an error entity.
    fn sort(
        &self,
        _rank: i32,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        let live = live_model(model)?;
        Ok(live.is_error_entity(live.number(ent)))
    }
    fn extract_label(&self) -> AsciiString {
        AsciiString::new("Error Entities")
    }
}

plain_extract!(
    /// `IFSelect_SelectUnknownEntities`: the entities of unrecognized type.
    IFSelectSelectUnknownEntities, "IFSelect_SelectUnknownEntities"
);

impl IFSelectSelectExtract for IFSelectSelectUnknownEntities {
    fn extract_base(&self) -> &IFSelectSelectExtractBase {
        &self.base
    }
    /// `Sort`: the model records `ent` as unknown.
    fn sort(
        &self,
        _rank: i32,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        let live = live_model(model)?;
        Ok(live.is_unknown_entity(live.number(ent)))
    }
    fn extract_label(&self) -> AsciiString {
        AsciiString::new("Unrecognized type Entities")
    }
}
