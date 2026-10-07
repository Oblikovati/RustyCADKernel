// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the dumps.

//! The dumps of the share-out, of the items, of the modifiers, of a selection, of the
//! model and of an entity.
use super::item_lists::h_text;
use super::params::info;
use super::statics::defined_line;
use super::{IFSelectWorkSession, write_out};
use crate::if_select::{IFSelectPrintCount, IFSelectSelection, IFSelectSelectionIterator};
use crate::interface::{
    InterfaceInterfaceModel, InterfaceModelEntities, InterfaceMsg, InterfaceShareFlags,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, StandardTransient, exceptions},
    tcollection::AsciiString,
};
use std::fmt::Write as _;
use std::io::Write;

/// The text of an `AsciiString` label.
fn label_text(label: AsciiString) -> String {
    String::from_utf8_lossy(label.as_bytes()).into_owned()
}

/// The check mode of a dump level above 2.
fn dump_check_mode(level: i32) -> IFSelectPrintCount {
    match level {
        5 | 8 => IFSelectPrintCount::CountByItem,
        6 | 9 => IFSelectPrintCount::ListByItem,
        7 | 10 => IFSelectPrintCount::EntitiesByItem,
        _ => IFSelectPrintCount::ItemsByEntity,
    }
}

/// The entities of `list` printed by the model (" " before each).
fn print_entities(
    out: &mut Vec<u8>,
    m: &dyn InterfaceInterfaceModel,
    list: &[Handle<StandardTransient>],
) {
    for ent in list {
        out.push(b' ');
        let _ = m.print(ent, out, 0);
    }
    out.push(b'\n');
}

impl IFSelectWorkSession {
    /// `DumpShare`: the file names, the dispatches and the count of modifiers of the
    /// share-out.
    pub fn dump_share(&self) -> Result<(), Failure> {
        let mut sout =
            String::from("        **********  Definition ShareOut (Complete)  **********\n");
        defined_line(
            &mut sout,
            "Prefix       : ",
            "Prefix       not Defined",
            &self.file_prefix()?,
        );
        defined_line(
            &mut sout,
            "Default Root : ",
            "Default Root not Defined",
            &self.default_file_root()?,
        );
        defined_line(
            &mut sout,
            "Extension    : ",
            "Extension    not defined",
            &self.file_extension()?,
        );
        let sho = self.share_out();
        let live = sho.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::DumpShare: no share-out")
        })?;
        let nb = live.nb_dispatches();
        let _ = writeln!(
            sout,
            "Nb Dispatches : {nb} (Last Run : {}) : ",
            live.last_run()
        );
        for i in 1..=nb {
            self.dump_dispatch(&mut sout, i, &live.dispatch(i)?)?;
        }
        let (nbm, nbf) = (live.nb_modifiers(true), live.nb_modifiers(false));
        if nbm > 0 {
            let _ = writeln!(
                sout,
                "  ***   {nbm} active Model Modifiers : see ListModifiers   ***"
            );
        }
        if nbf > 0 {
            let _ = writeln!(
                sout,
                "  ***   {nbf} active File  Modifiers : see ListModifiers   ***"
            );
        }
        if nbm + nbf == 0 {
            sout.push_str("  ***   No active Modifiers   ***\n");
        }
        info(&sout);
        Ok(())
    }
    /// The lines of the dispatch `i` in `DumpShare`.
    fn dump_dispatch(
        &self,
        sout: &mut String,
        i: i32,
        disp: &Handle<dyn crate::if_select::IFSelectDispatch>,
    ) -> Result<(), Failure> {
        let live = disp.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::DumpShare: null dispatch")
        })?;
        let _ = write!(sout, "Dispatch n0 {i}");
        if self.has_name(&disp.cast()) {
            let _ = write!(sout, "   Name:{}", h_text(&self.name(&disp.cast())));
        }
        let _ = writeln!(sout, "   Label:{}", label_text(live.label()));
        let sel = live.final_selection();
        match sel.get() {
            None => sout.push_str("   No Final Selection Defined\n"),
            Some(s) if self.has_name(&sel.cast()) => {
                let _ = writeln!(
                    sout,
                    "   Final Selection : Name:{}  Label:{}",
                    h_text(&self.name(&sel.cast())),
                    label_text(s.label())
                );
            }
            Some(s) => {
                let _ = writeln!(sout, "   Final Selection : {}", label_text(s.label()));
            }
        }
        if live.has_root_name() {
            let _ = writeln!(sout, "   File Root Name : {}", h_text(&live.root_name()));
        } else {
            sout.push_str("   No specific file root name (see Default Root)\n");
        }
        Ok(())
    }
    /// `ListItems(label = "")`: the items whose label contains `label`, with their name,
    /// type and label.
    pub fn list_items(&self, label: &str) {
        let mut sout = String::from("        **********  Items in Session  **********\n");
        for i in 1..=self.max_ident() {
            let var = self.state.read().theitems[i as usize - 1].0.clone();
            let lab = self.item_label(i);
            if lab.is_null() {
                continue; //  -> item supprime
            }
            let text = h_text(&lab);
            if !label.is_empty() && !text.contains(label) {
                continue;
            }
            let _ = write!(sout, "#{i}");
            if self.has_name(&var) {
                let _ = write!(sout, "\t- Named : {}\t- ", h_text(&self.name(&var)));
            } else {
                sout.push_str(" - (no name) - ");
            }
            let _ = write!(
                sout,
                "{}\n    {text}\n",
                var.dynamic_type().map_or("", |t| t.name())
            );
        }
        info(&sout);
    }
    /// `ListFinalModifiers(formodel)`: the model or file modifiers of the share-out.
    pub fn list_final_modifiers(&self, formodel: bool) -> Result<(), Failure> {
        let sho = self.share_out();
        let live = sho.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::ListFinalModifiers: no share-out")
        })?;
        let nb = live.nb_modifiers(formodel);
        let mut sout = format!(
            "        **********  Modifiers in Session {}: {nb}  **********\n",
            if formodel {
                "(For Model)"
            } else {
                "(For File)"
            }
        );
        for i in 1..=nb {
            let modif = live.general_modifier(formodel, i)?;
            if let Some(m) = modif.get() {
                let _ = write!(sout, "Modifier n0.{i}\t: {}", label_text(m.label()));
            }
            if self.has_name(&modif.cast()) {
                let _ = write!(sout, "\t Named as : {}", h_text(&self.name(&modif.cast())));
            }
            sout.push('\n');
        }
        info(&sout);
        Ok(())
    }
    /// `DumpSelection(sel)`: the name, the label and the inputs of a selection item.
    pub fn dump_selection(&self, sel: &Handle<dyn IFSelectSelection>) -> Result<(), Failure> {
        let Some(live) = sel.get().filter(|_| self.item_ident(&sel.cast()) != 0) else {
            info("Selection :  Unknown\n"); // sout<<Handle
            return Ok(());
        };
        let mut sout = String::from("        **********  Selection");
        if self.has_name(&sel.cast()) {
            let _ = write!(sout, " , Name : {}", h_text(&self.name(&sel.cast())));
        }
        let _ = write!(
            sout,
            "  **********\nLabel : {} . Input(s) : \n",
            label_text(live.label())
        );
        let mut iter = IFSelectSelectionIterator::new();
        live.fill_iterator(&mut iter);
        let mut nb = 0;
        while iter.more() {
            nb += 1;
            let newsel = iter.value()?;
            let _ = writeln!(
                sout,
                " -- {}",
                newsel
                    .get()
                    .map(|s| label_text(s.label()))
                    .unwrap_or_default()
            );
            iter.next();
        }
        let _ = writeln!(sout, " Nb Inputs:{nb}");
        info(&sout);
        Ok(())
    }
    /// `DumpModel(level, S)`: the loaded file, the header (level 0), the count of
    /// entities and roots, then the roots (1), all the entities (2) or the check list
    /// (above, by a print mode). The native branch of the levels 3 and 4 after the test
    /// `level > 2` is unreachable and not ported.
    pub fn dump_model(&self, level: i32, s: &mut dyn Write) -> Result<(), Failure> {
        if !self.is_loaded() {
            return write_out(s, b" ***  Data for List not available  ***\n");
        }
        let (model, loaded, hgraph) = {
            let st = self.state.read();
            (
                st.my_model.clone(),
                st.theloaded.clone(),
                st.thegraph.clone(),
            )
        };
        let m = model
            .get()
            .ok_or_else(|| exceptions::null_object("IFSelect_WorkSession::DumpModel: no model"))?;
        let mut out =
            b"\n        *****************************************************************\n"
                .to_vec();
        if loaded.is_empty() {
            out.extend_from_slice(b"        ********  No name for Loaded File\n");
        } else {
            let name = label_text(loaded.clone());
            let _ = writeln!(
                out,
                "        ********  Loaded File : {name}{} ********",
                InterfaceMsg::blanks(32 - loaded.length())
            );
        }
        if level == 0 {
            out.extend_from_slice(b"        ********  Short Dump of Header                           ********\n        *****************************************************************\n\n");
            let _ = m.dump_header(&mut out, 0);
            out.push(b'\n');
        }
        let shar = InterfaceShareFlags::from_graph(&hgraph.graph())?;
        let nbent = m.nb_entities();
        let mut nbr = 0;
        for i in 1..=nbent {
            nbr += i32::from(!shar.is_shared(&m.value(i)?)?);
        }
        let _ = write!(
            out,
            "        *****************************************************************\n        ********  Model : {nbent} Entities, of which {nbr} Root(s)\n        *****************************************************************\n\n"
        );
        if level <= 0 {
            return write_out(s, &out);
        }
        match level {
            1 => {
                out.extend_from_slice(b"        ********  Root Entities  ********      ");
                self.list_entities(&shar.root_entities(), 1, &mut out)?;
            }
            2 => {
                out.extend_from_slice(b"        ********  Complete List  ********      ");
                self.list_entities(&m.entities(), 1, &mut out)?;
            }
            _ => self.print_check_list(
                &mut out,
                &self.model_check_list(true)?,
                false,
                dump_check_mode(level),
            )?,
        }
        let _ = write!(
            out,
            "\nThere are {nbent} Entities, of which {nbr} Root(s)\n"
        );
        write_out(s, &out)
    }
    /// `TraceDumpModel(mode)`: `DumpModel` on the information messenger.
    pub fn trace_dump_model(&self, mode: i32) -> Result<(), Failure> {
        let mut out = Vec::new();
        self.dump_model(mode, &mut out)?;
        info(&String::from_utf8_lossy(&out));
        Ok(())
    }
    /// `DumpEntity(ent, level, S)`: the dump of an entity of the model by the library.
    pub fn dump_entity(
        &self,
        ent: &Handle<StandardTransient>,
        level: i32,
        s: &mut dyn Write,
    ) -> Result<(), Failure> {
        if !self.is_loaded() {
            return write_out(s, b" ***  Data for List not available  ***\n");
        }
        let (model, lib, protocol) = {
            let st = self.state.read();
            (
                st.my_model.clone(),
                st.thelibrary.clone(),
                st.theprotocol.clone(),
            )
        };
        let num = model.get().map_or(0, |m| m.number(ent));
        if num == 0 {
            return write_out(s, b" ***  Entity to Dump not in the Model  ***\n");
        }
        let Some(l) = lib.get() else {
            return write_out(s, b" ***  WorkLibrary not defined  ***\n");
        };
        write_out(
            s,
            format!("        ********  Dumping Entity n0 {num} level:{level}  ********\n")
                .as_bytes(),
        )?;
        l.dump_entity(&model, &protocol, ent, s, level)
    }
    /// `TraceDumpEntity(ent, level)`: `DumpEntity` on the information messenger.
    pub fn trace_dump_entity(
        &self,
        ent: &Handle<StandardTransient>,
        level: i32,
    ) -> Result<(), Failure> {
        let mut out = Vec::new();
        self.dump_entity(ent, level, &mut out)?;
        info(&String::from_utf8_lossy(&out));
        Ok(())
    }
    /// `PrintEntityStatus(ent, S)`: the number, name, type, category, validity, check,
    /// sharing and shared entities of an entity of the model.
    pub fn print_entity_status(
        &self,
        ent: &Handle<StandardTransient>,
        s: &mut dyn Write,
    ) -> Result<(), Failure> {
        if self.starting_number(ent) == 0 {
            println!(" --  PrintEntityStatus : unknown");
            return Ok(());
        }
        let model = self.model();
        let m = model.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::PrintEntityStatus: no model")
        })?;
        let mut out = b"  Ent. n0/id:   ".to_vec();
        let _ = m.print(ent, &mut out, 0);
        let hname = h_text(&self.entity_name(ent)?);
        if !hname.is_empty() {
            let _ = write!(out, "\tName:{hname}");
        }
        out.push(b'\n');
        let signtype = self.sign_type();
        match signtype.get() {
            None => {
                let _ = writeln!(
                    out,
                    "  Type(CDL):{}",
                    ent.dynamic_type().map_or("", |t| t.name())
                );
            }
            Some(sign) => {
                let _ = writeln!(out, "  Type:{}", sign.value(ent, &model));
            }
        }
        let _ = writeln!(
            out,
            "    Category : {}    Validity : {}",
            self.category_name(ent),
            self.validity_name(ent)
        );
        let _ = self
            .check_one(ent, true)?
            .print_with_model(&mut out, &model, false, 0);
        print_relatives(
            &mut out,
            m,
            self.sharings(ent)?,
            ["  Root", "  Super-entities:", " : (n0/id):"],
        );
        print_relatives(
            &mut out,
            m,
            self.shareds(ent)?,
            ["  No sub-entity", "  Sub-entities:", " , i.e. (n0/id):"],
        );
        write_out(s, &out)
    }
}

/// The sharing or shared entities of `PrintEntityStatus`: `none` without list or
/// entity, else the count between `head` and `tail`, then the entities.
fn print_relatives(
    out: &mut Vec<u8>,
    m: &dyn InterfaceInterfaceModel,
    list: Option<Vec<Handle<StandardTransient>>>,
    [none, head, tail]: [&str; 3],
) {
    let Some(list) = list else {
        let _ = writeln!(out, "{none}");
        return;
    };
    if list.is_empty() {
        out.extend_from_slice(none.as_bytes());
    } else {
        let _ = write!(out, "{head}{}{tail}", list.len());
    }
    print_entities(out, m, &list);
}
