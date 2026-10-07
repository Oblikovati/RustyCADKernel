// Rust port of DataExchange/TKXSBase/XSControl/XSControl_TransferReader (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the statistics.

//! The statistics of [`XSControlTransferReader`]: on the final results, and on the
//! records of a transient process (by root, all, abnormal, checks).
use super::XSControlTransferReader;
use crate::if_select::{
    IFSelectCheckCounter, IFSelectPrintCount, IFSelectSignatureList, IFSelectSignatureListOps,
};
use crate::interface::{
    InterfaceCheckStatus, InterfaceInterfaceModel, InterfaceModelEntities, InterfaceMsg,
};
use crate::transfer::{
    TransferBinder, TransferIteratorOfProcess, TransferTransientKind, TransferTransientProcess,
};
use rustycad_tkernel::{
    message::Messenger,
    standard::{Failure, Handle, StandardTransient, exceptions},
};
use std::fmt::Write as _;
use std::io::Write;

/// `BinderStatus(binder, mess)`: the status of a binder (0 no data, 1/2/3 no result
/// ok/warning/fail, 11/12/13 with a result) and its message.
fn binder_status(binder: &Handle<dyn TransferBinder>) -> (i32, String) {
    let Some(live) = binder.get() else {
        return (0, String::from("(no data recorded)"));
    };
    let has = live.has_result();
    let name = live.result_type_name();
    match live.check().status() {
        InterfaceCheckStatus::CheckOK if has => (11, name),
        InterfaceCheckStatus::CheckOK => (1, String::from("(no result)")),
        InterfaceCheckStatus::CheckWarning if has => (12, format!("{name}  (+ warning)")),
        InterfaceCheckStatus::CheckWarning => (2, String::from("(warning)")),
        InterfaceCheckStatus::CheckFail if has => (13, format!("{name}  (+ FAIL)")),
        InterfaceCheckStatus::CheckFail => (3, String::from("(FAIL)")),
        _ => (0, String::new()),
    }
}

/// `PrintPercent(sout, mess, nb, nl)`: the percentage of `nb` among `nl` (nothing for
/// none), sent at once.
fn print_percent(messenger: &Messenger, mess: &str, nb: i32, nl: i32) {
    if nb <= 0 || nl == 0 {
        return;
    }
    let mut sender = messenger.send_info();
    let _ = write!(sender, "******      {mess}: ");
    let percent = nb * 100 / nl;
    let _ = match () {
        _ if nb == nl => writeln!(sender, "100 %"),
        _ if percent == 0 => writeln!(sender, "< 1 %"),
        _ => writeln!(
            sender,
            "{}{percent} %",
            if percent < 10 { "  " } else { " " }
        ),
    };
}

/// The titles of `PrintStatsOnList` for `what` and `mode`.
fn stats_titles(what: i32, mode: i32) -> String {
    let mut out = String::from(
        "\n*******************************************************************\n******        Statistics on Transfer Process (Read)          ******\n",
    );
    out.push_str(match what {
        1 => "******        Individual Transfers  (Roots)                  ******\n",
        2 => "******        All recorded data about Transfer               ******\n",
        3 => "******        Abnormal records                               ******\n",
        4 => "******        Check messages                                 ******\n",
        5 => "******        Fail  messages                                 ******\n",
        _ => "",
    });
    if (1..=3).contains(&what) {
        out.push_str(match mode {
            0 => "******        (n0s of recorded entities)                     ******\n",
            1 => "******        (per entity : type + result)                   ******\n",
            2 => "******        (per entity : type + result/status)            ******\n",
            3 => "******        (count per type of entity)                     ******\n",
            4 => "******        (count per type of result)                     ******\n",
            5 => "******   (count per couple entity-type / result-type/status) ******\n",
            6 => "******   (list per couple entity-type / result-type/status)  ******\n",
            _ => "",
        });
    }
    out.push_str("*******************************************************************\n");
    out
}

/// The counts of the binder statuses of `PrintStatsOnList`.
#[derive(Default)]
struct StatusCounts {
    nbw: i32,
    nbf: i32,
    nbr: i32,
    nbrw: i32,
    nbrf: i32,
    nbnr: i32,
    nbi: i32,
}

impl StatusCounts {
    fn count(&mut self, stat: i32) {
        match stat {
            2 => self.nbw += 1,
            3 => self.nbf += 1,
            11 => self.nbr += 1,
            12 => self.nbrw += 1,
            13 => self.nbrf += 1,
            _ => {}
        }
    }
}

/// The entity and its type, as the per entity lines print them.
fn entity_text(model: &dyn InterfaceInterfaceModel, ent: &Handle<StandardTransient>) -> String {
    let mut printed = Vec::new();
    let _ = model.print(ent, &mut printed, 0);
    format!(
        "{}   {}",
        String::from_utf8_lossy(&printed),
        model.type_name(ent, false)
    )
}

/// The messages of a check, as the per entity lines (mode 2) print them.
fn check_lines(out: &mut String, binder: &Handle<dyn TransferBinder>) -> Result<(), Failure> {
    let live = binder.get().ok_or_else(|| {
        exceptions::null_object("XSControl_TransferReader::PrintStatsOnList: null binder")
    })?;
    let ch = live.check();
    let (nbw, nbf) = (ch.nb_warnings(), ch.nb_fails());
    if nbw > 0 {
        let _ = writeln!(out, " - Warnings : {nbw}:");
        for i in 1..=nbw {
            let _ = writeln!(
                out,
                "{}",
                String::from_utf8_lossy(ch.c_warning(i, true)?.as_bytes())
            );
        }
    }
    if nbf > 0 {
        let _ = writeln!(out, " - Fails : {nbf}:");
        for i in 1..=nbf {
            let _ = writeln!(
                out,
                "{}",
                String::from_utf8_lossy(ch.c_fail(i, true)?.as_bytes())
            );
        }
    }
    Ok(())
}

impl XSControlTransferReader {
    /// `PrintStats(theStream, theWhat, theMode = 0)`: the statistics of the last transfer
    /// (`what` below 10, through the process messenger) and of the final results (by
    /// numbers, entities or counts per type by mode).
    pub fn print_stats(&self, sout: &mut dyn Write, what: i32, mode: i32) -> Result<(), Failure> {
        let mut out = String::from(
            "\n*******************************************************************\n******        Statistics on Transfer (Read)                  ******\n\n*******************************************************************\n",
        );
        if what > 10 {
            out.push_str(" ***  Not yet implemented\n");
            return sout.write_all(out.as_bytes()).map_err(io_failure);
        }
        if what < 10 {
            out.push_str("******        Data recorded on Last Transfer                 ******\n");
            Self::print_stats_process(&self.transient_process(), what, mode)?;
        }
        out.push_str("******        Final Results                                  ******\n");
        let model = self.model();
        let Some(m) = model.get() else {
            out.push_str("****    Model unknown\n");
            return sout.write_all(out.as_bytes()).map_err(io_failure);
        };
        self.print_final_results(&mut out, m, &model, mode)?;
        sout.write_all(out.as_bytes()).map_err(io_failure)
    }
    /// The final results of `PrintStats`.
    fn print_final_results(
        &self,
        out: &mut String,
        m: &dyn InterfaceInterfaceModel,
        model: &Handle<dyn InterfaceInterfaceModel>,
        mode: i32,
    ) -> Result<(), Failure> {
        let list = self.recorded_list();
        let counter = (mode > 2).then(|| IFSelectSignatureList::new(mode == 6));
        let pcm = if mode == 6 {
            IFSelectPrintCount::ListByItem
        } else {
            IFSelectPrintCount::CountByItem
        };
        let _ = write!(
            out,
            "****    Nb Recorded : {} : entities n0s : ",
            list.len()
        );
        for (i, ent) in list.iter().enumerate() {
            match mode {
                0 => {
                    let _ = write!(out, "  {}", m.number(ent));
                }
                1 | 2 => {
                    let mut printed = Vec::new();
                    let _ = m.print(ent, &mut printed, 0);
                    let _ = write!(
                        out,
                        "[ {} ]:{}  Type:{}",
                        InterfaceMsg::blanks_for_integer(i as i32 + 1, 6),
                        String::from_utf8_lossy(&printed),
                        m.type_name(ent, false)
                    );
                }
                3..=6 => {
                    if let Some(c) = &counter {
                        c.add(ent, &m.type_name(ent, false));
                    }
                }
                _ => {}
            }
        }
        if let Some(c) = &counter {
            let mut printed = Vec::new();
            c.print_list(&mut printed, model, pcm)?;
            out.push_str(&String::from_utf8_lossy(&printed));
        }
        out.push('\n');
        Ok(())
    }
    /// `PrintStatsProcess(theTP, theWhat, theMode = 0)`: `PrintStatsOnList` without list.
    pub fn print_stats_process(
        tp: &Handle<TransferTransientProcess>,
        what: i32,
        mode: i32,
    ) -> Result<(), Failure> {
        Self::print_stats_on_list(tp, None, what, mode)
    }
    /// `PrintStatsOnList(theTP, theList, theWhat, theMode = 0)`: the statistics of the
    /// process (0), of its roots (1), all its records (2), its abnormal records (3) (by
    /// numbers, entities with results, or counts by mode, then the percentages), or its
    /// checks (4) and fails (5); on the entities of `list` when given.
    /// `Standard_NullObject` for a null process. Native quirk kept: the per entity lines
    /// with a binder are numbered 0.
    pub fn print_stats_on_list(
        tp: &Handle<TransferTransientProcess>,
        list: Option<&[Handle<StandardTransient>]>,
        what: i32,
        mode: i32,
    ) -> Result<(), Failure> {
        let process = tp.get().ok_or_else(|| {
            exceptions::null_object(
                "XSControl_TransferReader::PrintStatsOnList: null transient process",
            )
        })?;
        let messenger = process.messenger();
        let mut sout = messenger.send_info();
        if what == 0 {
            let mut printed = Vec::new();
            let _ = process.print_stats(0, &mut printed);
            let _ = sout.write_str(&String::from_utf8_lossy(&printed));
            return Ok(());
        }
        let mut out = stats_titles(what, mode);
        let model = process.model();
        if (1..=3).contains(&what) {
            let itrp = match what {
                1 => process.root_result(true),
                2 => process.complete_result(true),
                _ => process.abnormal_result(),
            };
            print_records(&mut out, &messenger, &model, itrp, list, mode)?;
        } else if what == 4 || what == 5 {
            print_checks(&mut out, process, &model, what, mode)?;
        }
        let _ = sout.write_str(&out);
        Ok(())
    }
}

/// The records of `PrintStatsOnList` (what 1 to 3).
fn print_records(
    out: &mut String,
    messenger: &Messenger,
    model: &Handle<dyn InterfaceInterfaceModel>,
    mut itrp: TransferIteratorOfProcess<TransferTransientKind>,
    list: Option<&[Handle<StandardTransient>]>,
    mode: i32,
) -> Result<(), Failure> {
    let m = model.get().ok_or_else(|| {
        exceptions::null_object("XSControl_TransferReader::PrintStatsOnList: no model")
    })?;
    let nb = itrp.number();
    if list.is_some() {
        itrp.filter(list, true);
    }
    let nl = itrp.number(); // after filtering
    let counter = (mode > 2).then(|| IFSelectSignatureList::new(mode == 6));
    let notrec = list.is_some() && mode > 2; // note the "no record"
    let pcm = if mode == 6 {
        IFSelectPrintCount::ListByItem
    } else {
        IFSelectPrintCount::CountByItem
    };
    let _ = writeln!(
        out,
        "****        Entities in Model   : {}\n****        Nb Items (Transfer) : {nb}",
        m.nb_entities()
    );
    if list.is_some() {
        let _ = writeln!(out, "****        Nb Items (Listed)   : {nl}");
    }
    let mut counts = StatusCounts::default();
    itrp.start();
    while itrp.more() {
        counts.nbi += 1;
        let (binder, ent) = (itrp.value()?, itrp.starting()?);
        itrp.next();
        if binder.is_null() {
            counts.nbnr += 1;
            if notrec {
                if let Some(c) = &counter {
                    c.add(&ent, "(not recorded)");
                }
            } else if mode == 1 || mode == 2 {
                let _ = writeln!(
                    out,
                    "[{}{} ]:{}  (not recorded)",
                    InterfaceMsg::blanks_for_integer(counts.nbi, 4),
                    counts.nbi,
                    entity_text(m, &ent)
                );
                continue;
            }
        }
        if mode == 0 {
            let _ = write!(out, "  {}", m.number(&ent));
            continue;
        }
        let (stat, mess) = binder_status(&binder);
        if mode != 3 {
            counts.count(stat);
        }
        if mode == 1 || mode == 2 {
            let _ = writeln!(
                out,
                "[{}0 ]:{}\tResult:{mess}",
                InterfaceMsg::blanks_for_integer(0, 4),
                entity_text(m, &ent)
            );
            if mode == 2 {
                check_lines(out, &binder)?;
            }
            continue;
        }
        if let Some(c) = counter.as_ref().filter(|_| (3..=6).contains(&mode)) {
            let tname = m.type_name(&ent, false);
            match mode {
                3 => c.add(&ent, &tname),
                4 => c.add(&ent, &mess),
                _ => c.add(&ent, &format!("{tname}\t-> {mess}")),
            }
        }
    }
    match &counter {
        Some(c) => {
            let mut printed = Vec::new();
            c.print_list(&mut printed, model, pcm)?;
            out.push_str(&String::from_utf8_lossy(&printed));
        }
        None => out.push('\n'),
    }
    if mode != 3 && counts.nbi > 0 {
        out.push_str("******        Percentages according Transfer Status          ******\n");
        print_percent(messenger, "Result          ", counts.nbr + counts.nbrw, nl);
        print_percent(messenger, "Result + FAIL   ", counts.nbrf, nl);
        print_percent(messenger, "FAIL, no Result ", counts.nbf, nl);
        print_percent(messenger, "Just Warning    ", counts.nbw, nl);
        print_percent(messenger, "Nothing Recorded", counts.nbnr, nl);
    }
    Ok(())
}

/// The checks of `PrintStatsOnList` (what 4 and 5): listed by entity (mode 0) or counted
/// by message.
fn print_checks(
    out: &mut String,
    process: &TransferTransientProcess,
    model: &Handle<dyn InterfaceInterfaceModel>,
    what: i32,
    mode: i32,
) -> Result<(), Failure> {
    let mut chl = process.check_list(false);
    chl.set_name("** TRANSFER READ CHECK **");
    let mut printed = Vec::new();
    if mode == 0 {
        let _ = chl.print_with_model(&mut printed, model, what == 5, 0);
    } else {
        let pcm = if mode == 2 {
            IFSelectPrintCount::ListByItem
        } else {
            IFSelectPrintCount::CountByItem
        };
        let counter = IFSelectCheckCounter::new(true);
        counter.analyse(&chl, model, true, what == 5)?;
        counter.print_list(&mut printed, model, pcm)?;
    }
    out.push_str(&String::from_utf8_lossy(&printed));
    Ok(())
}

/// An output error as a `Standard_Failure`.
fn io_failure(e: std::io::Error) -> Failure {
    Failure::new(&format!(
        "XSControl_TransferReader::PrintStats: output failed: {e}"
    ))
}
