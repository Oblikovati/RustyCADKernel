// Rust port of DataExchange/TKXSBase/XSControl/XSControl_Controller (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: `Customise`.

//! `XSControl_Controller::Customise`: the work session prepared for a norm: its
//! parameters, its session items, the standard items (selections, signatures,
//! counters, dispatches) and the editor of the static parameters.
use super::{
    XSControlConnectedShapes, XSControlControllerBase, XSControlSelectForTransfer,
    XSControlSignTransferStatus, XSControlWorkSession,
};
use crate::if_select::{
    IFSelectDispPerCount, IFSelectDispPerFiles, IFSelectDispPerOne, IFSelectDispPerSignature,
    IFSelectDispatch, IFSelectEditor, IFSelectGeneralModifier, IFSelectGraphCounter,
    IFSelectIntParam, IFSelectParamEditor, IFSelectSelectDeduct, IFSelectSelectModelEntities,
    IFSelectSelectModelRoots, IFSelectSelectPointed, IFSelectSelectShared, IFSelectSelectSharing,
    IFSelectSelection, IFSelectSignAncestor, IFSelectSignCategory, IFSelectSignCounter,
    IFSelectSignType, IFSelectSignValidity, IFSelectSignature,
};
use crate::interface::InterfaceStatic;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};

/// Adds a named item to the session.
fn add(
    ws: &XSControlWorkSession,
    name: &str,
    item: Handle<StandardTransient>,
) -> Result<(), Failure> {
    ws.add_named_item(name, &item, true)?;
    Ok(())
}

/// A dispatch on the model roots, added under `name`.
fn add_dispatch(
    ws: &XSControlWorkSession,
    name: &str,
    disp: Handle<dyn IFSelectDispatch>,
    slr: &Handle<dyn IFSelectSelection>,
) -> Result<(), Failure> {
    disp.set_final_selection(slr);
    add(ws, name, disp.cast())
}

/// An integer parameter of value `value`.
fn int_param(value: i32) -> Handle<IFSelectIntParam> {
    let param = IFSelectIntParam::new();
    param.set_value(value);
    Handle::new(param)
}

/// The selections of the standard items: all the entities, the roots, the
/// transferable ones (the roots only for a norm other than STEP).
fn add_standard_selections(
    ws: &Handle<XSControlWorkSession>,
) -> Result<Handle<dyn IFSelectSelection>, Failure> {
    let sle: Handle<dyn IFSelectSelection> = Handle::new(IFSelectSelectModelEntities::new()).cast();
    add(ws, "xst-model-all", sle.cast())?;
    let slr: Handle<dyn IFSelectSelection> = Handle::new(IFSelectSelectModelRoots::new()).cast();
    add(ws, "xst-model-roots", slr.cast())?;
    let norm = String::from_utf8_lossy(ws.selected_norm(false).as_bytes()).into_owned();
    if !norm.eq_ignore_ascii_case("STEP") {
        let st1 = XSControlSelectForTransfer::new();
        st1.set_input(&slr);
        st1.set_reader(&ws.transfer_reader());
        add(ws, "xst-transferrable-roots", Handle::new(st1).cast())?;
    }
    let st2 = XSControlSelectForTransfer::new();
    st2.set_input(&sle);
    st2.set_reader(&ws.transfer_reader());
    add(ws, "xst-transferrable-all", Handle::new(st2).cast())?;
    let strs = XSControlSignTransferStatus::new();
    strs.set_reader(&ws.transfer_reader());
    add(ws, "xst-transfer-status", Handle::new(strs).cast())?;
    let scs = XSControlConnectedShapes::new();
    scs.set_reader(&ws.transfer_reader());
    add(ws, "xst-connected-faces", Handle::new(scs).cast())?;
    Ok(slr)
}

/// Two signatures: the long and the short type.
type TypeSignatures = (Handle<dyn IFSelectSignature>, Handle<dyn IFSelectSignature>);

/// The standard signatures and counters; the long type signature is returned.
fn add_standard_signatures(ws: &XSControlWorkSession) -> Result<TypeSignatures, Failure> {
    let stp: Handle<dyn IFSelectSignature> = Handle::new(IFSelectSignType::new(false)).cast();
    add(ws, "xst-long-type", stp.cast())?;
    let stc: Handle<dyn IFSelectSignature> = Handle::new(IFSelectSignType::new(true)).cast();
    add(ws, "xst-type", stc.cast())?;
    add(
        ws,
        "xst-ancestor-type",
        Handle::new(IFSelectSignAncestor::new(false)).cast(),
    )?;
    add(
        ws,
        "xst-types",
        Handle::new(IFSelectSignCounter::with_signature(&stp, false, true)?).cast(),
    )?;
    add(
        ws,
        "xst-category",
        Handle::new(IFSelectSignCategory::new()).cast(),
    )?;
    add(
        ws,
        "xst-validity",
        Handle::new(IFSelectSignValidity::new()).cast(),
    )?;
    Ok((stp, stc))
}

/// The standard dispatches on the model roots (one, count 5, files 10, signature).
fn add_standard_dispatches(
    ws: &XSControlWorkSession,
    slr: &Handle<dyn IFSelectSelection>,
    stc: &Handle<dyn IFSelectSignature>,
) -> Result<(), Failure> {
    add_dispatch(
        ws,
        "xst-disp-one",
        Handle::new(IFSelectDispPerOne::new()).cast(),
        slr,
    )?;
    let dispcount = IFSelectDispPerCount::new();
    dispcount.set_count(&int_param(5));
    add_dispatch(ws, "xst-disp-count", Handle::new(dispcount).cast(), slr)?;
    let dispfiles = IFSelectDispPerFiles::new();
    dispfiles.set_count(&int_param(10));
    add_dispatch(ws, "xst-disp-files", Handle::new(dispfiles).cast(), slr)?;
    let dispsign = IFSelectDispPerSignature::new();
    dispsign.set_sign_counter(
        &Handle::new(IFSelectSignCounter::with_signature(stc, true, false)?).cast(),
    )?;
    add_dispatch(ws, "xst-disp-sign", Handle::new(dispsign).cast(), slr)
}

/// The standard items of a session without them.
fn add_standard_items(ws: &Handle<XSControlWorkSession>) -> Result<(), Failure> {
    let slr = add_standard_selections(ws)?;
    let (stp, stc) = add_standard_signatures(ws)?;
    add_standard_dispatches(ws, &slr, &stc)?;
    add(
        ws,
        "xst-pointed",
        Handle::new(IFSelectSelectPointed::new()).cast(),
    )?;
    add(
        ws,
        "xst-sharing",
        Handle::new(IFSelectSelectSharing::new()).cast(),
    )?;
    add(
        ws,
        "xst-shared",
        Handle::new(IFSelectSelectShared::new()).cast(),
    )?;
    add(
        ws,
        "xst-nb-selected",
        Handle::new(IFSelectGraphCounter::new(true, false)).cast(),
    )?;
    ws.set_sign_type(&stp);
    Ok(())
}

/// The default `Customise(WS)`: the parameters of the controller, its session items,
/// the standard items (once), the session items to apply as modifiers of the share-out,
/// and the editor and form of all the static parameters.
pub(super) fn customise_base(
    base: &XSControlControllerBase,
    ws: &Handle<XSControlWorkSession>,
) -> Result<(), Failure> {
    let (params, uses) = base.params();
    ws.set_params(&params, &uses)?;
    for (name, item) in base.session_items() {
        ws.add_named_item(&String::from_utf8_lossy(name.as_bytes()), &item, true)?;
    }
    if ws.named_item("xst-model-all").is_null() {
        add_standard_items(ws)?;
    }
    for anitem in base.adaptor_applied() {
        let _name = ws.name(&anitem);
        ws.set_applied_modifier(
            &anitem.cast::<dyn IFSelectGeneralModifier>(),
            &ws.share_out().cast(),
        )?;
    }
    let listat = InterfaceStatic::items(0, "");
    let paramed = IFSelectParamEditor::static_editor(Some(&listat), "All Static Parameters")?;
    ws.add_named_item("xst-static-params-edit", &paramed.clone().cast(), true)?;
    let paramform = paramed.form(false, true)?;
    ws.add_named_item("xst-static-params", &paramform.cast(), true)?;
    Ok(())
}
