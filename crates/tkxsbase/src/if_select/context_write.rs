// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_ContextWrite (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_ContextWrite`: the context of writing a file: the model, the protocol, the
//! applied file modifiers (one at a time, with its entities) and the checks.
use super::{IFSelectAppliedModifiers, IFSelectGeneralModifier};
use crate::interface::{
    InterfaceCheck, InterfaceCheckIterator, InterfaceGraph, InterfaceHGraph,
    InterfaceInterfaceModel, InterfaceModelEntities, InterfaceProtocol,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, StandardTransient, exceptions},
    tcollection::AsciiString,
};
use std::sync::RwLockReadGuard;

/// `IFSelect_ContextWrite`.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectContextWrite;
/// use rustycad_tkernel::standard::Handle;
/// let ctx = IFSelectContextWrite::new(&Handle::default(), &Handle::default(), &Handle::default(), "out.stp");
/// assert_eq!((ctx.nb_modifiers(), ctx.file_name().as_bytes()), (0, &b"out.stp"[..]));
/// ```
pub struct IFSelectContextWrite {
    themodel: Handle<dyn InterfaceInterfaceModel>,
    theproto: Handle<dyn InterfaceProtocol>,
    thefile: AsciiString,
    theapply: Handle<IFSelectAppliedModifiers>,
    thehgraf: Handle<InterfaceHGraph>,
    thecheck: InterfaceCheckIterator,
    thenumod: i32,
    thenbent: i32,
    thecurr: i32,
    themodif: Handle<dyn IFSelectGeneralModifier>,
}

/// The native null dereference of a missing model.
fn no_model() -> Failure {
    exceptions::null_object("IFSelect_ContextWrite: no model; expected a model")
}

impl IFSelectContextWrite {
    /// `IFSelect_ContextWrite(model, proto, applieds, filename)`.
    pub fn new(
        model: &Handle<dyn InterfaceInterfaceModel>,
        proto: &Handle<dyn InterfaceProtocol>,
        applieds: &Handle<IFSelectAppliedModifiers>,
        filename: &str,
    ) -> Self {
        Self {
            themodel: model.clone(),
            theproto: proto.clone(),
            thefile: AsciiString::new(filename),
            theapply: applieds.clone(),
            thehgraf: Handle::default(),
            thecheck: InterfaceCheckIterator::new(),
            thenumod: 0,
            thenbent: 0,
            thecurr: 0,
            themodif: Handle::default(),
        }
    }
    /// `IFSelect_ContextWrite(hgraph, proto, applieds, filename)`: the model of the
    /// graph; `Standard_NullObject` for a null graph (a native null dereference).
    pub fn with_graph(
        hgraph: &Handle<InterfaceHGraph>,
        proto: &Handle<dyn InterfaceProtocol>,
        applieds: &Handle<IFSelectAppliedModifiers>,
        filename: &str,
    ) -> Result<Self, Failure> {
        let live = hgraph.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_ContextWrite: null graph; expected a graph")
        })?;
        let model = live.graph().model().clone();
        let mut ctx = Self::new(&model, proto, applieds, filename);
        ctx.thehgraf = hgraph.clone();
        Ok(ctx)
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.themodel.clone()
    }
    /// `Protocol`.
    pub fn protocol(&self) -> Handle<dyn InterfaceProtocol> {
        self.theproto.clone()
    }
    /// `FileName`.
    pub fn file_name(&self) -> &AsciiString {
        &self.thefile
    }
    /// `AppliedModifiers`.
    pub fn applied_modifiers(&self) -> Handle<IFSelectAppliedModifiers> {
        self.theapply.clone()
    }
    /// `Graph`: the graph, computed from the model and the protocol at the first call.
    pub fn graph(&mut self) -> Result<RwLockReadGuard<'_, InterfaceGraph>, Failure> {
        if self.thehgraf.is_null() {
            self.thehgraf = Handle::new(InterfaceHGraph::with_protocol(
                &self.themodel,
                &self.theproto,
                true,
            )?);
        }
        let live = self
            .thehgraf
            .get()
            .ok_or_else(|| exceptions::null_object("IFSelect_ContextWrite::Graph: no graph"))?;
        Ok(live.graph())
    }
    /// `NbModifiers`.
    pub fn nb_modifiers(&self) -> i32 {
        self.theapply
            .get()
            .map_or(0, IFSelectAppliedModifiers::count)
    }
    /// `SetModifier(numod)`: the modifier `numod` with its entities (false out of range).
    pub fn set_modifier(&mut self, numod: i32) -> Result<bool, Failure> {
        self.themodif = Handle::default();
        (self.thenumod, self.thenbent, self.thecurr) = (0, 0, 0);
        let Some(apply) = self.theapply.get() else {
            return Ok(false);
        };
        if numod < 1 || numod > apply.count() {
            return Ok(false);
        }
        if let Some((modif, nbent)) = apply.item(numod)? {
            (self.themodif, self.thenbent) = (modif, nbent);
        }
        Ok(true)
    }
    /// `FileModifier`.
    pub fn file_modifier(&self) -> Handle<dyn IFSelectGeneralModifier> {
        self.themodif.clone()
    }
    /// `IsForNone`: the current modifier has no entity.
    pub fn is_for_none(&self) -> bool {
        self.thenbent == 0
    }
    /// `IsForAll`: the current modifier applies to all the entities;
    /// `Standard_NullObject` without applied modifiers.
    pub fn is_for_all(&self) -> Result<bool, Failure> {
        Ok(self
            .theapply
            .get()
            .ok_or_else(|| {
                exceptions::null_object("IFSelect_ContextWrite::IsForAll: no applied modifiers")
            })?
            .is_for_all())
    }
    /// `NbEntities`.
    pub fn nb_entities(&self) -> i32 {
        self.thenbent
    }
    /// `Start`.
    pub fn start(&mut self) {
        self.thecurr = 1;
    }
    /// `More`.
    pub fn more(&self) -> bool {
        self.thecurr <= self.thenbent
    }
    /// `Next`.
    pub fn next(&mut self) {
        self.thecurr += 1;
    }
    /// `Value`: the current entity of the modifier; `Standard_NoSuchObject` without one.
    pub fn value(&self) -> Result<Handle<StandardTransient>, Failure> {
        if self.thecurr < 1 || self.thecurr > self.thenbent {
            return Err(exceptions::no_such_object("IFSelect_ContextWrite:Value"));
        }
        let apply = self.theapply.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_ContextWrite::Value: no applied modifiers")
        })?;
        let num = apply.item_num(self.thecurr)?;
        self.themodel.get().ok_or_else(no_model)?.value(num)
    }
    /// `AddCheck(check)`: a check with messages (number -1 for an entity out of the
    /// model).
    pub fn add_check(&mut self, check: &Handle<InterfaceCheck>) -> Result<(), Failure> {
        let Some(live) = check.get().filter(|c| c.nb_fails() + c.nb_warnings() != 0) else {
            return Ok(());
        };
        let ent = live.entity();
        let mut num = self.themodel.get().ok_or_else(no_model)?.number(&ent);
        if num == 0 && ent.is_some() {
            num = -1;
        }
        self.thecheck.add(check, num);
        Ok(())
    }
    /// `AddWarning(start, mess, orig = "")`.
    pub fn add_warning(
        &mut self,
        start: &Handle<StandardTransient>,
        mess: &str,
        orig: &str,
    ) -> Result<(), Failure> {
        let num = self.themodel.get().ok_or_else(no_model)?.number(start);
        self.thecheck.c_check(num).add_warning_text(mess, orig);
        Ok(())
    }
    /// `AddFail(start, mess, orig = "")`.
    pub fn add_fail(
        &mut self,
        start: &Handle<StandardTransient>,
        mess: &str,
        orig: &str,
    ) -> Result<(), Failure> {
        let num = self.themodel.get().ok_or_else(no_model)?.number(start);
        self.thecheck.c_check(num).add_fail_text(mess, orig);
        Ok(())
    }
    /// `CCheck(num = 0)`: the check of number `num` (with its entity in the model).
    pub fn c_check(&mut self, num: i32) -> Result<Handle<InterfaceCheck>, Failure> {
        let ach = self.thecheck.c_check(num);
        let model = self.themodel.get().ok_or_else(no_model)?;
        if num > 0 && num <= model.nb_entities() {
            ach.set_entity(&model.value(num)?);
        }
        Ok(ach)
    }
    /// `CCheck(ent)`: the check of `ent` (number -1 out of the model).
    pub fn c_check_entity(
        &mut self,
        ent: &Handle<StandardTransient>,
    ) -> Result<Handle<InterfaceCheck>, Failure> {
        let mut num = self.themodel.get().ok_or_else(no_model)?.number(ent);
        if num == 0 {
            num = -1;
        }
        let ach = self.thecheck.c_check(num);
        ach.set_entity(ent);
        Ok(ach)
    }
    /// `CheckList`.
    pub fn check_list(&self) -> InterfaceCheckIterator {
        self.thecheck.clone()
    }
}
