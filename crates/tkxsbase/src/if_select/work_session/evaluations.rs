// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the evaluations and listings.

//! The printing of check and signature lists, the evaluation of a selection, of a
//! dispatch and of the whole share-out, and the listing of entities.
use super::params::info;
use super::{IFSelectWorkSession, write_out};
use crate::if_select::{
    IFSelectCheckCounter, IFSelectDispatch, IFSelectPacketList, IFSelectPrintCount,
    IFSelectSelection, IFSelectShareOutResult, IFSelectSignatureList, IFSelectSignatureListOps,
};
use crate::interface::{
    InterfaceCategory, InterfaceCheckIterator, InterfaceEntityIterator, InterfaceInterfaceModel,
    InterfaceModelEntities, InterfaceModelReports, InterfaceMsg, InterfaceShareFlags,
};
use rustycad_tkernel::standard::{Failure, Handle, exceptions};
use std::io::Write;

/// The header of the guarded evaluations.
fn evaluation_header(what: &str) -> String {
    format!("    ****    {what} Interrupted by Exception    ****  Title")
}

impl IFSelectWorkSession {
    /// `PrintCheckList(S, checklist, failsonly, mode)`: by entity, or counted by message
    /// (with the entities for the list modes).
    pub fn print_check_list(
        &self,
        s: &mut dyn Write,
        checklist: &InterfaceCheckIterator,
        failsonly: bool,
        mode: IFSelectPrintCount,
    ) -> Result<(), Failure> {
        let model = self.model();
        if mode == IFSelectPrintCount::ItemsByEntity {
            let mut out = Vec::new();
            let _ = checklist.print_with_model(&mut out, &model, failsonly, 0);
            return write_out(s, &out);
        }
        let withlist = (mode as i32) > 1 && mode != IFSelectPrintCount::CountSummary;
        let counter = IFSelectCheckCounter::new(withlist);
        counter.analyse(checklist, &model, true, failsonly)?;
        counter.print_list(s, &model, mode)
    }
    /// `PrintSignatureList(S, signlist, mode)`: nothing for null.
    pub fn print_signature_list(
        &self,
        s: &mut dyn Write,
        signlist: &Handle<IFSelectSignatureList>,
        mode: IFSelectPrintCount,
    ) -> Result<(), Failure> {
        match signlist.cast::<dyn IFSelectSignatureListOps>().get() {
            Some(list) => list.print_list(s, &self.model(), mode),
            None => Ok(()),
        }
    }
    /// `EvaluateSelection(sel)`: the entities of the result of a selection item; a
    /// guarded call.
    pub fn evaluate_selection(&self, sel: &Handle<dyn IFSelectSelection>) -> Result<(), Failure> {
        self.guarded(&evaluation_header("EvaluateSelection"), || {
            if !self.is_loaded() {
                info(" ***  Data for Evaluation not available  ***\n");
                return Ok(());
            }
            let Some(live) = sel.get().filter(|_| self.item_ident(&sel.cast()) != 0) else {
                info(" Selection :  Unknown\n"); // sout<<Handle
                return Ok(());
            };
            let mut out = Vec::new();
            self.list_entities(&self.eval_selection(sel)?, 1, &mut out)?;
            let _ = writeln!(
                out,
                "****  (Unique) RootResult, Selection \t: {}",
                String::from_utf8_lossy(live.label().as_bytes())
            );
            info(&String::from_utf8_lossy(&out));
            Ok(())
        })?;
        Ok(())
    }
    /// `EvaluateDispatch(disp, mode = 0)`: the packets of a dispatch of the share-out
    /// (roots for mode 0, contents else), then the entities forgotten (modes 1, 3) and
    /// duplicated (modes >= 2); a guarded call.
    pub fn evaluate_dispatch(
        &self,
        disp: &Handle<dyn IFSelectDispatch>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.guarded(&evaluation_header("EvaluateDispatch"), || {
            let numdisp = self.dispatch_rank(disp)?;
            if !self.is_loaded() {
                info(" ***  Data for List not available  ***\n");
                return Ok(());
            }
            let nbdisp = self.share_out().get().map_or(0, |s| s.nb_dispatches());
            if nbdisp < numdisp || numdisp <= 0 {
                info("Dispatch :  Unknown\n"); // sout<<Handle
                return Ok(());
            }
            let live = disp.get().ok_or_else(|| {
                exceptions::null_object("IFSelect_WorkSession::EvaluateDispatch: null dispatch")
            })?;
            if live.final_selection().is_null() {
                info("Dispatch  : No Final Selection\n"); // sout<<Handle
                return Ok(());
            }
            let mut out = format!(
                " --- Dispatch Label : {}\n",
                String::from_utf8_lossy(live.label().as_bytes())
            )
            .into_bytes();
            let hgraph = self.current_graph().ok_or_else(|| {
                exceptions::null_object("IFSelect_WorkSession::EvaluateDispatch: no graph")
            })?;
            let mut eval = IFSelectShareOutResult::with_dispatch_graph(disp, &hgraph.graph())?;
            eval.evaluate()?;
            let evres = eval.packets(mode != 0)?;
            self.print_packets(
                &mut out,
                &evres,
                mode,
                "    ****    Starting Entities not taken by this Dispatch    ****",
                "    ****    Entites in more than one packet    ****",
            )?;
            info(&String::from_utf8_lossy(&out));
            Ok(())
        })?;
        Ok(())
    }
    /// `EvaluateComplete(mode = 0)`: as `EvaluateDispatch` for the whole share-out.
    pub fn evaluate_complete(&self, mode: i32) -> Result<(), Failure> {
        self.guarded(&super::interrupted("EvaluateComplete"), || {
            if !self.is_loaded() {
                info(" ***  Data for List not available  ***\n");
                return Ok(());
            }
            let hgraph = self.current_graph().ok_or_else(|| exceptions::null_object("IFSelect_WorkSession::EvaluateComplete: no graph"))?;
            let mut eval = IFSelectShareOutResult::with_share_out_graph(&self.share_out(), &hgraph.graph())?;
            eval.evaluate()?;
            let mut out = format!("\n********    Evaluation ShareOutResult (Complete)    ********\n    ****    List of Packets    ****  Count : {}\n", eval.nb_packets()?).into_bytes();
            out.extend_from_slice(if mode == 0 { b" ** (for each one : Root Entities)  **\n" } else { b" ** (for each one : Evaluated Content)  **\n" });
            let evres = eval.packets(mode != 0)?;
            self.print_packets(&mut out, &evres, mode, "    ****    Starting Entities Forgotten    ****", "    ****    Entites in more than one packet    ****\n")?;
            info(&String::from_utf8_lossy(&out));
            Ok(())
        })?;
        Ok(())
    }
    /// The packets of an evaluation, the forgotten entities (`forgotten` title) and the
    /// duplicated ones (`duplicated` title).
    fn print_packets(
        &self,
        out: &mut Vec<u8>,
        evres: &IFSelectPacketList,
        mode: i32,
        forgotten: &str,
        duplicated: &str,
    ) -> Result<(), Failure> {
        let nbpack = evres.nb_packets();
        let _ = writeln!(out, "Nb Packets produced : {nbpack} :");
        for numpack in 1..=nbpack {
            let _ = writeln!(out, "\n    ****    Packet n0 : {numpack} ****");
            if mode == 0 {
                println!("Root Entities :");
            }
            self.list_entities(
                &evres.entities(numpack)?,
                if mode != 0 { 2 } else { -1 },
                out,
            )?;
        }
        if mode == 0 {
            return Ok(());
        }
        if mode == 1 || mode == 3 {
            out.push(b'\n');
            if evres.nb_duplicated(0, false) == 0 {
                out.extend_from_slice(b"    ****    All the Model is taken into account    ****\n");
            } else {
                let _ = writeln!(out, "{forgotten}");
                self.list_entities(&evres.duplicated(0, false)?, 2, out)?;
            }
        }
        if mode >= 2 {
            self.print_duplicated(out, evres, duplicated)?;
        }
        Ok(())
    }
    /// The entities put in several packets.
    fn print_duplicated(
        &self,
        out: &mut Vec<u8>,
        evres: &IFSelectPacketList,
        title: &str,
    ) -> Result<(), Failure> {
        out.extend_from_slice(title.as_bytes());
        let max = evres.highest_duplication_count();
        if max < 2 {
            out.extend_from_slice(b" :   There are none\n");
            return Ok(());
        }
        out.push(b'\n');
        for newcount in 2..=max {
            if evres.nb_duplicated(newcount, false) == 0 {
                continue;
            }
            let _ = writeln!(out, "    ****   Entities put in {newcount} packets    ****");
            self.list_entities(&evres.duplicated(newcount, false)?, 2, out)?;
        }
        Ok(())
    }
    /// `ListEntities(iter, mode, S)`: the count (mode >= 0), then the entities: by line
    /// with their category, validity and type (1), their numbers (2), or ten by line
    /// with their root, unknown and redefined marks (0, or negative). An error is
    /// reported in the listing.
    pub fn list_entities(
        &self,
        iter: &InterfaceEntityIterator,
        mmode: i32,
        s: &mut dyn Write,
    ) -> Result<(), Failure> {
        let mut out = Vec::new();
        if mmode >= 0 {
            let _ = writeln!(out, " List of {} Entities :", iter.nb_entities());
        }
        if !self.is_loaded() {
            out.extend_from_slice(b" ***  Data for List not available  ***\n");
            return write_out(s, &out);
        }
        if let Err(failure) = self.list_entities_body(iter, mmode.abs(), &mut out) {
            let _ = write!(
                out,
                "    ****    ListEntities Interrupted by Exception :   ****\n{}\n    Abandon\n",
                failure.message()
            );
        }
        write_out(s, &out)
    }
    /// The body of `ListEntities` for the mode `mode` (>= 0).
    fn list_entities_body(
        &self,
        iter: &InterfaceEntityIterator,
        mode: i32,
        out: &mut Vec<u8>,
    ) -> Result<(), Failure> {
        let hgraph = self.current_graph().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::ListEntities: no graph")
        })?;
        let tool = InterfaceShareFlags::from_graph(&hgraph.graph())?;
        let model = self.model();
        let m = model.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::ListEntities: no model")
        })?;
        let (mut newcount, mut cnt) = (-1, 0);
        for (rank, ent) in iter.content().iter().enumerate() {
            if rank == 0 {
                out.extend_from_slice(match mode {
                    1 => b"Number/Id.           Category Validity    Type\n-----------          ----....\n".as_slice(),
                    0 => b"  Keys : R Root   ? Unknown   * Unloaded\n",
                    2 => b"(",
                    _ => b"",
                });
            }
            let num = m.number(ent);
            match mode {
                1 => self.list_line(out, m, &tool, ent, num)?,
                2 => {
                    newcount += 1;
                    if newcount > 0 {
                        out.push(b',');
                    }
                    let _ = write!(out, "{num}");
                }
                _ => list_marked(out, m, &tool, ent, num, (&mut newcount, &mut cnt))?,
            }
        }
        if mode == 0 {
            out.push(b'\n');
        }
        if mode == 2 {
            out.extend_from_slice(b")\n");
        }
        Ok(())
    }
    /// One line of `ListEntities` mode 1.
    fn list_line(
        &self,
        out: &mut Vec<u8>,
        m: &dyn InterfaceInterfaceModel,
        tool: &InterfaceShareFlags,
        ent: &Handle<rustycad_tkernel::standard::StandardTransient>,
        num: i32,
    ) -> Result<(), Failure> {
        out.extend_from_slice(InterfaceMsg::blanks_for_integer(num, 6).as_bytes());
        let _ = m.print(ent, out, 0);
        out.extend_from_slice(if tool.is_shared(ent)? {
            b"       "
        } else {
            b" #ROOT#"
        });
        let catnum = m.category_number(num);
        if catnum > 0 {
            let _ = write!(out, "  {}", InterfaceCategory::name(catnum));
        }
        let _ = writeln!(
            out,
            "  ({})   Type:{}",
            self.validity_name(ent),
            m.type_name(ent, false)
        );
        Ok(())
    }
}

/// One entity of `ListEntities` mode 0, ten by line, with its marks.
fn list_marked(
    out: &mut Vec<u8>,
    m: &dyn InterfaceInterfaceModel,
    tool: &InterfaceShareFlags,
    ent: &Handle<rustycad_tkernel::standard::StandardTransient>,
    num: i32,
    (newcount, cnt): (&mut i32, &mut i32),
) -> Result<(), Failure> {
    *newcount += 1;
    *cnt += 1;
    if *newcount >= 10 {
        let _ = write!(out, "\n[{cnt}]:");
        *newcount = 1;
    }
    if *newcount > 0 {
        out.push(b'\t');
    }
    let _ = m.print(ent, out, 0);
    let mut mods = 0;
    if !tool.is_shared(ent)? {
        out.extend_from_slice(b"(R");
        mods += 1;
    }
    for (mark, on) in [
        (b'?', m.is_unknown_entity(num)),
        (b'*', m.is_redefined_content(num)),
    ] {
        if on {
            out.extend_from_slice(&[if mods == 0 { b'(' } else { b' ' }, mark]);
            mods += 1;
        }
    }
    if mods > 0 {
        out.push(b')');
        *newcount += 1;
    }
    Ok(())
}
