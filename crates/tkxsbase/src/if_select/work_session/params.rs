// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the parameters, signatures,
// selections and counters.

//! The parameters (integer and text), the signatures, the evaluation of the
//! selections, the selections of the dispatches and modifiers, and the counters.
use super::{IFSelectTransientList, IFSelectWorkSession, interrupted};
use crate::if_select::{
    IFSelectDispatch, IFSelectGeneralModifier, IFSelectIntParam, IFSelectSelectDeduct,
    IFSelectSelection, IFSelectSelectionIterator, IFSelectSignCounter, IFSelectSignCounterOps,
    IFSelectSignature,
};
use crate::interface::{InterfaceEntityIterator, InterfaceParamType, InterfaceStatic};
use rustycad_tkernel::{
    message::Message,
    standard::{Failure, Handle, StandardTransient, exceptions},
    tcollection::{AsciiString, HAsciiString},
};
use std::fmt::Write as _;

impl IFSelectWorkSession {
    /// The item `param` once added under `name`, null when refused.
    fn added_param<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
        &self,
        name: &str,
        param: Handle<T>,
    ) -> Result<Handle<T>, Failure> {
        let added = self.add_named_item(name, &param.cast(), true)?;
        Ok(if added == 0 { Handle::default() } else { param })
    }
    /// `NewParamFromStatic(statname, name = "")`: an integer parameter bound to an
    /// integer static, else the text value of the static, added under `name`; null for
    /// an unknown static.
    pub fn new_param_from_static(
        &self,
        statname: &str,
        name: &str,
    ) -> Result<Handle<StandardTransient>, Failure> {
        let stat = InterfaceStatic::static_named(statname);
        let Some(live) = stat.get() else {
            return Ok(Handle::default());
        };
        let param: Handle<StandardTransient> =
            if live.param_type() == InterfaceParamType::ParamInteger {
                let intpar = IFSelectIntParam::new();
                intpar.set_static_name(statname);
                Handle::new(intpar).cast()
            } else {
                live.h_string_value().cast()
            };
        if param.is_null() {
            return Ok(param);
        }
        self.added_param(name, param)
    }
    /// `IntParam(id)`: null when not an integer parameter.
    pub fn int_param(&self, id: i32) -> Handle<IFSelectIntParam> {
        self.item(id).cast()
    }
    /// `IntValue(it)`: 0 for null.
    pub fn int_value(&self, it: &Handle<IFSelectIntParam>) -> i32 {
        it.get().map_or(0, IFSelectIntParam::value)
    }
    /// `NewIntParam(name = "")`: a new integer parameter added under `name`.
    pub fn new_int_param(&self, name: &str) -> Result<Handle<IFSelectIntParam>, Failure> {
        self.added_param(name, Handle::new(IFSelectIntParam::new()))
    }
    /// `SetIntValue(it, val)`: false when not an item.
    pub fn set_int_value(&self, it: &Handle<IFSelectIntParam>, val: i32) -> bool {
        match it.get().filter(|_| self.item_ident(&it.cast()) != 0) {
            Some(par) => {
                par.set_value(val);
                true
            }
            None => false,
        }
    }
    /// `TextParam(id)`: null when not a text parameter.
    pub fn text_param(&self, id: i32) -> Handle<HAsciiString> {
        self.item(id).cast()
    }
    /// `TextValue(par)`: empty for null.
    pub fn text_value(&self, par: &Handle<HAsciiString>) -> AsciiString {
        par.get().map(|p| p.string().clone()).unwrap_or_default()
    }
    /// `NewTextParam(name = "")`: a new empty text parameter added under `name`.
    pub fn new_text_param(&self, name: &str) -> Result<Handle<HAsciiString>, Failure> {
        self.added_param(name, Handle::new(HAsciiString::new("")))
    }
    /// `SetTextValue(par, val)`: false when not an item.
    pub fn set_text_value(&self, par: &Handle<HAsciiString>, val: &str) -> bool {
        match par.get().filter(|_| self.item_ident(&par.cast()) != 0) {
            Some(text) => {
                text.clear();
                text.assign_cat(val);
                true
            }
            None => false,
        }
    }
    /// `Signature(id)`: null when not a signature.
    pub fn signature(&self, id: i32) -> Handle<dyn IFSelectSignature> {
        self.item(id).cast()
    }
    /// `SignValue(sign, ent)`: the value of `sign` for an entity of the model (empty
    /// otherwise).
    pub fn sign_value(
        &self,
        sign: &Handle<dyn IFSelectSignature>,
        ent: &Handle<StandardTransient>,
    ) -> String {
        let model = self.model();
        match sign.get() {
            Some(s) if model.is_some() && self.starting_number(ent) != 0 => s.value(ent, &model),
            _ => String::new(),
        }
    }
    /// `Selection(id)`: null when not a selection.
    pub fn selection(&self, id: i32) -> Handle<dyn IFSelectSelection> {
        self.item(id).cast()
    }
    /// `EvalSelection(sel)`: the unique result of `sel` on the graph (empty without
    /// graph); a guarded call.
    pub fn eval_selection(
        &self,
        sel: &Handle<dyn IFSelectSelection>,
    ) -> Result<InterfaceEntityIterator, Failure> {
        let result = self.guarded(&interrupted("EvalSelection"), || {
            let Some(hgraph) = self.current_graph() else {
                return Ok(InterfaceEntityIterator::new());
            };
            let live = sel.get().ok_or_else(|| {
                exceptions::null_object("IFSelect_WorkSession::EvalSelection: null selection")
            })?;
            live.unique_result(&hgraph.graph())
        })?;
        Ok(result.unwrap_or_default())
    }
    /// `Sources(sel)`: the inputs of `sel`.
    pub fn sources(
        &self,
        sel: &Handle<dyn IFSelectSelection>,
    ) -> Result<IFSelectSelectionIterator, Failure> {
        IFSelectSelectionIterator::of_selection(sel)
    }
    /// `SelectionResult(sel)`: the result of `sel` (empty when not loaded, `None` for a
    /// null selection); a guarded call.
    pub fn selection_result(
        &self,
        sel: &Handle<dyn IFSelectSelection>,
    ) -> Result<IFSelectTransientList, Failure> {
        let result = self.guarded(&interrupted("SelectionResult"), || {
            if !self.is_loaded() {
                println!(" ***  Data for Evaluation not available  ***");
                return Ok(Some(Vec::new()));
            }
            if sel.is_null() {
                println!(" Selection :  Unknown");
                return Ok(None);
            }
            Ok(Some(self.eval_selection(sel)?.content()))
        })?;
        Ok(result.flatten())
    }
    /// `SelectionResultFromList(sel, list)`: the result of `sel` with its deepest
    /// deduction input taking `list` (its alternate); `SelectionResult` for a null list
    /// or a selection which is not a deduction.
    pub fn selection_result_from_list(
        &self,
        sel: &Handle<dyn IFSelectSelection>,
        list: Option<&[Handle<StandardTransient>]>,
    ) -> Result<IFSelectTransientList, Failure> {
        let deduct = sel.cast::<dyn IFSelectSelectDeduct>();
        if list.is_none() || deduct.is_null() {
            return self.selection_result(sel);
        }
        let mut ssel = sel.clone();
        let mut current = deduct;
        for _ in 1..=self.max_ident() * 2 {
            let newinput = current.get().map(|d| d.input()).unwrap_or_default();
            current = newinput.cast();
            if current.is_null() {
                break;
            }
            ssel = newinput;
        }
        let last = ssel.cast::<dyn IFSelectSelectDeduct>();
        let live = last.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::SelectionResultFromList: no deduction")
        })?;
        live.alternate().set_list(list);
        self.selection_result(sel)
    }
    /// `SetItemSelection(item, sel)`: the final selection of a dispatch or the selection
    /// of a modifier (both items, `sel` an item or null); false otherwise.
    pub fn set_item_selection(
        &self,
        item: &Handle<StandardTransient>,
        sel: &Handle<dyn IFSelectSelection>,
    ) -> bool {
        let (disp, modif) = (
            item.cast::<dyn IFSelectDispatch>(),
            item.cast::<dyn IFSelectGeneralModifier>(),
        );
        let sel_known = sel.is_null() || self.item_ident(&sel.cast()) != 0;
        if let Some(d) = disp.get() {
            if self.item_ident(item) == 0 || !sel_known {
                return false;
            }
            d.set_final_selection(sel);
            return true;
        }
        let Some(m) = modif.get() else {
            return false;
        };
        if self.item_ident(item) == 0 || !sel_known {
            return false;
        }
        m.set_selection(sel);
        true
    }
    /// `ResetItemSelection(item)`: `SetItemSelection` with a null selection.
    pub fn reset_item_selection(&self, item: &Handle<StandardTransient>) -> bool {
        self.set_item_selection(item, &Handle::default())
    }
    /// `ItemSelection(item)`: the selection of a dispatch or modifier item (null else).
    pub fn item_selection(
        &self,
        item: &Handle<StandardTransient>,
    ) -> Handle<dyn IFSelectSelection> {
        if let Some(d) = item
            .cast::<dyn IFSelectDispatch>()
            .get()
            .filter(|_| self.item_ident(item) > 0)
        {
            return d.final_selection();
        }
        if let Some(m) = item
            .cast::<dyn IFSelectGeneralModifier>()
            .get()
            .filter(|_| self.item_ident(item) > 0)
        {
            return m.selection();
        }
        Handle::default() // Null or unknown -> Null
    }
    /// `SignCounter(id)`: null when not a counter.
    pub fn sign_counter(&self, id: i32) -> Handle<IFSelectSignCounter> {
        self.item(id).cast()
    }
    /// `ComputeCounter(counter, forced = false)`: the counter computed on its selection;
    /// false for null or without graph.
    pub fn compute_counter(
        &self,
        counter: &Handle<IFSelectSignCounter>,
        forced: bool,
    ) -> Result<bool, Failure> {
        let ops = counter.cast::<dyn IFSelectSignCounterOps>();
        let Some(live) = ops.get() else {
            return Ok(false);
        };
        if !self.compute_graph(false)? {
            return Ok(false);
        }
        live.compute_selected(&self.graph()?.graph(), forced)
    }
    /// `ComputeCounterFromList(counter, list, clear = true)`: the counter computed on
    /// `list` (on its selection for a null list).
    pub fn compute_counter_from_list(
        &self,
        counter: &Handle<IFSelectSignCounter>,
        list: Option<&[Handle<StandardTransient>]>,
        clear: bool,
    ) -> Result<bool, Failure> {
        let ops = counter.cast::<dyn IFSelectSignCounterOps>();
        let Some(live) = ops.get() else {
            return Ok(false);
        };
        if clear {
            live.clear();
        }
        if list.is_none() {
            return self.compute_counter(counter, true);
        }
        live.add_list(list, &self.model())?;
        Ok(true)
    }
}

/// Reports a message on the information messenger.
pub(crate) fn info(text: &str) {
    let _ = write!(Message::send_info(), "{text}");
}
