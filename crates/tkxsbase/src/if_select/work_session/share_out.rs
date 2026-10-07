// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the dispatches, modifiers and
// transformers.

//! The dispatches and the final modifiers of the share-out, the transformers and the
//! modifiers run on the model, and the reduction of the model to a selection.
use super::IFSelectWorkSession;
use crate::if_select::{
    IFSelectDispatch, IFSelectGeneralModifier, IFSelectModifier, IFSelectSelectPointed,
    IFSelectSelection, IFSelectShareOut, IFSelectTransformStandard, IFSelectTransformer,
};
use crate::interface::{InterfaceCheckIterator, InterfaceCopyTool, InterfaceModelEntities};
use rustycad_tkernel::{
    message::Message,
    standard::{Failure, Handle, StandardTransient, exceptions},
};
use std::fmt::Write as _;

/// The share-out of a session, `Standard_NullObject` when null.
fn live_share_out(sho: &Handle<IFSelectShareOut>) -> Result<&IFSelectShareOut, Failure> {
    sho.get()
        .ok_or_else(|| exceptions::null_object("IFSelect_WorkSession: no share-out"))
}

impl IFSelectWorkSession {
    /// `AppliedDispatches`: the idents of the dispatches of the share-out.
    pub fn applied_dispatches(&self) -> Result<Vec<i32>, Failure> {
        let sho = self.share_out();
        let live = live_share_out(&sho)?;
        (1..=live.nb_dispatches())
            .map(|i| Ok(self.item_ident(&live.dispatch(i)?.cast())))
            .collect()
    }
    /// `ClearShareOut(onlydisp)`.
    pub fn clear_share_out(&self, onlydisp: bool) -> Result<(), Failure> {
        live_share_out(&self.share_out())?.clear(onlydisp);
        Ok(())
    }
    /// `Dispatch(id)`: null when not a dispatch.
    pub fn dispatch(&self, id: i32) -> Handle<dyn IFSelectDispatch> {
        self.item(id).cast()
    }
    /// `DispatchRank(disp)`: its rank in the share-out, 0 when not an item.
    pub fn dispatch_rank(&self, disp: &Handle<dyn IFSelectDispatch>) -> Result<i32, Failure> {
        if self.item_ident(&disp.cast()) == 0 {
            return Ok(0);
        }
        Ok(live_share_out(&self.share_out())?.dispatch_rank(disp))
    }
    /// `NbFinalModifiers(formodel)`.
    pub fn nb_final_modifiers(&self, formodel: bool) -> Result<i32, Failure> {
        Ok(live_share_out(&self.share_out())?.nb_modifiers(formodel))
    }
    /// `FinalModifierIdents(formodel)`: the idents of the model or file modifiers.
    pub fn final_modifier_idents(&self, formodel: bool) -> Result<Vec<i32>, Failure> {
        let sho = self.share_out();
        let live = live_share_out(&sho)?;
        (1..=live.nb_modifiers(formodel))
            .map(|i| Ok(self.item_ident(&live.general_modifier(formodel, i)?.cast())))
            .collect()
    }
    /// `GeneralModifier(id)`: null when not a modifier.
    pub fn general_modifier(&self, id: i32) -> Handle<dyn IFSelectGeneralModifier> {
        self.item(id).cast()
    }
    /// `ModelModifier(id)`: null when not a model modifier.
    pub fn model_modifier(&self, id: i32) -> Handle<dyn IFSelectModifier> {
        self.item(id).cast()
    }
    /// `ModifierRank(modif)`: its rank in the share-out, 0 when not an item.
    pub fn modifier_rank(
        &self,
        modif: &Handle<dyn IFSelectGeneralModifier>,
    ) -> Result<i32, Failure> {
        if self.item_ident(&modif.cast()) == 0 {
            return Ok(0);
        }
        live_share_out(&self.share_out())?.modifier_rank(modif)
    }
    /// `ChangeModifierRank(formodel, before, after)`.
    pub fn change_modifier_rank(
        &self,
        formodel: bool,
        before: i32,
        after: i32,
    ) -> Result<bool, Failure> {
        Ok(live_share_out(&self.share_out())?.change_modifier_rank(formodel, before, after))
    }
    /// `ClearFinalModifiers`: the model then file modifiers removed as items.
    pub fn clear_final_modifiers(&self) -> Result<(), Failure> {
        for formodel in [true, false] {
            for id in self.final_modifier_idents(formodel)? {
                self.remove_item(&self.general_modifier(id).cast())?;
            }
        }
        Ok(())
    }
    /// `SetAppliedModifier(modif, item)`: the modifier applied to the whole share-out,
    /// to a dispatch, or (a model modifier) to a standard transformer instead of the
    /// share-out; false otherwise.
    pub fn set_applied_modifier(
        &self,
        modif: &Handle<dyn IFSelectGeneralModifier>,
        item: &Handle<StandardTransient>,
    ) -> Result<bool, Failure> {
        if self.item_ident(&modif.cast()) == 0 || item.is_null() {
            return Ok(false);
        }
        let sho = self.share_out();
        let live = live_share_out(&sho)?;
        if item.object_address() == sho.object_address() {
            live.add_modifier(modif, 0)?;
            return Ok(true);
        }
        let disp = item.cast::<dyn IFSelectDispatch>();
        if disp.is_some() {
            live.add_modifier(modif, 0)?;
            modif.set_dispatch(&disp);
            return Ok(true);
        }
        let stf_handle = item.cast::<IFSelectTransformStandard>();
        let Some(stf) = stf_handle.get() else {
            return Ok(false);
        };
        let tmod = modif.cast::<dyn IFSelectModifier>();
        if tmod.is_null() {
            return Ok(false);
        }
        stf.add_modifier(&tmod, 0);
        live.remove_item(&modif.cast())?;
        Ok(true)
    }
    /// `ResetAppliedModifier(modif)`: removed from the share-out.
    pub fn reset_applied_modifier(
        &self,
        modif: &Handle<dyn IFSelectGeneralModifier>,
    ) -> Result<bool, Failure> {
        if self.item_ident(&modif.cast()) == 0 {
            return Ok(false);
        }
        live_share_out(&self.share_out())?.remove_item(&modif.cast())
    }
    /// `UsesAppliedModifier(modif)`: the dispatch the modifier applies to, else the
    /// share-out; null when not applied.
    pub fn uses_applied_modifier(
        &self,
        modif: &Handle<dyn IFSelectGeneralModifier>,
    ) -> Result<Handle<StandardTransient>, Failure> {
        if self.item_ident(&modif.cast()) == 0 {
            return Ok(Handle::default());
        }
        let sho = self.share_out();
        if live_share_out(&sho)?.modifier_rank(modif)? == 0 {
            return Ok(Handle::default());
        }
        let disp = modif.get().map(|m| m.dispatch()).unwrap_or_default();
        Ok(if disp.is_null() {
            sho.cast()
        } else {
            disp.cast()
        })
    }
    /// `Transformer(id)`: null when not a transformer.
    pub fn transformer(&self, id: i32) -> Handle<dyn IFSelectTransformer> {
        self.item(id).cast()
    }
    /// `RunTransformer(transf)`: the transformer performed on the loaded model; 0 when
    /// not run, +/-1 without new model, +/-2 (4 for a new protocol, -4 when the graph
    /// fails) when the model is edited, +/-3 (5 for a new protocol) for a new model;
    /// negative when the transformer fails.
    pub fn run_transformer(
        &self,
        transf: &Handle<dyn IFSelectTransformer>,
    ) -> Result<i32, Failure> {
        let Some(live) = transf.get().filter(|_| self.is_loaded()) else {
            return Ok(0);
        };
        let (hgraph, protocol, model) = {
            let st = self.state.read();
            (
                st.thegraph.clone(),
                st.theprotocol.clone(),
                st.my_model.clone(),
            )
        };
        let mut newmod = Handle::default(); // Null au depart
        let mut checks = InterfaceCheckIterator::new();
        checks.set_name("X-STEP WorkSession : RunTransformer");
        let res = live.perform(&hgraph.graph(), &protocol, &mut checks, &mut newmod)?;
        if !checks.is_empty(false) {
            let mut text =
                String::from("  **    RunTransformer has produced Check Messages :    **\n");
            let mut out = Vec::new();
            let _ = checks.print_with_model(&mut out, &model, false, 0);
            text.push_str(&String::from_utf8_lossy(&out));
            let _ = write!(Message::send_info(), "{text}");
        }
        {
            let mut st = self.state.write();
            st.thecheckdone = false;
            st.thecheckrun = checks;
        }
        if newmod.is_null() {
            return Ok(if res { 1 } else { -1 });
        }
        for sp in self.items_of::<IFSelectSelectPointed>() {
            let live_sp = sp.get().ok_or_else(|| {
                exceptions::null_object(
                    "IFSelect_WorkSession::RunTransformer: pointed selection expected",
                )
            })?;
            live_sp.update_transformer(transf)?;
        }
        if newmod == model {
            if !res {
                return Ok(-2);
            }
            self.adopt_new_protocol(live);
            return Ok(if self.compute_graph(true)? { 4 } else { -4 });
        }
        if !res {
            return Ok(-3);
        }
        let effect = if self.adopt_new_protocol(live) { 5 } else { 3 };
        self.state.write().theoldel = model;
        self.set_model(&newmod, false)?;
        Ok(effect)
    }
    /// The protocol the transformer changes to, adopted; true when changed.
    fn adopt_new_protocol(&self, transf: &dyn IFSelectTransformer) -> bool {
        let mut newproto = self.protocol();
        if !transf.change_protocol(&mut newproto) {
            return false;
        }
        let gtool = {
            let mut st = self.state.write();
            st.theprotocol = newproto.clone();
            st.thegtool.clone()
        };
        if let Some(g) = gtool.get() {
            g.set_protocol(&newproto, false);
        }
        true
    }
    /// `RunModifier(modif, copy)`: `RunModifierSelected` without selection.
    pub fn run_modifier(
        &self,
        modif: &Handle<dyn IFSelectModifier>,
        copy: bool,
    ) -> Result<i32, Failure> {
        self.run_modifier_selected(modif, &Handle::default(), copy)
    }
    /// `RunModifierSelected(modif, sel, copy)`: a standard transformer of the modifier
    /// run; 0 when the modifier is not an item.
    pub fn run_modifier_selected(
        &self,
        modif: &Handle<dyn IFSelectModifier>,
        sel: &Handle<dyn IFSelectSelection>,
        copy: bool,
    ) -> Result<i32, Failure> {
        if self.item_ident(&modif.cast()) == 0 {
            return Ok(0);
        }
        let stf = IFSelectTransformStandard::new();
        stf.set_copy_option(copy);
        stf.set_selection(sel);
        stf.add_modifier(modif, 0);
        self.run_transformer(&Handle::new(stf).cast())
    }
    /// `NewTransformStandard(copy, name = "")`: a new standard transformer added under
    /// `name` (null when refused).
    pub fn new_transform_standard(
        &self,
        copy: bool,
        name: &str,
    ) -> Result<Handle<dyn IFSelectTransformer>, Failure> {
        let stf = IFSelectTransformStandard::new();
        stf.set_copy_option(copy);
        let handle: Handle<dyn IFSelectTransformer> = Handle::new(stf).cast();
        let added = self.add_named_item(name, &handle.cast(), true)?;
        Ok(if added == 0 {
            Handle::default()
        } else {
            handle
        })
    }
    /// `SetModelContent(sel, keep)`: the model reduced to the result of `sel` (`keep`)
    /// or to the other entities, with what they share; false when nothing remains.
    pub fn set_model_content(
        &self,
        sel: &Handle<dyn IFSelectSelection>,
        keep: bool,
    ) -> Result<bool, Failure> {
        let Some(live) = sel.get().filter(|_| self.is_loaded()) else {
            return Ok(false);
        };
        let (hgraph, protocol, model) = {
            let st = self.state.read();
            (
                st.thegraph.clone(),
                st.theprotocol.clone(),
                st.my_model.clone(),
            )
        };
        let list = live.unique_result(&hgraph.graph())?;
        if list.nb_entities() == 0 {
            return Ok(false);
        }
        let m = model.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::SetModelContent: no model")
        })?;
        let newmod = m.new_empty_model();
        let mut tc = InterfaceCopyTool::with_protocol(&model, &protocol)?;
        transfer_content(m, &list.content(), keep, &mut tc)?;
        tc.fill_model(&newmod)?;
        if newmod.nb_entities() == 0 {
            return Ok(false);
        }
        self.state.write().theoldel = model.clone();
        self.set_model(&newmod, false)?;
        for sp in self.items_of::<IFSelectSelectPointed>() {
            let live_sp = sp.get().ok_or_else(|| {
                exceptions::null_object(
                    "IFSelect_WorkSession::SetModelContent: pointed selection expected",
                )
            })?;
            live_sp.update_control(&tc.control())?;
        }
        Ok(true)
    }
}

/// The entities of `list` (`keep`) or the other entities of the model, transferred.
fn transfer_content(
    m: &dyn crate::interface::InterfaceInterfaceModel,
    list: &[Handle<StandardTransient>],
    keep: bool,
    tc: &mut InterfaceCopyTool,
) -> Result<(), Failure> {
    if keep {
        return list.iter().try_for_each(|ent| tc.transfer_entity(ent));
    }
    let nb = m.nb_entities();
    let mut flags = vec![false; nb as usize + 1];
    for ent in list {
        let num = m.number(ent);
        if num <= nb {
            flags[num as usize] = true;
        }
    }
    for i in 1..=nb {
        if !flags[i as usize] {
            tc.transfer_entity(&m.value(i)?)?;
        }
    }
    Ok(())
}
