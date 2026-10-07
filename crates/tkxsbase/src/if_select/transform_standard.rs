// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_TransformStandard (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_TransformStandard`: a transformation by model modifiers, on a copy of the
//! model (standard copy) or on the model itself (on the spot).
use super::{IFSelectContextModif, IFSelectModifier, IFSelectSelection, IFSelectTransformer};
use crate::interface::{
    InterfaceCheckIterator, InterfaceCopyControl, InterfaceCopyTool, InterfaceGraph,
    InterfaceInterfaceModel, InterfaceModelEntities, InterfaceProtocol,
};
use rustycad_tkernel::{
    impl_transient,
    message::Message,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::AsciiString,
};
use std::fmt::Write as _;

/// The fields of `IFSelect_TransformStandard`.
struct TransformState {
    thecopy: bool,
    thesel: Handle<dyn IFSelectSelection>,
    themodifs: Vec<Handle<dyn IFSelectModifier>>,
    themap: Handle<dyn InterfaceCopyControl>,
}

/// `IFSelect_TransformStandard`.
///
/// ```
/// use rustycad_tkxsbase::if_select::{IFSelectTransformStandard, IFSelectTransformer};
/// let trf = IFSelectTransformStandard::new();
/// assert_eq!(trf.label().as_bytes(), b"Standard Copy (no Modifier)");
/// ```
pub struct IFSelectTransformStandard {
    transient: StandardTransient,
    state: SharedState<TransformState>,
}
impl_transient!(IFSelectTransformStandard, "IFSelect_TransformStandard", dyn IFSelectTransformer, transient, interfaces: [dyn IFSelectTransformer]);

impl Default for IFSelectTransformStandard {
    fn default() -> Self {
        Self::new()
    }
}

/// The live model of a graph (a native null dereference without one).
fn model_of(g: &InterfaceGraph) -> Result<&dyn InterfaceInterfaceModel, Failure> {
    g.model()
        .get()
        .ok_or_else(|| exceptions::null_object("IFSelect_TransformStandard: graph without model"))
}

impl IFSelectTransformStandard {
    /// `IFSelect_TransformStandard()`: standard copy, no modifier.
    pub fn new() -> Self {
        let state = TransformState {
            thecopy: true,
            thesel: Handle::default(),
            themodifs: Vec::new(),
            themap: Handle::default(),
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// `SetCopyOption(option)`.
    pub fn set_copy_option(&self, option: bool) {
        self.state.write().thecopy = option;
    }
    /// `CopyOption`.
    pub fn copy_option(&self) -> bool {
        self.state.read().thecopy
    }
    /// `SetSelection(sel)`.
    pub fn set_selection(&self, sel: &Handle<dyn IFSelectSelection>) {
        self.state.write().thesel = sel.clone();
    }
    /// `Selection`.
    pub fn selection(&self) -> Handle<dyn IFSelectSelection> {
        self.state.read().thesel.clone()
    }
    /// `NbModifiers`.
    pub fn nb_modifiers(&self) -> i32 {
        self.state.read().themodifs.len() as i32
    }
    /// `Modifier(num)`: `Standard_OutOfRange` out of 1..NbModifiers.
    pub fn modifier(&self, num: i32) -> Result<Handle<dyn IFSelectModifier>, Failure> {
        let st = self.state.read();
        usize::try_from(num - 1)
            .ok()
            .and_then(|i| st.themodifs.get(i))
            .cloned()
            .ok_or_else(|| {
                exceptions::out_of_range(&format!(
                    "IFSelect_TransformStandard::Modifier: {num}; expected [1, {}]",
                    st.themodifs.len()
                ))
            })
    }
    /// The modifier `num`, not null (a native null dereference otherwise).
    fn live_modifier(&self, num: i32) -> Result<Handle<dyn IFSelectModifier>, Failure> {
        let modif = self.modifier(num)?;
        if modif.is_null() {
            return Err(exceptions::null_object(&format!(
                "IFSelect_TransformStandard: modifier {num} is null"
            )));
        }
        Ok(modif)
    }
    /// `ModifierRank(modif)`: 0 when absent.
    pub fn modifier_rank(&self, modif: &Handle<dyn IFSelectModifier>) -> i32 {
        self.state
            .read()
            .themodifs
            .iter()
            .rposition(|m| m == modif)
            .map_or(0, |p| p as i32 + 1)
    }
    /// `AddModifier(modif, atnum = 0)`: appended (0) or inserted before `atnum`; false
    /// out of range.
    pub fn add_modifier(&self, modif: &Handle<dyn IFSelectModifier>, atnum: i32) -> bool {
        let mut st = self.state.write();
        if atnum < 0 || atnum > st.themodifs.len() as i32 {
            return false;
        }
        if atnum == 0 {
            st.themodifs.push(modif.clone());
        } else {
            st.themodifs.insert(atnum as usize - 1, modif.clone());
        }
        true
    }
    /// `RemoveModifier(modif)`.
    pub fn remove_modifier(&self, modif: &Handle<dyn IFSelectModifier>) -> bool {
        self.remove_modifier_at(self.modifier_rank(modif))
    }
    /// `RemoveModifier(num)`: false out of range.
    pub fn remove_modifier_at(&self, num: i32) -> bool {
        let mut st = self.state.write();
        if num <= 0 || num > st.themodifs.len() as i32 {
            return false;
        }
        st.themodifs.remove(num as usize - 1);
        true
    }
    /// `Copy(G, TC)`: the standard copy or the model itself, by the copy option.
    pub fn copy(
        &self,
        g: &InterfaceGraph,
        tc: &mut InterfaceCopyTool,
    ) -> Result<Handle<dyn InterfaceInterfaceModel>, Failure> {
        if self.copy_option() {
            self.standard_copy(g, tc)
        } else {
            self.on_the_spot(g, tc)
        }
    }
    /// `StandardCopy(G, TC)`: a new model with a copy of every entity.
    pub fn standard_copy(
        &self,
        g: &InterfaceGraph,
        tc: &mut InterfaceCopyTool,
    ) -> Result<Handle<dyn InterfaceInterfaceModel>, Failure> {
        let original = model_of(g)?;
        let newmod = original.new_empty_model();
        tc.clear()?;
        for i in 1..=g.size() {
            tc.transfer_entity(&original.value(i)?)?;
        }
        tc.fill_model(&newmod)?;
        Ok(newmod)
    }
    /// `OnTheSpot(G, TC)`: the model itself, each entity bound to itself.
    pub fn on_the_spot(
        &self,
        g: &InterfaceGraph,
        tc: &mut InterfaceCopyTool,
    ) -> Result<Handle<dyn InterfaceInterfaceModel>, Failure> {
        for i in 1..=g.size() {
            let ent = g.entity(i)?;
            tc.bind(&ent, &ent)?;
        }
        Ok(g.model().clone())
    }
    /// One modifier of `ApplyModifiers`, `(i, nb)` its rank and the count: false to
    /// abandon (fails reported).
    fn apply_one(
        &self,
        (i, nb): (i32, i32),
        g: &InterfaceGraph,
        protocol: &Handle<dyn InterfaceProtocol>,
        tc: &mut InterfaceCopyTool,
        checks: &mut InterfaceCheckIterator,
        newmod: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        let unmod = self.live_modifier(i)?;
        let mut ctx = IFSelectContextModif::with_copy(g, tc, "")?;
        let mut sel = self.selection();
        if sel.is_null() {
            sel = unmod.selection();
        }
        if let Some(live) = sel.get() {
            ctx.select(&live.unique_result(g)?)?;
        }
        if ctx.is_for_none() {
            return Ok(true);
        }
        unmod.perform(&mut ctx, newmod, protocol, tc)?;
        let checklist = ctx.check_list();
        let mut sout = Message::send_info();
        if !checklist.is_empty(false) {
            checks.merge(&checklist);
            let _ = writeln!(
                sout,
                "IFSelect_TransformStandard :  Messages from Modifier n0 {i} of {nb}"
            );
            let mut text = Vec::new();
            let _ = checklist.print_with_model(&mut text, newmod, false, 0);
            let _ = sout.write_str(&String::from_utf8_lossy(&text));
        }
        if !checklist.is_empty(true) {
            let _ = writeln!(sout, " --  Abandon TransformStandard  --");
            return Ok(false);
        }
        Ok(true)
    }
    /// `ApplyModifiers(G, protocol, TC, checks, newmod)`: the modifiers in turn (false
    /// when one gives fails); `newmod` becomes null when it is the original model and no
    /// modifier may change the graph.
    pub fn apply_modifiers(
        &self,
        g: &InterfaceGraph,
        protocol: &Handle<dyn InterfaceProtocol>,
        tc: &mut InterfaceCopyTool,
        checks: &mut InterfaceCheckIterator,
        newmod: &mut Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        let (mut res, mut chg) = (true, false);
        let nb = self.nb_modifiers();
        for i in 1..=nb {
            chg |= self.live_modifier(i)?.may_change_graph();
            if !self.apply_one((i, nb), g, protocol, tc, checks, newmod)? {
                res = false;
                break;
            }
        }
        if *newmod == *g.model() && !chg {
            *newmod = Handle::default();
        }
        Ok(res)
    }
}

impl IFSelectTransformer for IFSelectTransformStandard {
    /// `Perform(G, protocol, checks, newmod)`: the copy, then the modifiers.
    fn perform(
        &self,
        g: &InterfaceGraph,
        protocol: &Handle<dyn InterfaceProtocol>,
        checks: &mut InterfaceCheckIterator,
        newmod: &mut Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        let mut tc = InterfaceCopyTool::with_protocol(g.model(), protocol)?;
        self.state.write().themap = tc.control();
        *newmod = self.copy(g, &mut tc)?;
        self.apply_modifiers(g, protocol, &mut tc, checks, newmod)
    }
    /// `Updated(entfrom, entto)`: the image in the copy map of the last run.
    fn updated(
        &self,
        entfrom: &Handle<StandardTransient>,
        entto: &mut Handle<StandardTransient>,
    ) -> bool {
        let map = self.state.read().themap.clone();
        map.get()
            .is_some_and(|m| m.search(entfrom, entto).unwrap_or(false))
    }
    /// `Label`: the copy mode and the modifiers.
    fn label(&self) -> AsciiString {
        let mode = if self.copy_option() {
            "Standard Copy"
        } else {
            "On the spot Edition"
        };
        let nb = self.nb_modifiers();
        let tail = match nb {
            0 => String::from(" (no Modifier)"),
            1 => format!(
                " - {}",
                self.live_modifier(1)
                    .map(|m| String::from_utf8_lossy(m.label().as_bytes()).into_owned())
                    .unwrap_or_default()
            ),
            _ => format!(" - {nb} Modifiers"),
        };
        AsciiString::new(&format!("{mode}{tail}"))
    }
}
