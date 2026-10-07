// Rust port of DataExchange/TKXSBase/IFGraph/IFGraph_AllShared, IFGraph_Compare,
// IFGraph_Cumulate and IFGraph_ExternalSources (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The graph contents of IFGraph: the entities shared by others (`AllShared`), the
//! comparison of two lists (`Compare`), the count of selections of each entity
//! (`Cumulate`) and the entities that share a list from outside (`ExternalSources`).
use crate::interface::{
    InterfaceEntityIterator, InterfaceGraph, InterfaceGraphContent, InterfaceGraphContentEvaluate,
    InterfaceModelEntities,
};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, exceptions};
use std::ops::{Deref, DerefMut};

/// Declares a graph content of IFGraph: its content (the dereference target) and its
/// copy of the graph.
macro_rules! graph_content_tool {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        pub struct $name {
            content: InterfaceGraphContent,
            thegraph: InterfaceGraph,
        }
        impl Deref for $name {
            type Target = InterfaceGraphContent;
            fn deref(&self) -> &InterfaceGraphContent {
                &self.content
            }
        }
        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut InterfaceGraphContent {
                &mut self.content
            }
        }
        impl $name {
            /// The constructor from a graph: an empty content and a copy of `agraph`.
            pub fn new(agraph: &InterfaceGraph) -> Result<Self, Failure> {
                Ok(Self { content: InterfaceGraphContent::new(), thegraph: InterfaceGraph::from_graph(agraph, false)? })
            }
            /// `ResetData`: an empty content and no entity in the graph.
            pub fn reset_data(&mut self) -> Result<(), Failure> {
                self.content.reset();
                self.thegraph.reset()
            }
        }
    };
}

graph_content_tool!(
    /// `IFGraph_AllShared`: an entity (or a list) with all the entities it shares.
    ///
    /// ```
    /// use rustycad_tkxsbase::if_graph::IFGraphAllShared;
    /// use rustycad_tkxsbase::interface::{InterfaceGraph, InterfaceGraphContentEvaluate};
    /// fn shared(graph: &InterfaceGraph, ent: &rustycad_tkernel::standard::Handle<rustycad_tkernel::standard::StandardTransient>) -> i32 {
    ///     let mut all = IFGraphAllShared::with_entity(graph, ent).unwrap();
    ///     all.result().unwrap().nb_entities()
    /// }
    /// ```
    IFGraphAllShared
);

impl IFGraphAllShared {
    /// `IFGraph_AllShared(agraph, ent)`: `ent` and its shareds, when `ent` is in the
    /// model; `Standard_NullObject` for a graph without model (a native null dereference).
    pub fn with_entity(
        agraph: &InterfaceGraph,
        ent: &Handle<StandardTransient>,
    ) -> Result<Self, Failure> {
        let mut all = Self::new(agraph)?;
        let model = agraph.model().get().ok_or_else(|| {
            exceptions::null_object("IFGraph_AllShared: graph without model; expected a model")
        })?;
        if model.contains(ent) {
            all.get_from_entity(ent)?;
        }
        Ok(all)
    }
    /// `GetFromEntity(ent)`: `ent` and its shareds.
    pub fn get_from_entity(&mut self, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        self.thegraph.get_from_entity(ent, true, 0)
    }
    /// `GetFromIter(iter)`: the entities of `iter` and their shareds.
    pub fn get_from_iter(&mut self, iter: &InterfaceEntityIterator) -> Result<(), Failure> {
        iter.content()
            .iter()
            .try_for_each(|ent| self.thegraph.get_from_entity(ent, true, 0))
    }
}

impl InterfaceGraphContentEvaluate for IFGraphAllShared {
    fn graph_content(&mut self) -> &mut InterfaceGraphContent {
        &mut self.content
    }
    /// `Evaluate`: the entities marked in the graph.
    fn evaluate(&mut self) -> Result<(), Failure> {
        self.content.reset();
        self.content.get_from_graph(&self.thegraph)
    }
}

/// The `IFGraph_AllShared(Model(), ent)` the native `GetFromEntity` of `Compare` and
/// `Cumulate` builds through the implicit graph of a model. Native defect kept: it is
/// passed as a plain entity iterator, whose list stays empty, so nothing is added.
fn implicit_all_shared(
    agraph: &InterfaceGraph,
    ent: &Handle<StandardTransient>,
) -> Result<IFGraphAllShared, Failure> {
    let model_graph = InterfaceGraph::new(agraph.model(), true)?;
    IFGraphAllShared::with_entity(&model_graph, ent)
}

graph_content_tool!(
    /// `IFGraph_Compare`: two lists of entities (with their shareds), their common part
    /// and their specific parts (statuses 1: first, 2: second, 3: both).
    IFGraphCompare
);

impl IFGraphCompare {
    /// `GetFromEntity(ent, first)`. Native defect kept: nothing is added (see the
    /// implicit `IFGraph_AllShared` it builds).
    pub fn get_from_entity(
        &mut self,
        ent: &Handle<StandardTransient>,
        first: bool,
    ) -> Result<(), Failure> {
        let iter = implicit_all_shared(&self.thegraph, ent)?;
        self.get_from_iter(&iter, first)
    }
    /// `GetFromIter(iter, first)`: the entities of `iter` in the first or second list.
    pub fn get_from_iter(
        &mut self,
        iter: &InterfaceEntityIterator,
        first: bool,
    ) -> Result<(), Failure> {
        let stat = if first { 1 } else { 2 };
        self.thegraph.get_from_iter_overlap(iter, stat, 3, false)
    }
    /// `Merge`: everything in the first list.
    pub fn merge(&mut self) -> Result<(), Failure> {
        self.thegraph.change_status(2, 1)?;
        self.thegraph.change_status(3, 1)
    }
    /// `RemoveSecond`: the first list without the second.
    pub fn remove_second(&mut self) -> Result<(), Failure> {
        self.thegraph.change_status(3, 1)?;
        self.thegraph.remove_status(2)
    }
    /// `KeepCommon`: the common part only.
    pub fn keep_common(&mut self) -> Result<(), Failure> {
        self.thegraph.remove_status(1)?;
        self.thegraph.remove_status(2)?;
        self.thegraph.change_status(3, 1)
    }
    /// `Common`: the entities of both lists.
    pub fn common(&self) -> Result<InterfaceEntityIterator, Failure> {
        Ok(InterfaceGraphContent::from_graph_status(&self.thegraph, 3)?
            .deref()
            .clone())
    }
    /// `FirstOnly`.
    pub fn first_only(&self) -> Result<InterfaceEntityIterator, Failure> {
        Ok(InterfaceGraphContent::from_graph_status(&self.thegraph, 1)?
            .deref()
            .clone())
    }
    /// `SecondOnly`.
    pub fn second_only(&self) -> Result<InterfaceEntityIterator, Failure> {
        Ok(InterfaceGraphContent::from_graph_status(&self.thegraph, 2)?
            .deref()
            .clone())
    }
}

impl InterfaceGraphContentEvaluate for IFGraphCompare {
    fn graph_content(&mut self) -> &mut InterfaceGraphContent {
        &mut self.content
    }
    /// `Evaluate`: the entities of both lists.
    fn evaluate(&mut self) -> Result<(), Failure> {
        self.content.reset();
        self.content.get_from_graph(&self.thegraph)
    }
}

graph_content_tool!(
    /// `IFGraph_Cumulate`: counts how many times each entity is selected (status = count
    /// plus 1).
    IFGraphCumulate
);

impl IFGraphCumulate {
    /// `GetFromEntity(ent)`. Native defect kept: nothing is added (see the implicit
    /// `IFGraph_AllShared` it builds).
    pub fn get_from_entity(&mut self, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        let iter = implicit_all_shared(&self.thegraph, ent)?;
        self.get_from_iter(&iter)
    }
    /// `GetFromIter(iter)`: one more selection of each entity of `iter`.
    pub fn get_from_iter(&mut self, iter: &InterfaceEntityIterator) -> Result<(), Failure> {
        self.thegraph.get_from_iter_overlap(iter, 1, 1, true)?;
        self.thegraph.change_status(1, 2)
    }
    /// The entities (as the graph gives them) present with a status matching `keep`.
    fn present_with(&self, keep: impl Fn(i32) -> bool) -> Result<InterfaceEntityIterator, Failure> {
        let mut iter = InterfaceEntityIterator::new();
        for i in 1..=self.thegraph.size() {
            if self.thegraph.is_present(i)? && keep(self.thegraph.status(i)?) {
                iter.get_one_item(&self.thegraph.entity(i)?);
            }
        }
        Ok(iter)
    }
    /// The live model of the graph (a native null dereference without one).
    fn live_model(&self) -> Result<&dyn crate::interface::InterfaceInterfaceModel, Failure> {
        self.thegraph.model().get().ok_or_else(|| {
            exceptions::null_object("IFGraph_Cumulate: graph without model; expected a model")
        })
    }
    /// `Overlapped`: the entities selected more than once.
    pub fn overlapped(&self) -> Result<InterfaceEntityIterator, Failure> {
        self.present_with(|stat| stat > 2)
    }
    /// `Forgotten`: the entities of the model never selected.
    pub fn forgotten(&self) -> Result<InterfaceEntityIterator, Failure> {
        let model = self.live_model()?;
        let mut iter = InterfaceEntityIterator::new();
        for i in 1..=self.thegraph.size() {
            if !self.thegraph.is_present(i)? {
                iter.get_one_item(&model.value(i)?);
            }
        }
        Ok(iter)
    }
    /// `PerCount(count = 1)`: the entities selected exactly `count` times.
    pub fn per_count(&self, count: i32) -> Result<InterfaceEntityIterator, Failure> {
        let model = self.live_model()?;
        let mut iter = InterfaceEntityIterator::new();
        for i in 1..=self.thegraph.size() {
            if self.thegraph.is_present(i)? && self.thegraph.status(i)? == count + 1 {
                iter.get_one_item(&model.value(i)?);
            }
        }
        Ok(iter)
    }
    /// `NbTimes(ent)`: the count of selections of `ent` (0 out of the model; -1 when
    /// loaded but never selected, as natively).
    pub fn nb_times(&self, ent: &Handle<StandardTransient>) -> Result<i32, Failure> {
        let num = self.thegraph.entity_number(ent);
        if num == 0 {
            return Ok(0);
        }
        Ok(self.thegraph.status(num)? - 1)
    }
    /// `HighestNbTimes`.
    pub fn highest_nb_times(&self) -> Result<i32, Failure> {
        let mut max = 0;
        for i in 1..=self.thegraph.size() {
            if self.thegraph.is_present(i)? {
                max = max.max(self.thegraph.status(i)? - 1);
            }
        }
        Ok(max)
    }
}

impl InterfaceGraphContentEvaluate for IFGraphCumulate {
    fn graph_content(&mut self) -> &mut InterfaceGraphContent {
        &mut self.content
    }
    /// `Evaluate`: the selected entities.
    fn evaluate(&mut self) -> Result<(), Failure> {
        self.content.reset();
        self.content.get_from_graph(&self.thegraph)
    }
}

graph_content_tool!(
    /// `IFGraph_ExternalSources`: the entities which share a list from outside it.
    IFGraphExternalSources
);

impl IFGraphExternalSources {
    /// `GetFromEntity(ent)`: `ent` and its shareds form the list.
    pub fn get_from_entity(&mut self, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        self.thegraph.get_from_entity(ent, true, 0)
    }
    /// `GetFromIter(iter)`: the entities of `iter` form the list.
    pub fn get_from_iter(&mut self, iter: &InterfaceEntityIterator) -> Result<(), Failure> {
        self.thegraph.get_from_iter(iter, 0)
    }
    /// `IsEmpty`: evaluates, then looks for an entity present or of status 1. Native quirk
    /// kept: any entity of the list is present, so a non-empty list is never empty.
    pub fn is_empty(&mut self) -> Result<bool, Failure> {
        self.evaluate()?;
        for i in 1..=self.thegraph.size() {
            if self.thegraph.is_present(i)? || self.thegraph.status(i)? == 1 {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl InterfaceGraphContentEvaluate for IFGraphExternalSources {
    fn graph_content(&mut self) -> &mut InterfaceGraphContent {
        &mut self.content
    }
    /// `Evaluate`: the sharings of the list (status 0), marked with status 1.
    fn evaluate(&mut self) -> Result<(), Failure> {
        self.content.reset();
        self.thegraph.remove_status(1)?;
        for i in 1..=self.thegraph.size() {
            if self.thegraph.is_present(i)? && self.thegraph.status(i)? == 0 {
                let sharings = self.thegraph.sharings(&self.thegraph.entity(i)?)?;
                self.thegraph.get_from_iter(&sharings, 1)?;
            }
        }
        self.content.get_from_graph_status(&self.thegraph, 1)
    }
}
