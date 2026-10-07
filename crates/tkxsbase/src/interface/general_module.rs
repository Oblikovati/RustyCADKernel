// Rust port of DataExchange/TKXSBase/Interface/Interface_GeneralModule (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_GeneralModule`: the services a norm gives on its entities by case number:
//! shared and implied entities, checks, copies, categories and names.
use super::{
    InterfaceCheck, InterfaceCopyTool, InterfaceEntityIterator, InterfaceInterfaceModel,
    InterfaceShareTool,
};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Failure, Handle, StandardTransient, Transient},
    tcollection::HAsciiString,
};

/// `Interface_GeneralModule`, abstract. The copy members return `Result` because the
/// copy tool they drive raises `Interface_InterfaceError` natively.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceGeneralModule;
/// fn copyable(module: &dyn InterfaceGeneralModule) -> bool {
///     module.can_copy(1, &Default::default())
/// }
/// ```
pub trait InterfaceGeneralModule: Transient {
    /// `FillShared(model, CN, ent, iter)`: by default `FillSharedCase` (the model is not
    /// used).
    fn fill_shared(
        &self,
        _model: &Handle<dyn InterfaceInterfaceModel>,
        cn: i32,
        ent: &Handle<StandardTransient>,
        iter: &mut InterfaceEntityIterator,
    ) {
        self.fill_shared_case(cn, ent, iter);
    }
    /// `FillSharedCase(CN, ent, iter)`: the entities `ent` shares.
    fn fill_shared_case(
        &self,
        cn: i32,
        ent: &Handle<StandardTransient>,
        iter: &mut InterfaceEntityIterator,
    );
    /// `Share(iter, shared)`: adds a shared entity to the iterator.
    fn share(&self, iter: &mut InterfaceEntityIterator, shared: &Handle<StandardTransient>) {
        iter.get_one_item(shared);
    }
    /// `ListImplied(model, CN, ent, iter)`: by default `ListImpliedCase`.
    fn list_implied(
        &self,
        _model: &Handle<dyn InterfaceInterfaceModel>,
        cn: i32,
        ent: &Handle<StandardTransient>,
        iter: &mut InterfaceEntityIterator,
    ) {
        self.list_implied_case(cn, ent, iter);
    }
    /// `ListImpliedCase(CN, ent, iter)`: no implied entity by default.
    fn list_implied_case(
        &self,
        _cn: i32,
        _ent: &Handle<StandardTransient>,
        _iter: &mut InterfaceEntityIterator,
    ) {
    }
    /// `CheckCase(CN, ent, shares, ach)`: the semantic check of an entity. Returns
    /// `Result`: `Interface_CheckTool` catches what a check raises.
    fn check_case(
        &self,
        cn: i32,
        ent: &Handle<StandardTransient>,
        shares: &InterfaceShareTool,
        ach: &mut Handle<InterfaceCheck>,
    ) -> Result<(), Failure>;
    /// `CanCopy(CN, ent)`: false by default.
    fn can_copy(&self, _cn: i32, _ent: &Handle<StandardTransient>) -> bool {
        false
    }
    /// `Dispatch(CN, entfrom, entto, TC)`: by default the entity itself (`entto` =
    /// `entfrom`), returns false.
    fn dispatch(
        &self,
        _cn: i32,
        entfrom: &Handle<StandardTransient>,
        entto: &mut Handle<StandardTransient>,
        _tc: &mut InterfaceCopyTool,
    ) -> Result<bool, Failure> {
        *entto = entfrom.clone();
        Ok(false)
    }
    /// `NewVoid(CN, entto)`: an empty entity of the case.
    fn new_void(&self, cn: i32, entto: &mut Handle<StandardTransient>) -> bool;
    /// `CopyCase(CN, entfrom, entto, TC)`: fills `entto` from `entfrom`.
    fn copy_case(
        &self,
        cn: i32,
        entfrom: &Handle<StandardTransient>,
        entto: &Handle<StandardTransient>,
        tc: &mut InterfaceCopyTool,
    ) -> Result<(), Failure>;
    /// `NewCopiedCase(CN, entfrom, entto, TC)`: false by default.
    fn new_copied_case(
        &self,
        _cn: i32,
        _entfrom: &Handle<StandardTransient>,
        _entto: &mut Handle<StandardTransient>,
        _tc: &mut InterfaceCopyTool,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    /// `RenewImpliedCase(CN, entfrom, entto, TC)`: nothing by default.
    fn renew_implied_case(
        &self,
        _cn: i32,
        _entfrom: &Handle<StandardTransient>,
        _entto: &Handle<StandardTransient>,
        _tc: &InterfaceCopyTool,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `WhenDeleteCase(CN, ent, dispatched)`: nothing by default.
    fn when_delete_case(&self, _cn: i32, _ent: &Handle<StandardTransient>, _dispatched: bool) {}
    /// `CategoryNumber(CN, ent, shares)`: 0 (not specified) by default.
    fn category_number(
        &self,
        _cn: i32,
        _ent: &Handle<StandardTransient>,
        _shares: &InterfaceShareTool,
    ) -> i32 {
        0
    }
    /// `Name(CN, ent, shares)`: null (not specified) by default.
    fn name(
        &self,
        _cn: i32,
        _ent: &Handle<StandardTransient>,
        _shares: &InterfaceShareTool,
    ) -> Handle<HAsciiString> {
        Handle::default()
    }
}
impl_transient_interface!(
    dyn InterfaceGeneralModule,
    "Interface_GeneralModule",
    StandardTransient
);
