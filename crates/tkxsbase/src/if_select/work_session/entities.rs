// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the entities, the graph and the checks.

//! The entities of the model (numbers, labels, names, categories), the data clearing,
//! the graph and the checks of the session.
use super::{FLAG_INCORRECT, IFSelectTransientList, IFSelectWorkSession};
use crate::c_text::atoi;
use crate::if_select::{
    IFSelectEditForm, IFSelectSelectPointed, IFSelectSignCounter, IFSelectSignValidity,
    IFSelectSignatureList, IFSelectSignatureListOps,
};
use crate::interface::{
    InterfaceCategory, InterfaceCheckIterator, InterfaceCheckTool, InterfaceGraph, InterfaceHGraph,
    InterfaceInterfaceModel, InterfaceModelEntities, InterfaceModelReports, InterfaceShareTool,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, RuntimeTyped, StandardTransient, exceptions},
    tcollection::{AsciiString, HAsciiString},
};
use std::sync::RwLockReadGuard;

/// A query of the graph on an entity (its shared or sharing entities).
type GraphQuery = fn(
    &InterfaceGraph,
    &Handle<StandardTransient>,
) -> Result<Vec<Handle<StandardTransient>>, Failure>;

/// The graph of a session, `Standard_DomainError` when not available.
fn no_graph() -> Failure {
    exceptions::domain_error("IFSelect WorkSession : Graph not available")
}

impl IFSelectWorkSession {
    /// `NbStartingEntities`: the entities of the model (0 without model).
    pub fn nb_starting_entities(&self) -> i32 {
        self.model().get().map_or(0, |m| m.nb_entities())
    }
    /// `StartingEntity(num)`: null out of range.
    pub fn starting_entity(&self, num: i32) -> Handle<StandardTransient> {
        let model = self.model();
        let Some(m) = model.get() else {
            return Handle::default();
        };
        if num < 1 || num > m.nb_entities() {
            return Handle::default();
        }
        m.value(num).unwrap_or_default()
    }
    /// `StartingNumber(ent)`: 0 when not in the model.
    pub fn starting_number(&self, ent: &Handle<StandardTransient>) -> i32 {
        self.model().get().map_or(0, |m| m.number(ent))
    }
    /// `NumberFromLabel(val, afternum = 0)`: a number given directly, else the number of
    /// the only entity whose label ends with `val` (minus the first one when several).
    pub fn number_from_label(&self, val: &str, afternum: i32) -> i32 {
        let mut num = atoi(val.as_bytes());
        let model = self.model();
        let Some(m) = model.get().filter(|_| num <= 0) else {
            return num; // a direct n0 : won !
        };
        if num > m.nb_entities() {
            return 0;
        }
        let (mut cnt, after) = (0, afternum.abs());
        let mut i = m.next_number_for_label(val, after, false);
        while i != 0 {
            cnt += 1;
            if num <= 0 {
                num = i;
            }
            i = m.next_number_for_label(val, i, false);
        }
        if cnt == 1 { num } else { -num }
    }
    /// `EntityLabel(ent)`: null for an entity out of the model.
    pub fn entity_label(&self, ent: &Handle<StandardTransient>) -> Handle<HAsciiString> {
        let model = self.model();
        match model.get() {
            Some(m) if ent.is_some() && m.contains(ent) => m.string_label(ent),
            _ => Handle::default(),
        }
    }
    /// `EntityName(ent)`: the name the general module gives (null when none);
    /// `Standard_NullObject` without graph (a native null dereference).
    pub fn entity_name(
        &self,
        ent: &Handle<StandardTransient>,
    ) -> Result<Handle<HAsciiString>, Failure> {
        let (model, hgraph, gtool) = {
            let st = self.state.read();
            (
                st.my_model.clone(),
                st.thegraph.clone(),
                st.thegtool.clone(),
            )
        };
        if model.is_null() || ent.is_null() {
            return Ok(Handle::default());
        }
        let graph = hgraph
            .get()
            .ok_or_else(|| exceptions::null_object("IFSelect_WorkSession::EntityName: no graph"))?;
        let sht = InterfaceShareTool::from_graph(&graph.graph())?;
        let Some((module, cn)) = gtool.get().and_then(|g| g.select(ent, false)) else {
            return Ok(Handle::default());
        };
        Ok(module
            .get()
            .map(|m| m.name(cn, ent, &sht))
            .unwrap_or_default())
    }
    /// `CategoryNumber(ent)`: -1 without model.
    pub fn category_number(&self, ent: &Handle<StandardTransient>) -> i32 {
        let num = self.starting_number(ent);
        self.model().get().map_or(-1, |m| m.category_number(num))
    }
    /// `CategoryName(ent)`.
    pub fn category_name(&self, ent: &Handle<StandardTransient>) -> String {
        InterfaceCategory::name(self.category_number(ent))
    }
    /// `ValidityName(ent)`: empty for an entity out of the model.
    pub fn validity_name(&self, ent: &Handle<StandardTransient>) -> &'static str {
        if self.starting_number(ent) == 0 {
            return "";
        }
        IFSelectSignValidity::c_val(ent, &self.model())
    }
    /// The items of the kind `T`, with their idents.
    pub(crate) fn items_of<T: ?Sized + RuntimeTyped>(&self) -> Vec<Handle<T>> {
        let idents = self.item_idents(T::static_type());
        idents.iter().map(|id| self.item(*id).cast::<T>()).collect()
    }
    /// The non virtual `ClearData(mode)` (see [`super::IFSelectWorkSessionOps::clear_data`]).
    /// Native defect kept: mode 4 also clears all the items (the pointed selections,
    /// counters and edit forms cleared before stay unreachable by ident).
    pub fn clear_data_base(&self, mode: i32) -> Result<(), Failure> {
        match mode {
            1 => self.clear_all_data(),
            2 => {
                let mut st = self.state.write();
                st.thegraph = Handle::default();
                st.thecheckdone = false;
                st.thecheckana.clear();
                Ok(())
            }
            3 => {
                self.state.write().thecheckdone = false;
                Ok(())
            }
            4 => self.clear_pointed_data(),
            _ => Ok(()),
        }
    }
    /// `ClearData(1)`: the model and all that depends on it.
    fn clear_all_data(&self) -> Result<(), Failure> {
        let model = {
            let mut st = self.state.write();
            st.theloaded = AsciiString::default();
            std::mem::take(&mut st.my_model)
        };
        if let Some(m) = model.get() {
            m.clear();
        }
        self.clear_data_virtual(2)?;
        self.clear_data_virtual(4)?;
        self.state.write().thecheckrun.clear();
        Ok(())
    }
    /// `ClearData(4)`: the pointed selections, signature lists (counters back to the
    /// mode -1) and edit forms are cleared, then the items.
    fn clear_pointed_data(&self) -> Result<(), Failure> {
        for sp in self.items_of::<IFSelectSelectPointed>() {
            if let Some(p) = sp.get() {
                p.clear();
            }
        }
        for sl in self.items_of::<IFSelectSignatureList>() {
            if let Some(l) = sl.cast::<dyn IFSelectSignatureListOps>().get() {
                l.clear();
            }
            if let Some(sc) = sl.cast::<IFSelectSignCounter>().get() {
                sc.set_sel_mode(-1);
            }
        }
        for edf in self.items_of::<IFSelectEditForm>() {
            let form = edf.get().ok_or_else(|| {
                exceptions::null_object("IFSelect_WorkSession::ClearData: edit form expected")
            })?;
            form.clear_data();
        }
        self.state.write().theitems.clear();
        Ok(())
    }
    /// `ComputeGraph(enforce = false)`: the graph of the model (kept when its size still
    /// matches, unless enforced), then the checks and, in statistics mode, the statuses,
    /// the "Incorrect" flag and the categories. False without protocol, model or entity.
    pub fn compute_graph(&self, enforce: bool) -> Result<bool, Failure> {
        let (protocol, model, hgraph, modelstat) = {
            let st = self.state.read();
            (
                st.theprotocol.clone(),
                st.my_model.clone(),
                st.thegraph.clone(),
                st.themodelstat,
            )
        };
        let Some(m) = model.get().filter(|_| protocol.is_some()) else {
            return Ok(false);
        };
        if !enforce
            && hgraph
                .get()
                .is_some_and(|g| g.graph().size() == m.nb_entities())
        {
            return Ok(true);
        }
        self.state.write().thegraph = Handle::default();
        if m.nb_entities() == 0 {
            return Ok(false);
        }
        let hgraph = Handle::new(InterfaceHGraph::new(&model, modelstat)?);
        self.state.write().thegraph = hgraph.clone();
        if modelstat {
            init_statistics(&hgraph)?;
        }
        self.compute_check(false)?;
        self.state.write().thecheckdone = true;
        if modelstat {
            self.compute_categories(&hgraph, m)?;
        }
        Ok(true)
    }
    /// The category of each entity of the model, in statistics mode.
    fn compute_categories(
        &self,
        hgraph: &Handle<InterfaceHGraph>,
        m: &dyn InterfaceInterfaceModel,
    ) -> Result<(), Failure> {
        let categ = InterfaceCategory::with_gtool(&self.state.read().thegtool);
        let sht = InterfaceShareTool::from_h_graph(hgraph);
        for i in 1..=m.nb_entities() {
            m.set_category_number(i, categ.cat_num(&m.value(i)?, &sht))?;
        }
        Ok(())
    }
    /// `HGraph`: the graph computed if needed (null when not available).
    pub fn h_graph(&self) -> Result<Handle<InterfaceHGraph>, Failure> {
        self.compute_graph(false)?;
        Ok(self.state.read().thegraph.clone())
    }
    /// `Graph`: the graph computed if needed, read through its handle;
    /// `Standard_DomainError` when not available.
    pub fn graph(&self) -> Result<Handle<InterfaceHGraph>, Failure> {
        let hgraph = self.h_graph()?;
        if hgraph.is_null() {
            return Err(no_graph());
        }
        Ok(hgraph)
    }
    /// The current graph (not computed), `None` when not available.
    pub(crate) fn current_graph(&self) -> Option<Handle<InterfaceHGraph>> {
        let hgraph = self.state.read().thegraph.clone();
        hgraph.is_some().then_some(hgraph)
    }
    /// The entities a query of the graph gives on `ent`, `None` without graph or for an
    /// entity out of the model.
    fn graph_query(
        &self,
        ent: &Handle<StandardTransient>,
        query: GraphQuery,
    ) -> Result<IFSelectTransientList, Failure> {
        if !self.compute_graph(false)? || self.starting_number(ent) == 0 {
            return Ok(None);
        }
        let hgraph = self.state.read().thegraph.clone();
        let graph: RwLockReadGuard<'_, InterfaceGraph> = hgraph.get().ok_or_else(no_graph)?.graph();
        query(&graph, ent).map(Some)
    }
    /// `Shareds(ent)`: the entities `ent` shares (`None` when not available).
    pub fn shareds(
        &self,
        ent: &Handle<StandardTransient>,
    ) -> Result<IFSelectTransientList, Failure> {
        self.graph_query(ent, |g, e| Ok(g.shareds(e)?.content()))
    }
    /// `Sharings(ent)`: the entities sharing `ent` (`None` when not available).
    pub fn sharings(
        &self,
        ent: &Handle<StandardTransient>,
    ) -> Result<IFSelectTransientList, Failure> {
        self.graph_query(ent, |g, e| Ok(g.sharings(e)?.content()))
    }
    /// `IsLoaded`: a protocol, a non empty model and its graph of the same size.
    pub fn is_loaded(&self) -> bool {
        let st = self.state.read();
        let (Some(m), Some(g)) = (st.my_model.get(), st.thegraph.get()) else {
            return false;
        };
        st.theprotocol.is_some() && m.nb_entities() != 0 && m.nb_entities() == g.graph().size()
    }
    /// `ComputeCheck(enforce = false)`: the check of the loaded model (done once unless
    /// enforced); in statistics mode the failing entities are flagged "Incorrect".
    pub fn compute_check(&self, enforce: bool) -> Result<bool, Failure> {
        if enforce {
            self.state.write().thecheckdone = false;
        }
        if self.state.read().thecheckdone {
            return Ok(true);
        }
        if !self.is_loaded() {
            return Ok(false);
        }
        let (hgraph, model, modelstat) = {
            let st = self.state.read();
            (st.thegraph.clone(), st.my_model.clone(), st.themodelstat)
        };
        let checklist = InterfaceCheckTool::from_h_graph(&hgraph)?.verify_check_list()?;
        let m = model.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::ComputeCheck: no model")
        })?;
        m.fill_semantic_checks(&checklist, false)?;
        if modelstat {
            flag_incorrect(&hgraph, &checklist, m)?;
        }
        Ok(true)
    }
    /// `ModelCheckList(complete = true)`: the complete or syntactic check of the model; a
    /// fail "DATA NOT AVAILABLE FOR CHECK" when not loaded.
    pub fn model_check_list(&self, complete: bool) -> Result<InterfaceCheckIterator, Failure> {
        let mut checks = InterfaceCheckIterator::new();
        if !self.is_loaded() {
            checks
                .c_check(0)
                .add_fail_text("DATA NOT AVAILABLE FOR CHECK", "");
            return Ok(checks);
        }
        let mut cht = InterfaceCheckTool::from_graph(&self.graph()?.graph())?;
        checks = if complete {
            cht.complete_check_list()?
        } else {
            cht.analyse_check_list()?
        };
        checks.set_name(if complete {
            "Model Complete Check List"
        } else {
            "Model Syntactic Check List"
        });
        Ok(checks)
    }
    /// `CheckOne(ent, complete = true)`: the check of one entity (of the model for null
    /// or the model itself), syntactic and, when complete, semantic.
    pub fn check_one(
        &self,
        ent: &Handle<StandardTransient>,
        complete: bool,
    ) -> Result<InterfaceCheckIterator, Failure> {
        let model = self.model();
        let mut checks = InterfaceCheckIterator::new();
        checks.set_model(&model);
        if !self.is_loaded() {
            checks
                .c_check(0)
                .add_fail_text("DATA NOT AVAILABLE FOR CHECK", "");
            return Ok(checks);
        }
        let m = model
            .get()
            .ok_or_else(|| exceptions::null_object("IFSelect_WorkSession::CheckOne: no model"))?;
        let is_model = ent.is_null() || ent.object_address() == model.object_address();
        let num = if is_model { 0 } else { m.number(ent) };
        let ach = m.check(num, true);
        if complete {
            ach.get_messages(&m.check(num, false));
        }
        if num > 0 {
            ach.set_entity(ent);
        }
        checks.add(&ach, num);
        checks.set_name("Data Check (One Entity)");
        Ok(checks)
    }
    /// `LastRunCheckList`.
    pub fn last_run_check_list(&self) -> InterfaceCheckIterator {
        self.state.read().thecheckrun.clone()
    }
}

/// The statistics of a new graph: statuses 0 and the "Incorrect" flag.
fn init_statistics(hgraph: &Handle<InterfaceHGraph>) -> Result<(), Failure> {
    let live = hgraph.get().ok_or_else(no_graph)?;
    let mut graph = live.c_graph();
    for i in 1..=graph.size() {
        graph.set_status(i, 0)?;
    }
    let bm = graph.c_bit_map();
    bm.add_flag("")?;
    bm.set_flag_name(FLAG_INCORRECT, "Incorrect");
    Ok(())
}

/// The failing and error entities flagged "Incorrect" in the bit map of the graph.
fn flag_incorrect(
    hgraph: &Handle<InterfaceHGraph>,
    checklist: &InterfaceCheckIterator,
    m: &dyn InterfaceInterfaceModel,
) -> Result<(), Failure> {
    let live = hgraph.get().ok_or_else(no_graph)?;
    let graph = live.c_graph();
    let (bm, nb) = (graph.bit_map(), graph.size());
    bm.init(false, FLAG_INCORRECT)?;
    checklist.start();
    while checklist.more() {
        let num = checklist.number()?;
        if checklist.value()?.has_failed() && num > 0 && num <= nb {
            bm.set_true(num, FLAG_INCORRECT)?;
        }
        checklist.next();
    }
    for num in 1..=nb {
        if m.is_error_entity(num) {
            bm.set_true(num, FLAG_INCORRECT)?;
        }
    }
    Ok(())
}
