// Rust port of DataExchange/TKXSBase/Interface/Interface_Graph.cxx (GetFromModel,
// GetFromEntity, GetFromIter, GetFromGraph) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The members that mark entities present in an `Interface_Graph`, with a status.
use super::graph::GRAPH_PRESENT;
use super::int_array::{int_at, set_int};
use super::{InterfaceEntityIterator, InterfaceGraph};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, exceptions};

impl InterfaceGraph {
    /// `GetFromModel`: every entity present with status 0.
    pub fn get_from_model(&mut self) -> Result<(), Failure> {
        if self.themodel.is_null() || self.thestats.is_null() {
            return Ok(()); // no model ... (-> we won't go far)
        }
        self.theflags.init(true, GRAPH_PRESENT)?;
        let upper = super::int_array::int_upper(&self.thestats)?;
        (1..=upper).try_for_each(|i| set_int(&self.thestats, i, 0))
    }
    /// `GetFromEntity(ent, shared, newstat = 0)`: marks `ent` (and, with `shared`, the
    /// entities it shares, recursively) present with `newstat`; an entity already present
    /// is skipped with its sub-tree.
    pub fn get_from_entity(
        &mut self,
        ent: &Handle<StandardTransient>,
        shared: bool,
        newstat: i32,
    ) -> Result<(), Failure> {
        if self.thestats.is_null() {
            return Ok(());
        }
        let num = self.entity_number(ent);
        if num == 0 || self.theflags.c_true(num, GRAPH_PRESENT)? {
            return Ok(()); // already taken : we skip
        }
        set_int(&self.thestats, num, newstat)?;
        if !shared {
            return Ok(());
        }
        for sub in self.get_shareds(ent)? {
            self.get_from_entity(&sub, true, newstat)?;
        }
        Ok(())
    }
    /// `GetFromEntity(ent, shared, newstat, overlapstat, cumul)`: as above, but an entity
    /// already present with another status gets `overlapstat` (added to its status with
    /// `cumul`). Native quirk kept: the shared entities are added with the simple form.
    pub fn get_from_entity_overlap(
        &mut self,
        ent: &Handle<StandardTransient>,
        shared: bool,
        newstat: i32,
        overlapstat: i32,
        cumul: bool,
    ) -> Result<(), Failure> {
        if self.thestats.is_null() {
            return Ok(());
        }
        let num = self.entity_number(ent);
        if num == 0 {
            return Ok(());
        }
        let pasla = !self.theflags.c_true(num, GRAPH_PRESENT)?;
        let stat = int_at(&self.thestats, num)?;
        if pasla {
            set_int(&self.thestats, num, newstat)?;
        } else if stat != newstat {
            let overstat = if cumul {
                stat + overlapstat
            } else {
                overlapstat
            };
            if stat != overstat {
                set_int(&self.thestats, num, overstat)?;
            }
        }
        if !shared {
            return Ok(());
        }
        for sub in self.get_shareds(ent)? {
            self.get_from_entity(&sub, true, newstat)?;
        }
        Ok(())
    }
    /// `GetFromIter(iter, newstat)`: marks the entities of `iter` (not their shareds).
    pub fn get_from_iter(
        &mut self,
        iter: &InterfaceEntityIterator,
        newstat: i32,
    ) -> Result<(), Failure> {
        if self.thestats.is_null() {
            return Ok(());
        }
        for ent in iter.content() {
            let num = self.entity_number(&ent);
            if num == 0 || self.theflags.c_true(num, GRAPH_PRESENT)? {
                continue;
            }
            set_int(&self.thestats, num, newstat)?;
        }
        Ok(())
    }
    /// `GetFromIter(iter, newstat, overlapstat, cumul)`.
    pub fn get_from_iter_overlap(
        &mut self,
        iter: &InterfaceEntityIterator,
        newstat: i32,
        overlapstat: i32,
        cumul: bool,
    ) -> Result<(), Failure> {
        if self.thestats.is_null() {
            return Ok(());
        }
        for ent in iter.content() {
            let num = self.entity_number(&ent);
            if num == 0 {
                continue;
            }
            // The native code reads the flag and the status here without using them.
            self.theflags.value(num, GRAPH_PRESENT)?;
            int_at(&self.thestats, num)?;
            self.get_from_entity_overlap(&ent, false, newstat, overlapstat, cumul)?;
        }
        Ok(())
    }
    /// `GetFromGraph(agraph)`: the entities present in `agraph`, with their statuses;
    /// `Standard_DomainError` for a graph of another model.
    pub fn get_from_graph(&mut self, agraph: &InterfaceGraph) -> Result<(), Failure> {
        self.require_same_model(agraph)?;
        for i in 1..=self.size() {
            if agraph.is_present(i)? {
                let status = agraph.status(i)?;
                self.get_from_entity(&agraph.entity(i)?, false, status)?;
            }
        }
        Ok(())
    }
    /// `GetFromGraph(agraph, stat)`: the entities present in `agraph` with status `stat`.
    pub fn get_from_graph_status(
        &mut self,
        agraph: &InterfaceGraph,
        stat: i32,
    ) -> Result<(), Failure> {
        self.require_same_model(agraph)?;
        for i in 1..=self.size() {
            if agraph.is_present(i)? && agraph.status(i)? == stat {
                self.get_from_entity(&agraph.entity(i)?, false, stat)?;
            }
        }
        Ok(())
    }
    fn require_same_model(&self, agraph: &InterfaceGraph) -> Result<(), Failure> {
        if self.themodel != agraph.themodel {
            return Err(exceptions::domain_error(
                "Graph from Interface : GetFromGraph: the graphs have different models; expected the same model",
            ));
        }
        Ok(())
    }
}
