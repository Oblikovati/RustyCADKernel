// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_SignatureList (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_SignatureList`: the counts of the signatures of entities, with optionally
//! the list of the entities of each signature.
use super::IFSelectPrintCount;
use crate::interface::{InterfaceInterfaceModel, InterfaceModelEntities, InterfaceMsg};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    ncollection::IndexedDataMap,
    standard::{Failure, Handle, SharedState, StandardTransient, Transient, exceptions},
    tcollection::{AsciiString, HAsciiString},
};
use std::io::{self, Write};

/// The counts by signature (in the order of the first occurrence).
pub type IFSelectSignatureCounts = IndexedDataMap<AsciiString, i32>;
/// The entities by signature.
pub type IFSelectSignatureEntities = IndexedDataMap<AsciiString, Vec<Handle<StandardTransient>>>;

/// The counts, the entity lists (copies) and the count of nulls of a list.
type ListSnapshot = (
    Vec<(String, i32)>,
    Vec<(String, Vec<Handle<StandardTransient>>)>,
    i32,
);

/// The fields of `IFSelect_SignatureList`.
struct ListState {
    thesignonly: bool,
    thelistat: bool,
    thenbnuls: i32,
    thename: AsciiString,
    thelastval: AsciiString,
    thedicount: IFSelectSignatureCounts,
    thediclist: IFSelectSignatureEntities,
}

/// `IFSelect_SignatureList` (concrete, embedded by the counters); its virtual members
/// are [`IFSelectSignatureListOps`].
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectSignatureList;
/// use rustycad_tkernel::standard::Handle;
/// let list = IFSelectSignatureList::new(false);
/// list.add(&Handle::default(), "A");
/// list.add(&Handle::default(), "");
/// assert_eq!((list.nb_times("A"), list.nb_nulls()), (1, 1));
/// ```
pub struct IFSelectSignatureList {
    transient: StandardTransient,
    state: SharedState<ListState>,
}
impl_transient!(IFSelectSignatureList, "IFSelect_SignatureList", StandardTransient, transient, interfaces: [dyn IFSelectSignatureListOps]);

impl Default for IFSelectSignatureList {
    fn default() -> Self {
        Self::new(false)
    }
}

impl IFSelectSignatureList {
    /// `IFSelect_SignatureList(withlist = false)`: named "...".
    pub fn new(withlist: bool) -> Self {
        let state = ListState {
            thesignonly: false,
            thelistat: withlist,
            thenbnuls: 0,
            thename: AsciiString::new("..."),
            thelastval: AsciiString::default(),
            thedicount: IFSelectSignatureCounts::default(),
            thediclist: IFSelectSignatureEntities::default(),
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// `SetList(withlist)`.
    pub fn set_list(&self, withlist: bool) {
        self.state.write().thelistat = withlist;
    }
    /// `ModeSignOnly`: whether `Add` only records the last value.
    pub fn mode_sign_only(&self) -> bool {
        self.state.read().thesignonly
    }
    /// `ModeSignOnly() = mode` (the native modifiable reference).
    pub fn set_mode_sign_only(&self, mode: bool) {
        self.state.write().thesignonly = mode;
    }
    /// The base `Clear`: no count, no list, no last value.
    pub fn clear_base(&self) {
        let mut st = self.state.write();
        st.thelastval = AsciiString::default();
        st.thenbnuls = 0;
        st.thedicount.clear(true);
        st.thediclist.clear(true);
    }
    /// `Add(ent, sign)`: counts `sign` (an empty one as a null), with `ent` in its list.
    pub fn add(&self, ent: &Handle<StandardTransient>, sign: &str) {
        let mut st = self.state.write();
        if st.thesignonly {
            st.thelastval = AsciiString::new(sign);
            return;
        }
        if sign.is_empty() {
            st.thenbnuls += 1;
            return;
        }
        let key = AsciiString::new(sign);
        match st.thedicount.change_seek(&key) {
            Some(count) => *count += 1,
            None => {
                st.thedicount.add(key.clone(), 1);
            }
        }
        if st.thelistat {
            match st.thediclist.change_seek(&key) {
                Some(list) => list.push(ent.clone()),
                None => {
                    st.thediclist.add(key, vec![ent.clone()]);
                }
            }
        }
    }
    /// `LastValue`.
    pub fn last_value(&self) -> AsciiString {
        self.state.read().thelastval.clone()
    }
    /// `Init(name, count, list, nbnuls)`: the counts and lists given (no list when it is
    /// empty).
    pub fn init(
        &self,
        name: &str,
        count: &IFSelectSignatureCounts,
        list: &IFSelectSignatureEntities,
        nbnuls: i32,
    ) {
        let mut st = self.state.write();
        st.thelastval = AsciiString::default();
        st.thename = AsciiString::new(name);
        st.thedicount = count.clone();
        st.thediclist = list.clone();
        st.thenbnuls = nbnuls;
        if st.thediclist.is_empty() {
            st.thelistat = false;
        }
    }
    /// `List(root = "")`: the signatures starting with `root`.
    pub fn list(&self, root: &str) -> Vec<Handle<HAsciiString>> {
        let st = self.state.read();
        (1..=st.thedicount.extent())
            .filter_map(|i| st.thedicount.find_key(i).ok())
            .filter(|key| key.starts_with(root.as_bytes()))
            .map(|key| Handle::new(HAsciiString::from_string(key)))
            .collect()
    }
    /// `HasEntities`.
    pub fn has_entities(&self) -> bool {
        self.state.read().thelistat
    }
    /// `NbNulls`.
    pub fn nb_nulls(&self) -> i32 {
        self.state.read().thenbnuls
    }
    /// `NbTimes(sign)`: 0 for an unknown signature.
    pub fn nb_times(&self, sign: &str) -> i32 {
        self.state
            .read()
            .thedicount
            .seek(&AsciiString::new(sign))
            .copied()
            .unwrap_or(0)
    }
    /// `Entities(sign)`: the entities of `sign` (`None` without lists, empty for an
    /// unknown signature).
    pub fn entities(&self, sign: &str) -> Option<Vec<Handle<StandardTransient>>> {
        let st = self.state.read();
        if !st.thelistat {
            return None;
        }
        Some(
            st.thediclist
                .seek(&AsciiString::new(sign))
                .cloned()
                .unwrap_or_default(),
        )
    }
    /// `SetName(name)`.
    pub fn set_name(&self, name: &str) {
        self.state.write().thename = AsciiString::new(name);
    }
    /// The base `Name`.
    pub fn name_base(&self) -> String {
        String::from_utf8_lossy(self.state.read().thename.as_bytes()).into_owned()
    }
    /// The counts and the lists (copies) for the printing members.
    fn snapshot(&self) -> ListSnapshot {
        let st = self.state.read();
        let text = |k: &AsciiString| String::from_utf8_lossy(k.as_bytes()).into_owned();
        let counts = (1..=st.thedicount.extent())
            .filter_map(|i| {
                Some((
                    text(st.thedicount.find_key(i).ok()?),
                    *st.thedicount.find_from_index(i).ok()?,
                ))
            })
            .collect();
        let lists = (1..=st.thediclist.extent())
            .filter_map(|i| {
                Some((
                    text(st.thediclist.find_key(i).ok()?),
                    st.thediclist.find_from_index(i).ok()?.clone(),
                ))
            })
            .collect();
        (counts, lists, st.thenbnuls)
    }
}

/// The virtual members of `IFSelect_SignatureList` (`Clear`, `Name`, the printing).
pub trait IFSelectSignatureListOps: Transient {
    /// The embedded list.
    fn signature_list(&self) -> &IFSelectSignatureList;
    /// `Clear`.
    fn clear(&self) {
        self.signature_list().clear_base();
    }
    /// `Name`.
    fn name(&self) -> String {
        self.signature_list().name_base()
    }
    /// `PrintCount(S)`: the count of each signature, then the nulls and the total.
    fn print_count(&self, s: &mut dyn Write) -> io::Result<()> {
        print_count(self.signature_list(), s)
    }
    /// `PrintList(S, model, mod = IFSelect_ListByItem)`: by mode, the counts, the summary,
    /// or the entities (their numbers in `model`) of each signature.
    fn print_list(
        &self,
        s: &mut dyn Write,
        model: &Handle<dyn InterfaceInterfaceModel>,
        mode: IFSelectPrintCount,
    ) -> Result<(), Failure> {
        print_list(self, s, model, mode)
    }
    /// `PrintSum(S)`: the totals and the statistics of the integer signatures.
    fn print_sum(&self, s: &mut dyn Write) -> Result<(), Failure> {
        print_sum(self.signature_list(), s)
    }
}
impl_transient_interface!(
    dyn IFSelectSignatureListOps,
    "IFSelect_SignatureList",
    StandardTransient
);

impl IFSelectSignatureListOps for IFSelectSignatureList {
    fn signature_list(&self) -> &IFSelectSignatureList {
        self
    }
}

/// An I/O failure as a `Failure`.
fn io_failure(err: io::Error) -> Failure {
    Failure::new(&format!("IFSelect_SignatureList: output error {err}"))
}

/// `IFSelect_SignatureList::PrintCount`.
fn print_count(list: &IFSelectSignatureList, s: &mut dyn Write) -> io::Result<()> {
    let name = list.name_base();
    let (counts, _, nbnuls) = list.snapshot();
    write!(s, " Count\t{name}\n -----\t-----------\n")?;
    let mut nbtot = 0;
    for (key, val) in &counts {
        writeln!(
            s,
            "{}{val}\t{key}",
            InterfaceMsg::blanks_for_integer(*val, 6)
        )?;
        nbtot += val;
    }
    if nbnuls > 0 {
        writeln!(s, "{name} Nul : {nbnuls}")?;
    }
    writeln!(s, "    Nb Total:{nbtot}  for {} items", counts.len())
}

/// The entity numbers of one signature in `PrintList`. Native quirk kept: an entity
/// numbered 2 (the value of `IFSelect_ShortByItem`) prints "??".
fn print_entities(
    s: &mut dyn Write,
    list: &[Handle<StandardTransient>],
    model: &Handle<dyn InterfaceInterfaceModel>,
    mode: IFSelectPrintCount,
) -> Result<(), Failure> {
    let nb = list.len();
    let nc = if nb > 5 && mode == IFSelectPrintCount::ShortByItem {
        5
    } else {
        nb
    };
    write!(s, "  - Nb: {nb} : ").map_err(io_failure)?;
    for ent in &list[..nc] {
        if ent.is_null() {
            write!(
                s,
                "  0{}",
                if mode == IFSelectPrintCount::EntitiesByItem {
                    ":(Global)"
                } else {
                    ""
                }
            )
            .map_err(io_failure)?;
            continue;
        }
        let live = model.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_SignatureList::PrintList: null model")
        })?;
        let num = live.number(ent);
        if num == IFSelectPrintCount::ShortByItem as i32 {
            write!(s, "  ??").map_err(io_failure)?;
            continue;
        }
        write!(s, "  {num}").map_err(io_failure)?;
        if mode == IFSelectPrintCount::EntitiesByItem {
            write!(s, ":").map_err(io_failure)?;
            live.print_label(ent, s).map_err(io_failure)?;
        }
    }
    writeln!(s, "{}", if nc < nb { "  .. etc" } else { "" }).map_err(io_failure)
}

/// `IFSelect_SignatureList::PrintList`.
fn print_list<L: IFSelectSignatureListOps + ?Sized>(
    list: &L,
    s: &mut dyn Write,
    model: &Handle<dyn InterfaceInterfaceModel>,
    mode: IFSelectPrintCount,
) -> Result<(), Failure> {
    match mode {
        IFSelectPrintCount::ItemsByEntity => return Ok(()),
        IFSelectPrintCount::CountByItem => return list.print_count(s).map_err(io_failure),
        IFSelectPrintCount::CountSummary => return list.print_sum(s),
        _ => {}
    }
    let base = list.signature_list();
    if !base.has_entities() {
        writeln!(
            s,
            " SignatureList {} : PrintList, list not available",
            list.name()
        )
        .map_err(io_failure)?;
        return list.print_count(s).map_err(io_failure);
    }
    let (_, lists, _) = base.snapshot();
    let mut nbtot = 0;
    for (key, entities) in &lists {
        writeln!(s, "{} : {key}", list.name()).map_err(io_failure)?;
        print_entities(s, entities, model, mode)?;
        nbtot += entities.len();
    }
    writeln!(s, " Nb Total:{nbtot}  for {} items", lists.len()).map_err(io_failure)
}

/// The integer value of a signature in `PrintSum` (an optional blank or sign, then
/// digits), `None` for another text.
fn integer_signature(name: &str) -> Result<Option<i32>, Failure> {
    let bytes = name.as_bytes();
    let digits = bytes
        .iter()
        .enumerate()
        .all(|(i, c)| c.is_ascii_digit() || (i == 0 && matches!(c, b' ' | b'+' | b'-')));
    if !digits {
        return Ok(None);
    }
    AsciiString::new(name).integer_value().map(Some)
}

/// `IFSelect_SignatureList::PrintSum`.
fn print_sum(list: &IFSelectSignatureList, s: &mut dyn Write) -> Result<(), Failure> {
    let (counts, _, _) = list.snapshot();
    write!(s, " Summary {}\n -----\t-----------\n", list.name_base()).map_err(io_failure)?;
    let (mut nbtot, mut maxent, mut nbval, mut nbve, mut minval, mut maxval, mut totval) =
        (0, 0, 0, 0, 0, 0, 0);
    for (name, nbent) in &counts {
        nbtot += nbent;
        maxent = maxent.max(*nbent);
        let Some(val) = integer_signature(name)? else {
            continue;
        };
        if nbval == 0 {
            (minval, maxval) = (val, val);
        }
        (minval, maxval) = (minval.min(val), maxval.max(val));
        nbval += 1;
        nbve += nbent;
        totval += val * nbent;
    }
    writeln!(s, "    Nb Total:{nbtot}  for {} items", counts.len()).map_err(io_failure)?;
    writeln!(s, "    Highest count of entities : {maxent} on one item").map_err(io_failure)?;
    if nbval == 0 {
        return Ok(());
    }
    let avg1 = totval / nbve;
    let avg2 = ((totval - avg1 * nbve) * 10) / nbve;
    let text = format!(
        "    Summary on Integer Values\n    Nb Integer Items : {nbval}\n    For Nb Entities  : {nbve}\n    Cumulated Values : {totval}\n    Maximum Value    : {maxval}\n    Average Value    : {avg1} {avg2}/10\n    Minimum Value    : {minval}\n"
    );
    s.write_all(text.as_bytes()).map_err(io_failure)
}
