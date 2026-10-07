// Rust port of DataExchange/TKXSBase/XSControl/XSControl_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: PrintTransferStatus.

//! `PrintTransferStatus` of [`XSControlWorkSession`]: one item of the writing (finder
//! process) or of the reading (transient process), with its checks.
use super::XSControlWorkSession;
use super::work_session::{live_reader, live_writer};
use crate::interface::InterfaceModelEntities;
use crate::transfer::{TransferBinder, TransferFinderProcess, TransferTransientProcess};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, exceptions};
use std::io::Write;

/// The index of an item and of its root: `num` > 0 an item, `num` < 0 a root (`None`
/// when 0 or out of range).
fn item_indices(num: i32, max: i32, maxr: i32) -> Option<(i32, i32)> {
    match num {
        0 => None,
        n if n > 0 => (n <= max).then_some((n, 0)),
        n => (-n <= maxr).then_some((0, -n)),
    }
}

/// The header line of an item; native quirk kept: the root line repeats the item number.
fn item_header(kind: &str, ne: i32, max: i32, nr: i32) -> String {
    let mut out = format!("Transfer {kind} item n0.{ne} of {max}");
    if nr > 0 {
        out.push_str(&format!("  ** Transfer Root n0.{ne}"));
    }
    out.push('\n');
    out
}

/// The name of the type of an object, `Standard_NullObject` for null (a native null
/// dereference).
fn type_name(obj: &Handle<StandardTransient>) -> Result<&'static str, Failure> {
    obj.dynamic_type().map(|t| t.name()).ok_or_else(|| {
        exceptions::null_object("XSControl_WorkSession::PrintTransferStatus: null item")
    })
}

/// The warnings and fails of a binder (nothing for null).
fn binder_checks(binder: &Handle<dyn TransferBinder>) -> Result<String, Failure> {
    let Some(live) = binder.get() else {
        return Ok(String::new());
    };
    let ch = live.check();
    let (nbw, nbf) = (ch.nb_warnings(), ch.nb_fails());
    let mut out = String::new();
    if nbw > 0 {
        out.push_str(&format!(" - Warnings : {nbw} :\n"));
        for i in 1..=nbw {
            out.push_str(&format!(
                "{}\n",
                String::from_utf8_lossy(ch.c_warning(i, true)?.as_bytes())
            ));
        }
    }
    if nbf > 0 {
        out.push_str(&format!(" - Fails : {nbf} :\n"));
        for i in 1..=nbf {
            out.push_str(&format!(
                "{}\n",
                String::from_utf8_lossy(ch.c_fail(i, true)?.as_bytes())
            ));
        }
    }
    Ok(out)
}

impl XSControlWorkSession {
    /// `PrintTransferStatus(theNum, theWri, theS)`: the item `num` (> 0) or the root
    /// `-num` of the writing (`wri`) or of the reading, its type, result and checks;
    /// false for 0, out of range or no process. Output errors are ignored (as natively).
    ///
    /// ```
    /// use rustycad_tkxsbase::xs_control::XSControlWorkSession;
    /// let ws = XSControlWorkSession::new();
    /// let mut out = Vec::new();
    /// assert!(!ws.print_transfer_status(1, true, &mut out).unwrap());
    /// ```
    pub fn print_transfer_status(
        &self,
        num: i32,
        wri: bool,
        s: &mut dyn Write,
    ) -> Result<bool, Failure> {
        if wri {
            let fp = live_writer(&self.transfer_writer())?.finder_process();
            return self.print_write_status(&fp, num, s);
        }
        let tp = live_reader(&self.transfer_reader())?.transient_process();
        self.print_read_status(&tp, num, s)
    }
    /// The writing branch: the finder, its transient result in the model; native quirk
    /// kept: the trace is started with a null binder, so no check is printed.
    fn print_write_status(
        &self,
        fp: &Handle<TransferFinderProcess>,
        num: i32,
        s: &mut dyn Write,
    ) -> Result<bool, Failure> {
        let Some(process) = fp.get() else {
            return Ok(false);
        };
        let (max, maxr) = (process.nb_mapped(), process.nb_roots());
        let Some((ne, nr)) = item_indices(num, max, maxr) else {
            return Ok(false);
        };
        let (ne, nr, finder) = if nr == 0 {
            let finder = process.mapped(ne)?;
            (ne, process.root_index(&finder), finder)
        } else {
            let finder = process.root(nr)?;
            (process.map_index(&finder), nr, finder)
        };
        let ent = process.find_transient(&finder);
        let _ = writeln!(
            s,
            "{} -> Type {}",
            item_header("Write", ne, max, nr),
            type_name(&finder.cast())?
        );
        let binder: Handle<dyn TransferBinder> = Handle::default();
        process.start_trace(&binder, &finder, 0, 0);
        self.print_write_result(&ent, s)?;
        let _ = s.write_all(binder_checks(&binder)?.as_bytes());
        Ok(true)
    }
    /// The transient result of a writing, with its entity in the model of the session.
    fn print_write_result(
        &self,
        ent: &Handle<StandardTransient>,
        s: &mut dyn Write,
    ) -> Result<(), Failure> {
        if ent.is_null() {
            return Ok(());
        }
        let _ = write!(s, " ** Transient Result, type {}", type_name(ent)?);
        if let Some(model) = self.model().get() {
            let _ = s.write_all(b" In output Model, Entity ");
            let _ = model.print(ent, s, 0);
        }
        let _ = s.write_all(b"\n");
        Ok(())
    }
    /// The reading branch: the entity in the model of the process, its binder traced and
    /// its checks; native quirks kept: the model notes go to the standard output, and the
    /// root index of an item is looked up for a null finder (so 0).
    fn print_read_status(
        &self,
        tp: &Handle<TransferTransientProcess>,
        num: i32,
        s: &mut dyn Write,
    ) -> Result<bool, Failure> {
        let Some(process) = tp.get() else {
            return Ok(false);
        };
        let model = process.model();
        if model.is_null() {
            println!("No Model");
        } else if model != self.model() {
            println!("Model different from the session");
        }
        let (max, maxr) = (process.nb_mapped(), process.nb_roots());
        let Some((ne, nr)) = item_indices(num, max, maxr) else {
            return Ok(false);
        };
        let (ne, nr, ent) = if nr == 0 {
            (
                ne,
                process.root_index(&Handle::default()),
                process.mapped(ne)?,
            )
        } else {
            let ent = process.root(nr)?;
            (process.map_index(&ent), nr, ent)
        };
        let _ = s.write_all(item_header("Read", ne, max, nr).as_bytes());
        if let Some(m) = model.get() {
            let _ = s.write_all(b" In Model, Entity ");
            let _ = m.print(&ent, s, 0);
        }
        let binder = process.map_item(ne)?;
        let _ = s.write_all(b"\n");
        process.start_trace(&binder, &ent, 0, 0);
        let _ = s.write_all(binder_checks(&binder)?.as_bytes());
        Ok(true)
    }
}
