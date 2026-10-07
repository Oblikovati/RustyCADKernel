// Rust port of DataExchange/TKXSBase/Interface/Interface_InterfaceModel (.hxx, .cxx: the
// virtual members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_InterfaceModel`: the set of entities of a file, with its header, its
//! checks and its reports. The non-virtual members are the extension traits
//! [`InterfaceModelEntities`](super::InterfaceModelEntities) and
//! [`InterfaceModelReports`](super::InterfaceModelReports).
use super::{
    InterfaceCheck, InterfaceModelBase, InterfaceModelEntities, InterfaceProtocol,
    InterfaceReportEntity,
};
use crate::c_text::atoi;
use rustycad_tkernel::{
    impl_transient_interface,
    ncollection::DataMap,
    standard::{Failure, Handle, StandardTransient, Transient, exceptions},
    tcollection::HAsciiString,
};
use std::io::{self, Write};

/// `Interface_InterfaceModel`, abstract. The pure virtual members are required methods,
/// the virtual ones provided methods with the native body; the private fields are the
/// embedded [`InterfaceModelBase`].
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceInterfaceModel, InterfaceModelEntities};
/// fn count(model: &dyn InterfaceInterfaceModel) -> i32 { model.nb_entities() }
/// ```
pub trait InterfaceInterfaceModel: Transient {
    /// The embedded private fields.
    fn model_base(&self) -> &InterfaceModelBase;
    /// `Protocol`: the protocol of the GTool (null without GTool).
    fn protocol(&self) -> Handle<dyn InterfaceProtocol> {
        let gtool = self.gtool();
        gtool.get().map(|g| g.protocol()).unwrap_or_default()
    }
    /// `Clear`: entities, checks, header, labels and categories.
    fn clear(&self) {
        self.clear_entities();
        clear_global_checks(self.model_base());
        self.clear_header();
        self.clear_labels();
        self.model_base().write().thecategory.nullify();
    }
    /// `ClearEntities`: the entities and their reports (and the GTool cache).
    fn clear_entities(&self) {
        self.model_base().clear_entities_base();
    }
    /// `ClearLabels`.
    fn clear_labels(&self);
    /// `ClearHeader`.
    fn clear_header(&self);
    /// `Reservate(nbent)`: room for `nbent` entities (for `-nbent` reports when negative).
    fn reservate(&self, nbent: i32) {
        let mut state = self.model_base().write();
        if nbent > 0 && nbent as usize > state.theentities.bucket_count() {
            state.theentities.resize(nbent as usize);
        }
        if nbent < 0 && nbent.unsigned_abs() as usize > state.thereports.bucket_count() {
            state.thereports.resize(nbent.unsigned_abs() as usize);
        }
    }
    /// `AddEntity(anentity)`: adds an entity; a report entity adds its concerned entity
    /// and is recorded as its report. A null entity (dereferenced natively) is a
    /// `Standard_NullObject` failure.
    fn add_entity(&self, anentity: &Handle<StandardTransient>) -> Result<(), Failure> {
        if anentity.is_null() {
            return Err(exceptions::null_object(
                "Interface_InterfaceModel::AddEntity: null entity; expected a live entity",
            ));
        }
        let rep = anentity.cast::<InterfaceReportEntity>();
        if rep.is_null() {
            self.model_base().write().theentities.add(anentity.clone());
            return Ok(());
        }
        let concerned = rep.concerned();
        self.add_entity(&concerned)?;
        let number = self.number(&concerned);
        let mut state = self.model_base().write();
        grow_reports(&mut state.thereports);
        state.thereports.bind(number, anentity.clone());
        Ok(())
    }
    /// `GetFromAnother(other)`: takes the header (and what the norm needs) of `other`.
    fn get_from_another(&self, other: &Handle<dyn InterfaceInterfaceModel>) -> Result<(), Failure>;
    /// `NewEmptyModel`: an empty model of the same type.
    fn new_empty_model(&self) -> Handle<dyn InterfaceInterfaceModel>;
    /// `VerifyCheck(ach)`: nothing by default.
    fn verify_check(&self, _ach: &mut Handle<InterfaceCheck>) {}
    /// `DumpHeader(S, level = 0)`.
    fn dump_header(&self, s: &mut dyn Write, level: i32) -> io::Result<()>;
    /// `PrintLabel(ent, S)`.
    fn print_label(&self, ent: &Handle<StandardTransient>, s: &mut dyn Write) -> io::Result<()>;
    /// `PrintToLog(ent, S)`: by default `PrintLabel`.
    fn print_to_log(&self, ent: &Handle<StandardTransient>, s: &mut dyn Write) -> io::Result<()> {
        self.print_label(ent, s)
    }
    /// `StringLabel(ent)`.
    fn string_label(&self, ent: &Handle<StandardTransient>) -> Handle<HAsciiString>;
    /// `NextNumberForLabel(label, lastnum = 0, exact = true)`: the next entity after
    /// `lastnum` whose label is `label` (case insensitive), or ends with it when not
    /// exact; when not exact and not found, a label that is a number in range is taken
    /// as the entity number.
    fn next_number_for_label(&self, label: &str, lastnum: i32, exact: bool) -> i32 {
        let n = self.nb_entities();
        let labs = Handle::new(HAsciiString::new(label));
        let lnb = labs.length();
        labs.lower_case();
        for i in lastnum + 1..=n {
            let lab = self
                .value(i)
                .map(|ent| self.string_label(&ent))
                .unwrap_or_default();
            if label_matches(&lab, &labs, lnb, exact) {
                return i;
            }
        }
        if exact {
            return 0;
        }
        let i = if labs.is_integer_value() {
            atoi(labs.string().as_bytes())
        } else {
            0
        };
        if i <= 0 || i > n { 0 } else { i }
    }
}
impl_transient_interface!(
    dyn InterfaceInterfaceModel,
    "Interface_InterfaceModel",
    StandardTransient
);

/// The label test of `NextNumberForLabel` (the label is lowered in place when not exact,
/// as natively).
fn label_matches(
    lab: &Handle<HAsciiString>,
    labs: &Handle<HAsciiString>,
    lnb: i32,
    exact: bool,
) -> bool {
    let Some(text) = lab.get() else { return false };
    if exact {
        return text.is_same_string_with_case(labs, false).unwrap_or(false);
    }
    if text.length() < lnb {
        return false;
    }
    text.lower_case();
    text.search_from_end_string(labs) == text.length() - lnb + 1
}

/// Clears the two global checks of a model.
pub(super) fn clear_global_checks(base: &InterfaceModelBase) {
    let (syntactic, semantic) = {
        let state = base.read();
        (state.thecheckstx.clone(), state.thechecksem.clone())
    };
    for check in [syntactic, semantic] {
        if let Some(live) = check.get() {
            live.clear();
        }
    }
}

/// The growth of the report map before a new report: 1.5 times the buckets when nearly
/// full, as natively.
pub(super) fn grow_reports(reports: &mut DataMap<i32, Handle<StandardTransient>>) {
    let max_rep = reports.bucket_count();
    if max_rep <= reports.len() + 10 {
        reports.resize(max_rep * 3 / 2);
    }
}
