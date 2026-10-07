// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_ModelCopier (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the model copy and the whole sends.

//! `CopiedModel`, `SendAll`, `SendSelected` and `CopiedRemaining` of
//! [`IFSelectModelCopier`].
use super::model_copier_send::{IFSelectCopiedModelArgs, info, live, remaining_report, text_of};
use super::{
    IFSelectAppliedModifiers, IFSelectContextModif, IFSelectContextWrite, IFSelectGeneralModifier,
    IFSelectModelCopier, IFSelectShareOut, IFSelectWorkLibrary,
};
use crate::interface::{
    InterfaceCheckIterator, InterfaceCopyTool, InterfaceEntityIterator, InterfaceGeneralLib,
    InterfaceGraph, InterfaceInterfaceModel, InterfaceModelEntities, InterfaceProtocol,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, StandardTransient},
    tcollection::AsciiString,
};
use std::io::Write;

/// The output of the whole send: a file, or a stream.
enum SendTarget<'s> {
    File,
    Stream(&'s mut dyn Write),
}

/// False when the modifier is restricted to another dispatch than `dispnum` (only for a
/// dispatch rank > 0).
fn skips_dispatch(
    modif: &dyn IFSelectGeneralModifier,
    sho: &IFSelectShareOut,
    dispnum: i32,
) -> Result<bool, Failure> {
    Ok(dispnum > 0 && !modif.applies(&sho.dispatch(dispnum)?))
}

/// The numbers in `newmod` of the copies of the entities of `list`.
fn copied_numbers(
    list: &InterfaceEntityIterator,
    tc: &InterfaceCopyTool,
    newmod: &dyn InterfaceInterfaceModel,
) -> Result<Vec<i32>, Failure> {
    let mut numbers = Vec::new();
    for ent in list.content() {
        let mut newent: Handle<StandardTransient> = Handle::default();
        if tc.search(&ent, &mut newent)? {
            let num = newmod.number(&newent);
            if num > 0 {
                numbers.push(num);
            }
        }
    }
    Ok(numbers)
}

impl IFSelectModelCopier {
    /// `CopiedModel(G, WL, protocol, topcopy, filename, dispnum, numod, TC, newmod,
    /// applied, checks)`: for `dispnum > 0`, `newmod` becomes a new model with the copy
    /// of `tocopy` (counted in the remaining data); else a null `newmod` becomes the
    /// original model. The model modifiers of the share-out are performed on `newmod`;
    /// the file modifiers are returned as applied (null without them).
    pub fn copied_model(
        &self,
        args: &IFSelectCopiedModelArgs<'_>,
        tc: &mut InterfaceCopyTool,
        newmod: &mut Handle<dyn InterfaceInterfaceModel>,
        checks: &mut InterfaceCheckIterator,
    ) -> Result<Handle<IFSelectAppliedModifiers>, Failure> {
        let original = args.graph.model().clone();
        if args.dispnum > 0 {
            *newmod = live(&original, "model")?.new_empty_model();
            tc.clear()?;
            live(args.wl, "work library")?.copy_model(&original, newmod, args.tocopy, tc)?;
            self.update_copies(tc, |c| *c += 1)?;
        } else if newmod.is_null() {
            *newmod = original;
        }
        let sho = self.state.read().theshareout.clone();
        let Some(live_sho) = sho.get() else {
            return Ok(Handle::default());
        };
        self.perform_model_modifiers(args, live_sho, tc, newmod, checks)?;
        Self::applied_file_modifiers(args, live_sho, tc, newmod)
    }
    /// The model modifiers of `sho` performed on `newmod`, their checks merged.
    fn perform_model_modifiers(
        &self,
        args: &IFSelectCopiedModelArgs<'_>,
        sho: &IFSelectShareOut,
        tc: &mut InterfaceCopyTool,
        newmod: &Handle<dyn InterfaceInterfaceModel>,
        checks: &mut InterfaceCheckIterator,
    ) -> Result<(), Failure> {
        for i in 1..=sho.nb_modifiers(true) {
            let unmod = sho.model_modifier(i)?;
            let live_mod = live(&unmod, "model modifier")?;
            if skips_dispatch(live_mod, sho, args.dispnum)? {
                continue;
            }
            let mut ctx = IFSelectContextModif::with_copy(args.graph, tc, &text_of(args.filename))?;
            if let Some(sel) = live_mod.selection().get() {
                ctx.select(&sel.unique_result(args.graph)?)?;
            }
            if ctx.is_for_none() {
                continue;
            }
            live_mod.perform(&mut ctx, newmod, args.protocol, tc)?;
            checks.merge(&ctx.check_list());
        }
        Ok(())
    }
    /// The file modifiers of `sho` with the numbers in `newmod` of their selected
    /// entities (all entities without selection); null without file modifier.
    fn applied_file_modifiers(
        args: &IFSelectCopiedModelArgs<'_>,
        sho: &IFSelectShareOut,
        tc: &InterfaceCopyTool,
        newmod: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<Handle<IFSelectAppliedModifiers>, Failure> {
        let nbmod = sho.nb_modifiers(false);
        if nbmod == 0 {
            return Ok(Handle::default());
        }
        let model = live(newmod, "copied model")?;
        let applied = IFSelectAppliedModifiers::new(nbmod, model.nb_entities())?;
        for i in 1..=nbmod {
            let unmod = sho.general_modifier(false, i)?;
            let live_mod = live(&unmod, "file modifier")?;
            if skips_dispatch(live_mod, sho, args.dispnum)? {
                continue;
            }
            let selection = live_mod.selection();
            let Some(sel) = selection.get() else {
                applied.add_modif(&unmod)?; // empty -> we take all
                continue;
            };
            let numbers = copied_numbers(&sel.unique_result(args.graph)?, tc, model)?;
            if !numbers.is_empty() {
                applied.add_modif(&unmod)?;
                for num in numbers {
                    applied.add_num(num)?;
                }
            }
        }
        Ok(Handle::new(applied))
    }
    /// `SendAll(filename, G, WL, protocol)`: the whole model (each entity its own copy,
    /// the model modifiers performed on it) written as `filename`.
    pub fn send_all(
        &self,
        filename: &str,
        g: &InterfaceGraph,
        wl: &Handle<dyn IFSelectWorkLibrary>,
        protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<InterfaceCheckIterator, Failure> {
        self.send_all_to(filename, SendTarget::File, g, wl, protocol)
    }
    /// `SendAll(ostream, name, G, WL, protocol)`: as `send_all`, written to a stream.
    pub fn send_all_stream(
        &self,
        ostream: &mut dyn Write,
        name: &str,
        g: &InterfaceGraph,
        wl: &Handle<dyn IFSelectWorkLibrary>,
        protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<InterfaceCheckIterator, Failure> {
        self.send_all_to(name, SendTarget::Stream(ostream), g, wl, protocol)
    }
    /// The private `sendAll`: nothing without model, protocol or work library.
    fn send_all_to(
        &self,
        name: &str,
        target: SendTarget<'_>,
        g: &InterfaceGraph,
        wl: &Handle<dyn IFSelectWorkLibrary>,
        protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<InterfaceCheckIterator, Failure> {
        let mut checks = InterfaceCheckIterator::new();
        checks.set_name("X-STEP WorkSession : Send All");
        info("** WorkSession : Sending all data");
        let (Some(model), Some(library)) = (g.model().get(), wl.get()) else {
            return Ok(checks);
        };
        if protocol.is_null() {
            return Ok(checks);
        }
        let mut tc = InterfaceCopyTool::with_protocol(g.model(), protocol)?;
        for i in 1..=model.nb_entities() {
            let ent = model.value(i)?;
            tc.bind(&ent, &ent)?;
        }
        let (dest, empty) = (AsciiString::new(name), InterfaceEntityIterator::new());
        let args = IFSelectCopiedModelArgs {
            graph: g,
            wl,
            protocol,
            tocopy: &empty,
            filename: &dest,
            dispnum: 0,
            numod: 0,
        };
        let mut copied = Handle::default();
        let applied = self.copied_model(&args, &mut tc, &mut copied, &mut checks)?;
        let mut ctx = IFSelectContextWrite::new(&copied, protocol, &applied, name);
        let (written, fail) = match target {
            SendTarget::File => (
                library.write_file(&mut ctx)?,
                "SendAll (WriteFile) has failed",
            ),
            SendTarget::Stream(s) => (
                library.write_stream(&mut ctx, s)?,
                "SendAll (WriteStream) has failed",
            ),
        };
        checks.merge(&ctx.check_list());
        if !written {
            checks.c_check(0).add_fail_text(fail, "");
        }
        Ok(checks)
    }
    /// The model of the entities of `list` with what they share, each its own copy.
    fn selected_model(
        original: &Handle<dyn InterfaceInterfaceModel>,
        protocol: &Handle<dyn InterfaceProtocol>,
        list: &InterfaceEntityIterator,
        tc: &mut InterfaceCopyTool,
    ) -> Result<Handle<dyn InterfaceInterfaceModel>, Failure> {
        let newmod = live(original, "model")?.new_empty_model();
        tc.fill_model(&newmod)?; // for Header ...
        let lib = InterfaceGeneralLib::new(protocol);
        for ent in list.content() {
            newmod.add_with_refs_lib(&ent, &lib, 0, false)?;
        }
        for i in 1..=newmod.nb_entities() {
            let ent = newmod.value(i)?;
            tc.bind(&ent, &ent)?;
        }
        Ok(newmod)
    }
    /// `SendSelected(filename, G, WL, protocol, iter)`: the entities of `iter` with what
    /// they share written as `filename`, counted in the remaining data.
    pub fn send_selected(
        &self,
        filename: &str,
        g: &InterfaceGraph,
        wl: &Handle<dyn IFSelectWorkLibrary>,
        protocol: &Handle<dyn InterfaceProtocol>,
        list: &InterfaceEntityIterator,
    ) -> Result<InterfaceCheckIterator, Failure> {
        let mut checks = InterfaceCheckIterator::new();
        checks.set_name("X-STEP WorkSession : Send Selected");
        info("** WorkSession : Sending selected data");
        if g.model().is_null() || protocol.is_null() || wl.is_null() {
            return Ok(checks);
        }
        let mut tc = InterfaceCopyTool::with_protocol(g.model(), protocol)?;
        let mut newmod = Self::selected_model(g.model(), protocol, list, &mut tc)?;
        self.state
            .write()
            .theremain
            .get_or_insert_with(|| vec![0; g.size() as usize + 1]);
        let dest = AsciiString::new(filename);
        let pipo = InterfaceEntityIterator::new();
        let args = IFSelectCopiedModelArgs {
            graph: g,
            wl,
            protocol,
            tocopy: &pipo,
            filename: &dest,
            dispnum: 0,
            numod: 0,
        };
        let applied = self.copied_model(&args, &mut tc, &mut newmod, &mut checks)?;
        self.update_copies(&tc, |c| *c += 1)?;
        if !Self::write_model(wl, &newmod, protocol, &applied, filename, &mut checks)? {
            checks
                .c_check(0)
                .add_fail_text("SendSelected (WriteFile) has failed", "");
        }
        Ok(checks)
    }
    /// `CopiedRemaining(G, WL, TC)`: the copy of the entities of status 0 in `G` (null
    /// when none), with a report on the standard output; the copy counts become -1 for
    /// the others and 1 for the copied ones, over `0..=size + 1`.
    pub fn copied_remaining(
        &self,
        g: &InterfaceGraph,
        wl: &Handle<dyn IFSelectWorkLibrary>,
        tc: &mut InterfaceCopyTool,
    ) -> Result<Handle<dyn InterfaceInterfaceModel>, Failure> {
        let original = g.model().clone();
        let newmod = live(&original, "model")?.new_empty_model();
        tc.clear()?;
        let nb = g.size();
        let mut remain = vec![0; nb as usize + 2];
        let mut tocopy = InterfaceEntityIterator::new();
        for i in 1..=nb {
            if g.status(i)? == 0 {
                tocopy.add_item(&original.value(i)?);
            } else {
                remain[i as usize] = -1; //  ?? -1
            }
        }
        self.state.write().theremain = Some(remain);
        live(wl, "work library")?.copy_model(&original, &newmod, &tocopy, tc)?;
        if newmod.nb_entities() == 0 {
            return Ok(Handle::default());
        }
        self.update_copies(tc, |c| *c = 1)?;
        let remain = self.state.read().theremain.clone().unwrap_or_default();
        print!("{}", remaining_report(newmod.nb_entities(), &remain, nb));
        Ok(newmod)
    }
}
