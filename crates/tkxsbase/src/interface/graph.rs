// Rust port of DataExchange/TKXSBase/Interface/Interface_Graph (.hxx, .cxx: construction,
// evaluation and statuses) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_Graph`: the sharing relations of the entities of a model (who shares whom),
//! with a status and a presence flag per entity for the selections. The queries are in
//! `graph_queries`, the filling members in `graph_fill`.
use super::int_array::{InterfaceIntArray, int_at, int_upper, new_int_array, set_int};
use super::{
    InterfaceBitMap, InterfaceEntityIterator, InterfaceGTool, InterfaceGeneralLib,
    InterfaceInterfaceModel, InterfaceModelEntities, InterfaceModelReports, InterfaceProtocol,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, StandardTransient, exceptions},
    tcollection::AsciiString,
};
use std::sync::Arc;

/// `Graph_Present`: the flag of the entities present in the graph.
pub(super) const GRAPH_PRESENT: i32 = 0;
/// `Graph_ShareError`: the flag of the entities sharing an entity out of the model.
pub(super) const GRAPH_SHARE_ERROR: i32 = 1;

/// The sharing table: for each entity (index 0 is entity 1), the numbers of the entities
/// that share it, in the native order (natively a shared
/// `NCollection_HArray1<NCollection_List<int>>`).
pub type InterfaceSharingTable = Arc<Vec<Vec<i32>>>;

/// `Interface_Graph`.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceGraph;
/// fn roots(graph: &InterfaceGraph) -> i32 { graph.root_entities().map_or(0, |r| r.nb_entities()) }
/// ```
#[derive(Clone)]
pub struct InterfaceGraph {
    pub(super) themodel: Handle<dyn InterfaceInterfaceModel>,
    pub(super) thepresents: AsciiString,
    pub(super) thestats: InterfaceIntArray,
    pub(super) thesharings: Option<InterfaceSharingTable>,
    pub(super) theflags: InterfaceBitMap,
}

impl InterfaceGraph {
    /// `Interface_Graph(amodel, lib, theModeStats = true)` (the library is not used).
    pub fn with_lib(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        _lib: &InterfaceGeneralLib,
        mode_stats: bool,
    ) -> Result<Self, Failure> {
        Self::new(amodel, mode_stats)
    }
    /// `Interface_Graph(amodel, protocol, theModeStats = true)` (the protocol is not used).
    pub fn with_protocol(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        _protocol: &Handle<dyn InterfaceProtocol>,
        mode_stats: bool,
    ) -> Result<Self, Failure> {
        Self::new(amodel, mode_stats)
    }
    /// `Interface_Graph(amodel, gtool, theModeStats = true)` (the GTool is not used).
    pub fn with_gtool(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        _gtool: &Handle<InterfaceGTool>,
        mode_stats: bool,
    ) -> Result<Self, Failure> {
        Self::new(amodel, mode_stats)
    }
    /// `Interface_Graph(amodel, theModeStats = true)`: evaluates the sharings through the
    /// GTool of the model; with statuses, every entity starts absent with status 0.
    pub fn new(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        mode_stats: bool,
    ) -> Result<Self, Failure> {
        let mut graph = Self {
            themodel: amodel.clone(),
            thepresents: AsciiString::default(),
            thestats: InterfaceIntArray::default(),
            thesharings: None,
            theflags: InterfaceBitMap::new()?,
        };
        if mode_stats {
            graph.init_stats()?;
        }
        graph.evaluate()?;
        Ok(graph)
    }
    /// `Interface_Graph(agraph, copied = false)`: shares the sharing table, copies the
    /// statuses and the flags.
    pub fn from_graph(agraph: &InterfaceGraph, _copied: bool) -> Result<Self, Failure> {
        let mut graph = Self {
            themodel: agraph.themodel.clone(),
            thepresents: AsciiString::default(),
            thestats: InterfaceIntArray::default(),
            thesharings: agraph.thesharings.clone(),
            theflags: InterfaceBitMap::new()?,
        };
        graph.copy_statuses(agraph)?;
        Ok(graph)
    }
    /// `operator=(theOther)`.
    pub fn assign(&mut self, other: &InterfaceGraph) -> Result<(), Failure> {
        self.themodel = other.themodel.clone();
        self.thepresents = other.thepresents.clone();
        self.thesharings = other.thesharings.clone();
        self.thestats = InterfaceIntArray::default();
        self.copy_statuses(other)
    }
    fn copy_statuses(&mut self, other: &InterfaceGraph) -> Result<(), Failure> {
        let nb = other.nb_statuses();
        if nb == 0 {
            return Ok(());
        }
        self.thestats = new_int_array(1, nb)?;
        for i in 1..=nb {
            set_int(&self.thestats, i, other.status(i)?)?;
        }
        self.theflags.initialize_from(&other.theflags, true)
    }
    /// `InitStats` (protected): a status per entity, the presence and share-error flags.
    pub(super) fn init_stats(&mut self) -> Result<(), Failure> {
        let nb = self.size();
        self.thestats = new_int_array(1, nb)?;
        self.theflags.initialize(nb, 2)?;
        self.theflags.add_flag("ShareError")?;
        Ok(())
    }
    /// `NbStatuses`: 0 without statuses.
    pub fn nb_statuses(&self) -> i32 {
        self.thestats.get().map_or(0, |s| s.array1().length())
    }
    /// `SharingTable`.
    pub fn sharing_table(&self) -> Option<InterfaceSharingTable> {
        self.thesharings.clone()
    }
    /// `Evaluate` (private): the sharing lists, from the shared entities each module
    /// lists; an entity sharing one out of the model gets the share-error flag.
    fn evaluate(&mut self) -> Result<(), Failure> {
        let count = self.size();
        let mut table: Vec<Vec<i32>> = vec![Vec::new(); count.max(0) as usize];
        let model = self.live_model()?.clone();
        let gtool = model.gtool();
        if gtool.is_some() {
            for i in 1..=count {
                self.evaluate_entity(&model, &gtool, i, &mut table)?;
            }
        }
        self.thesharings = Some(Arc::new(table));
        Ok(())
    }
    fn evaluate_entity(
        &mut self,
        model: &Handle<dyn InterfaceInterfaceModel>,
        gtool: &Handle<InterfaceGTool>,
        i: i32,
        table: &mut [Vec<i32>],
    ) -> Result<(), Failure> {
        let entity = model.value(i)?;
        let number = self.entity_number(&entity);
        if number == 0 {
            return Ok(());
        }
        let current = redefined_content(model, number, &entity);
        let mut iter = InterfaceEntityIterator::new();
        if let Some((module, cn)) = gtool.select(&current, false) {
            module.fill_shared(model, cn, &current, &mut iter);
        }
        for shared in iter.content() {
            if shared == entity {
                continue;
            }
            let share_num = self.entity_number(&shared);
            if share_num == 0 {
                if self.thestats.is_some() {
                    self.theflags.set_true(i, GRAPH_SHARE_ERROR)?;
                }
                continue;
            }
            table[(share_num - 1) as usize].push(i);
        }
        Ok(())
    }
    /// `Reset`: no statuses any more (the presence flags are cleared).
    pub fn reset(&mut self) -> Result<(), Failure> {
        if self.thestats.is_null() {
            return Ok(());
        }
        self.thestats.nullify();
        self.theflags.init(false, GRAPH_PRESENT)
    }
    /// `ResetStatus`: every status 0, every entity absent.
    pub fn reset_status(&mut self) -> Result<(), Failure> {
        if self.thestats.is_null() {
            return Ok(());
        }
        zero_statuses(&self.thestats)?;
        self.theflags.init(false, GRAPH_PRESENT)
    }
    /// `Size`: the number of entities of the model.
    pub fn size(&self) -> i32 {
        self.themodel.get().map_or(0, |m| m.nb_entities())
    }
    /// `EntityNumber(ent)`: its number in the model.
    pub fn entity_number(&self, ent: &Handle<StandardTransient>) -> i32 {
        self.themodel.get().map_or(0, |m| m.number(ent))
    }
    /// `IsPresent(num)`: false out of range or without statuses.
    pub fn is_present(&self, num: i32) -> Result<bool, Failure> {
        if num <= 0 || num > self.size() || self.thestats.is_null() {
            return Ok(false);
        }
        self.theflags.value(num, GRAPH_PRESENT)
    }
    /// `IsPresent(ent)`.
    pub fn is_present_entity(&self, ent: &Handle<StandardTransient>) -> Result<bool, Failure> {
        self.is_present(self.entity_number(ent))
    }
    /// `Entity(num)`: `Standard_OutOfRange` out of the model.
    pub fn entity(&self, num: i32) -> Result<Handle<StandardTransient>, Failure> {
        self.live_model()?.value(num)
    }
    /// `Status(num)`: 0 without statuses.
    pub fn status(&self, num: i32) -> Result<i32, Failure> {
        if self.thestats.is_null() {
            return Ok(0);
        }
        int_at(&self.thestats, num)
    }
    /// `SetStatus(num, stat)`: nothing without statuses.
    pub fn set_status(&mut self, num: i32, stat: i32) -> Result<(), Failure> {
        if self.thestats.is_null() {
            return Ok(());
        }
        set_int(&self.thestats, num, stat)
    }
    /// `RemoveItem(num)`: status 0, absent.
    pub fn remove_item(&mut self, num: i32) -> Result<(), Failure> {
        if self.thestats.is_null() {
            return Ok(());
        }
        set_int(&self.thestats, num, 0)?;
        self.theflags.set_false(num, GRAPH_PRESENT)
    }
    /// `ChangeStatus(oldstat, newstat)`.
    pub fn change_status(&mut self, oldstat: i32, newstat: i32) -> Result<(), Failure> {
        if self.thestats.is_null() {
            return Ok(());
        }
        for i in 1..=int_upper(&self.thestats)? {
            if int_at(&self.thestats, i)? == oldstat {
                set_int(&self.thestats, i, newstat)?;
            }
        }
        Ok(())
    }
    /// `RemoveStatus(stat)`: removes the entities of a status.
    pub fn remove_status(&mut self, stat: i32) -> Result<(), Failure> {
        if self.thestats.is_null() {
            return Ok(());
        }
        for i in 1..=int_upper(&self.thestats)? {
            if int_at(&self.thestats, i)? == stat {
                self.remove_item(i)?;
            }
        }
        Ok(())
    }
    /// `BitMap`.
    pub fn bit_map(&self) -> &InterfaceBitMap {
        &self.theflags
    }
    /// `CBitMap`.
    pub fn c_bit_map(&mut self) -> &mut InterfaceBitMap {
        &mut self.theflags
    }
    /// `Model`.
    pub fn model(&self) -> &Handle<dyn InterfaceInterfaceModel> {
        &self.themodel
    }
    /// `ModeStat`: statuses are recorded.
    pub fn mode_stat(&self) -> bool {
        self.thestats.is_some()
    }
    pub(super) fn live_model(&self) -> Result<&Handle<dyn InterfaceInterfaceModel>, Failure> {
        if self.themodel.is_null() {
            return Err(exceptions::null_object(
                "Interface_Graph: null model; expected a model",
            ));
        }
        Ok(&self.themodel)
    }
}

/// The content of a redefined entity (its report's content), else the entity.
pub(super) fn redefined_content(
    model: &Handle<dyn InterfaceInterfaceModel>,
    number: i32,
    entity: &Handle<StandardTransient>,
) -> Handle<StandardTransient> {
    if model.is_redefined_content(number) {
        return model.report_entity(number, false).content();
    }
    entity.clone()
}

fn zero_statuses(stats: &InterfaceIntArray) -> Result<(), Failure> {
    (1..=int_upper(stats)?).try_for_each(|i| set_int(stats, i, 0))
}
