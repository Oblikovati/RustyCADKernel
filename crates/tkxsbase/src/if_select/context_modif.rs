// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_ContextModif (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_ContextModif`: the context of a modifier applied to a copied model: the
//! original graph, the copy map, the selected entities and the checks.
use super::IFSelectGeneralModifier;
use crate::interface::{
    InterfaceCheck, InterfaceCheckIterator, InterfaceCopyControl, InterfaceCopyTool,
    InterfaceEntityIterator, InterfaceGraph, InterfaceInterfaceModel, InterfaceProtocol,
};
use rustycad_tkernel::{
    message::Message,
    standard::{Failure, Handle, StandardTransient, exceptions},
    tcollection::AsciiString,
};
use std::fmt::Write as _;

/// `IFSelect_ContextModif`.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectContextModif;
/// fn count(graph: &rustycad_tkxsbase::interface::InterfaceGraph) -> i32 {
///     IFSelectContextModif::new(graph, "").unwrap().selected_count()
/// }
/// ```
pub struct IFSelectContextModif {
    thegraf: InterfaceGraph,
    theprot: Handle<dyn InterfaceProtocol>,
    themap: Handle<dyn InterfaceCopyControl>,
    thefile: AsciiString,
    thelist: Vec<u8>,
    thechek: InterfaceCheckIterator,
    thesel: bool,
    thecurr: i32,
    thecurt: i32,
}

/// The native null dereference of a context without copy map.
fn no_map() -> Failure {
    exceptions::null_object("IFSelect_ContextModif: no copy map for an entity out of the graph")
}

impl IFSelectContextModif {
    /// The common fields, every entity marked `mark`.
    fn build(
        graph: &InterfaceGraph,
        themap: Handle<dyn InterfaceCopyControl>,
        filename: &str,
    ) -> Result<Self, Failure> {
        Ok(Self {
            thegraf: InterfaceGraph::from_graph(graph, false)?,
            theprot: Handle::default(),
            themap,
            thefile: AsciiString::new(filename),
            thelist: vec![b' '; graph.size().max(0) as usize],
            thechek: InterfaceCheckIterator::new(),
            thesel: false,
            thecurr: 0,
            thecurt: 0,
        })
    }
    /// `IFSelect_ContextModif(graph, TC, filename = "")`: the transferred entities of the
    /// copy tool are selected.
    pub fn with_copy(
        graph: &InterfaceGraph,
        tc: &InterfaceCopyTool,
        filename: &str,
    ) -> Result<Self, Failure> {
        let mut ctx = Self::build(graph, tc.control(), filename)?;
        for i in 1..=ctx.thelist.len() as i32 {
            if ctx.search_image(&graph.entity(i)?)?.is_some() {
                ctx.thelist[i as usize - 1] = b'1';
            }
        }
        Ok(ctx)
    }
    /// `IFSelect_ContextModif(graph, filename = "")`: no copy map, every entity selected.
    pub fn new(graph: &InterfaceGraph, filename: &str) -> Result<Self, Failure> {
        let mut ctx = Self::build(graph, Handle::default(), filename)?;
        ctx.thelist.fill(b'1');
        Ok(ctx)
    }
    /// `Search(ent, res)`: the image of `ent` in the copy map (`None` when not
    /// transferred); `Standard_NullObject` without copy map.
    pub fn search_image(
        &self,
        ent: &Handle<StandardTransient>,
    ) -> Result<Option<Handle<StandardTransient>>, Failure> {
        let map = self.themap.get().ok_or_else(no_map)?;
        let mut res = Handle::default();
        Ok(map.search(ent, &mut res)?.then_some(res))
    }
    /// `Select(list)`: only the entities of `list` (transferred, with a copy map).
    /// Native defect kept: without copy map, an entity out of the graph is a null
    /// dereference (`Standard_NullObject`).
    pub fn select(&mut self, list: &InterfaceEntityIterator) -> Result<(), Failure> {
        self.thesel = true;
        self.thelist.fill(b' ');
        let nb = self.thelist.len() as i32;
        for start in list.content() {
            let mut num = self.thegraf.entity_number(&start);
            if num > nb || num < 0 {
                num = 0;
            }
            let marked = if self.themap.is_null() && num > 0 {
                true
            } else {
                self.search_image(&start)?.is_some()
            };
            if marked && num > 0 {
                self.thelist[num as usize - 1] = b'1';
            }
        }
        Ok(())
    }
    /// `OriginalGraph`.
    pub fn original_graph(&self) -> &InterfaceGraph {
        &self.thegraf
    }
    /// `OriginalModel`.
    pub fn original_model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.thegraf.model().clone()
    }
    /// `SetProtocol(proto)`.
    pub fn set_protocol(&mut self, proto: &Handle<dyn InterfaceProtocol>) {
        self.theprot = proto.clone();
    }
    /// `Protocol`.
    pub fn protocol(&self) -> Handle<dyn InterfaceProtocol> {
        self.theprot.clone()
    }
    /// `HasFileName`.
    pub fn has_file_name(&self) -> bool {
        !self.thefile.is_empty()
    }
    /// `FileName`.
    pub fn file_name(&self) -> &AsciiString {
        &self.thefile
    }
    /// `Control`.
    pub fn control(&self) -> Handle<dyn InterfaceCopyControl> {
        self.themap.clone()
    }
    /// `IsForNone`: selected, but no entity is.
    pub fn is_for_none(&self) -> bool {
        self.thesel && self.thelist.iter().all(|c| *c == b' ')
    }
    /// `IsForAll`: not selected (all the entities).
    pub fn is_for_all(&self) -> bool {
        !self.thesel
    }
    /// `IsTransferred(ent)`: true without copy map.
    pub fn is_transferred(&self, ent: &Handle<StandardTransient>) -> Result<bool, Failure> {
        if self.themap.is_null() {
            return Ok(true);
        }
        Ok(self.search_image(ent)?.is_some())
    }
    /// `IsSelected(ent)`.
    pub fn is_selected(&self, ent: &Handle<StandardTransient>) -> bool {
        let num = self.thegraf.entity_number(ent);
        num > 0
            && self
                .thelist
                .get(num as usize - 1)
                .is_some_and(|c| *c != b' ')
    }
    /// The numbers of the selected entities.
    fn selected_numbers(&self) -> impl Iterator<Item = i32> + '_ {
        self.thelist
            .iter()
            .enumerate()
            .filter(|(_, c)| **c != b' ')
            .map(|(i, _)| i as i32 + 1)
    }
    /// `SelectedOriginal`: the selected entities of the original model.
    pub fn selected_original(&self) -> Result<InterfaceEntityIterator, Failure> {
        let mut list = InterfaceEntityIterator::new();
        for num in self.selected_numbers() {
            list.get_one_item(&self.thegraf.entity(num)?);
        }
        Ok(list)
    }
    /// `SelectedResult`: their images (every entity without copy map, as natively).
    pub fn selected_result(&self) -> Result<InterfaceEntityIterator, Failure> {
        let mut list = InterfaceEntityIterator::new();
        for i in 1..=self.thelist.len() as i32 {
            let newent = if self.themap.is_null() {
                self.thegraf.entity(i)?
            } else if self.thelist[i as usize - 1] != b' ' {
                self.search_image(&self.thegraf.entity(i)?)?
                    .unwrap_or_default()
            } else {
                Handle::default()
            };
            if newent.is_some() {
                list.get_one_item(&newent);
            }
        }
        Ok(list)
    }
    /// `SelectedCount`.
    pub fn selected_count(&self) -> i32 {
        self.selected_numbers().count() as i32
    }
    /// `Start`: the first selected entity.
    pub fn start(&mut self) {
        self.thecurr = 0;
        self.thecurt = 0;
        self.next();
    }
    /// `More`.
    pub fn more(&self) -> bool {
        self.thecurr > 0
    }
    /// `Next`: the next selected entity.
    pub fn next(&mut self) {
        let from = self.thecurr.max(0) as usize;
        match self.thelist.iter().skip(from).position(|c| *c != b' ') {
            Some(offset) => {
                self.thecurr = (from + offset) as i32 + 1;
                self.thecurt += 1;
            }
            None => {
                self.thecurr = 0;
                self.thecurt = 0;
            }
        }
    }
    /// `ValueOriginal`: `Standard_NoSuchObject` without current entity.
    pub fn value_original(&self) -> Result<Handle<StandardTransient>, Failure> {
        if self.thecurr <= 0 {
            return Err(exceptions::no_such_object("IFSelect_ContextModif"));
        }
        self.thegraf.entity(self.thecurr)
    }
    /// `ValueResult`: the image of the current entity (itself without copy map).
    pub fn value_result(&self) -> Result<Handle<StandardTransient>, Failure> {
        let ent = self.value_original()?;
        if self.themap.is_null() {
            return Ok(ent);
        }
        Ok(self.search_image(&ent)?.unwrap_or_default())
    }
    /// `TraceModifier(modif)`: a trace of the modifier run (to the default messenger).
    pub fn trace_modifier(&self, modif: &Handle<dyn IFSelectGeneralModifier>) {
        let Some(live) = modif.get() else { return };
        let mut sout = Message::send_info();
        let _ = sout.write_str("---   Run Modifier:\n");
        let sel = live.selection();
        let _ = match sel.get() {
            Some(s) => write!(
                sout,
                "      Selection:{}",
                String::from_utf8_lossy(s.label().as_bytes())
            ),
            None => sout.write_str("  (no Selection)"),
        };
        let (nb, ne) = (self.thelist.len(), self.selected_count() as usize);
        let _ = if nb == ne {
            writeln!(sout, "  All Model ({nb} Entities)")
        } else {
            writeln!(sout, "  Entities,Total:{nb} Concerned:{ne}")
        };
    }
    /// `Trace(mess = "")`: a trace of the current entity (to the default messenger).
    pub fn trace(&self, mess: &str) -> Result<(), Failure> {
        if self.thecurr <= 0 {
            return Ok(());
        }
        let same = self.value_original()? == self.value_result()?;
        let mut sout = Message::send_info();
        let _ = if same {
            writeln!(sout, "--  ContextModif. Entity  n0 {}", self.thecurr)
        } else {
            writeln!(
                sout,
                "--  ContextModif. Entity in Original, n0 {} in Result, n0 {}",
                self.thecurr, self.thecurt
            )
        };
        if !mess.is_empty() {
            let _ = writeln!(sout, "--  Message:{mess}");
        }
        Ok(())
    }
    /// `AddCheck(check)`: a check with messages (number -1 for an entity out of the
    /// graph).
    pub fn add_check(&mut self, check: &Handle<InterfaceCheck>) {
        let Some(live) = check.get().filter(|c| c.nb_fails() + c.nb_warnings() != 0) else {
            return;
        };
        let ent = live.entity();
        let mut num = self.thegraf.entity_number(&ent);
        if num == 0 && ent.is_some() {
            num = -1;
        }
        self.thechek.add(check, num);
    }
    /// `AddWarning(start, mess, orig = "")`.
    pub fn add_warning(&mut self, start: &Handle<StandardTransient>, mess: &str, orig: &str) {
        let num = self.thegraf.entity_number(start);
        self.thechek.c_check(num).add_warning_text(mess, orig);
    }
    /// `AddFail(start, mess, orig = "")`.
    pub fn add_fail(&mut self, start: &Handle<StandardTransient>, mess: &str, orig: &str) {
        let num = self.thegraf.entity_number(start);
        self.thechek.c_check(num).add_fail_text(mess, orig);
    }
    /// `CCheck(num = 0)`: the check of number `num` (with its entity in the graph).
    pub fn c_check(&mut self, num: i32) -> Result<Handle<InterfaceCheck>, Failure> {
        let ach = self.thechek.c_check(num);
        if num > 0 && num <= self.thegraf.size() {
            ach.set_entity(&self.thegraf.entity(num)?);
        }
        Ok(ach)
    }
    /// `CCheck(ent)`: the check of `ent` (number -1 out of the graph).
    pub fn c_check_entity(&mut self, ent: &Handle<StandardTransient>) -> Handle<InterfaceCheck> {
        let mut num = self.thegraf.entity_number(ent);
        if num == 0 {
            num = -1;
        }
        let ach = self.thechek.c_check(num);
        ach.set_entity(ent);
        ach
    }
    /// `CheckList`.
    pub fn check_list(&self) -> InterfaceCheckIterator {
        self.thechek.clone()
    }
}
