// Rust port of DataExchange/TKXSBase/IFGraph/IFGraph_SubPartsIterator (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFGraph_SubPartsIterator`: splits the entities of a graph into numbered parts (the
//! status of each entity is its part) and iterates on the non-empty parts.
use crate::interface::{
    InterfaceEntityIterator, InterfaceGraph, InterfaceGraphContent, InterfaceInterfaceModel,
    interface_error,
};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, exceptions};

/// The fields and the non-virtual members of `IFGraph_SubPartsIterator`.
pub struct IFGraphSubPartsIterator {
    /// `thegraph` (protected).
    pub thegraph: InterfaceGraph,
    theparts: Vec<i32>,
    /// `thefirsts`: `None` after the copy constructor, which natively leaves it null.
    thefirsts: Option<Vec<i32>>,
    thepart: i32,
    thecurr: i32,
}

/// The native null dereference of the `thefirsts` a copied iterator lacks.
fn no_firsts(member: &str) -> Failure {
    exceptions::null_object(&format!(
        "IFGraph_SubPartsIterator::{member}: no first entities (a copied iterator); expected an iterator built from a graph"
    ))
}

impl IFGraphSubPartsIterator {
    /// `IFGraph_SubPartsIterator(agraph, whole)`: a copy of `agraph`, every entity loaded
    /// when `whole`.
    pub fn new(agraph: &InterfaceGraph, whole: bool) -> Result<Self, Failure> {
        let mut thegraph = InterfaceGraph::from_graph(agraph, false)?;
        if whole {
            thegraph.get_from_model()?;
        }
        Ok(Self {
            thegraph,
            theparts: Vec::new(),
            thefirsts: Some(Vec::new()),
            thepart: 0,
            thecurr: 0,
        })
    }
    /// `IFGraph_SubPartsIterator(other)`: a copy of the graph of `other` with one part per
    /// part of `other` (iterated). Native defect kept: the first entities stay null, so
    /// `FirstEntity`, `Entities` and `Start` are `Standard_NullObject`.
    pub fn from_other(other: &mut dyn IFGraphSubPartsEvaluate) -> Result<Self, Failure> {
        let thegraph = InterfaceGraph::from_graph(&other.sub_parts().thegraph, false)?;
        let mut parts = Self {
            thegraph,
            theparts: Vec::new(),
            thefirsts: None,
            thepart: 0,
            thecurr: 0,
        };
        parts.append_parts_of(other)?;
        parts.thepart = 0;
        parts.thecurr = 1;
        Ok(parts)
    }
    /// One part per part of `other`, with the count of its entities.
    fn append_parts_of(&mut self, other: &mut dyn IFGraphSubPartsEvaluate) -> Result<(), Failure> {
        let nb = self.thegraph.size();
        other.start()?;
        while other.more()? {
            self.thepart += 1;
            self.get_from_iter(&other.sub_parts().entities()?)?;
            let part = self.thepart;
            let mut nbent = 0;
            for i in 1..=nb {
                nbent += i32::from(self.thegraph.status(i)? == part);
            }
            self.theparts.push(nbent);
            other.sub_parts_mut().next();
        }
        Ok(())
    }
    /// `GetParts(other)`: appends the parts of `other`; `Interface_InterfaceError` for
    /// another model.
    pub fn get_parts(&mut self, other: &mut dyn IFGraphSubPartsEvaluate) -> Result<(), Failure> {
        if self.model() != other.sub_parts().model() {
            return Err(interface_error("SubPartsIterator : GetParts"));
        }
        self.thepart = self.theparts.len() as i32;
        self.append_parts_of(other)
    }
    /// `Graph` (private natively): the graph of the parts.
    pub fn graph(&self) -> &InterfaceGraph {
        &self.thegraph
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.thegraph.model().clone()
    }
    /// `AddPart`: a new empty part, which becomes the current one.
    pub fn add_part(&mut self) {
        self.theparts.push(0);
        self.thepart = self.theparts.len() as i32;
    }
    /// `NbParts`.
    pub fn nb_parts(&self) -> i32 {
        self.theparts.len() as i32
    }
    /// `PartNum`.
    pub fn part_num(&self) -> i32 {
        self.thepart
    }
    /// `SetLoad`: the next entities are loaded, not in a part.
    pub fn set_load(&mut self) {
        self.thepart = 0;
    }
    /// `SetPartNum(num)`: `Standard_OutOfRange` out of 1..NbParts.
    pub fn set_part_num(&mut self, num: i32) -> Result<(), Failure> {
        if num <= 0 || num > self.nb_parts() {
            return Err(exceptions::out_of_range(&format!(
                "IFGraph_SubPartsIterator : SetPartNum: {num}; expected [1, {}]",
                self.nb_parts()
            )));
        }
        self.thepart = num;
        Ok(())
    }
    /// `GetFromEntity(ent, shared)`: puts `ent` (and its shareds) in the current part.
    pub fn get_from_entity(
        &mut self,
        ent: &Handle<StandardTransient>,
        shared: bool,
    ) -> Result<(), Failure> {
        self.thegraph
            .get_from_entity_overlap(ent, shared, self.thepart, self.thepart, false)
    }
    /// `GetFromIter(iter)`: puts the entities of `iter` in the current part.
    pub fn get_from_iter(&mut self, iter: &InterfaceEntityIterator) -> Result<(), Failure> {
        self.thegraph
            .get_from_iter_overlap(iter, self.thepart, self.thepart, false)
    }
    /// `Reset`: no entity loaded, no part.
    pub fn reset(&mut self) -> Result<(), Failure> {
        self.thegraph.reset()?;
        self.theparts.clear();
        self.thepart = 0;
        self.thecurr = 0;
        Ok(())
    }
    /// `Loaded`: the loaded entities not in a part (status 0).
    pub fn loaded(&self) -> Result<InterfaceGraphContent, Failure> {
        InterfaceGraphContent::from_graph_status(&self.thegraph, 0)
    }
    /// `LoadedGraph`: a new graph of the model with the loaded entities not in a part.
    pub fn loaded_graph(&self) -> Result<InterfaceGraph, Failure> {
        let mut graph = InterfaceGraph::new(&self.model(), true)?;
        for i in 1..=self.thegraph.size() {
            if self.thegraph.is_present(i)? && self.thegraph.status(i)? == 0 {
                graph.get_from_entity(&self.thegraph.entity(i)?, false, 0)?;
            }
        }
        Ok(graph)
    }
    /// `IsLoaded(ent)`.
    pub fn is_loaded(&self, ent: &Handle<StandardTransient>) -> Result<bool, Failure> {
        self.thegraph.is_present(self.thegraph.entity_number(ent))
    }
    /// `IsInPart(ent)`: loaded with a part.
    pub fn is_in_part(&self, ent: &Handle<StandardTransient>) -> Result<bool, Failure> {
        Ok(self.entity_part_num(ent)? != 0)
    }
    /// `EntityPartNum(ent)`: the part of `ent` (0 when not loaded or in no part).
    pub fn entity_part_num(&self, ent: &Handle<StandardTransient>) -> Result<i32, Failure> {
        let num = self.thegraph.entity_number(ent);
        if !self.thegraph.is_present(num)? {
            return Ok(0);
        }
        self.thegraph.status(num)
    }
    /// The part counts and first entities of `Start`, after `Evaluate`.
    pub(super) fn count_parts(&mut self) -> Result<(), Failure> {
        let nbp = self.theparts.len();
        self.thepart = self.thepart.min(nbp as i32);
        if nbp == 0 {
            self.thecurr = 1;
            return Ok(());
        }
        let (counts, firsts) = self.part_counts(nbp)?;
        let thefirsts = self.thefirsts.as_mut().ok_or_else(|| no_firsts("Start"))?;
        *thefirsts = firsts;
        self.theparts = counts;
        self.thecurr = 1;
        Ok(())
    }
    /// The count and the first entity of each part.
    fn part_counts(&self, nbp: usize) -> Result<(Vec<i32>, Vec<i32>), Failure> {
        let (mut counts, mut firsts) = (vec![0; nbp], vec![0; nbp]);
        for i in 1..=self.thegraph.size() {
            if !self.thegraph.is_present(i)? {
                continue;
            }
            let Some(slot) = usize::try_from(self.thegraph.status(i)? - 1)
                .ok()
                .filter(|p| *p < nbp)
            else {
                continue;
            };
            if counts[slot] == 0 {
                firsts[slot] = i;
            }
            counts[slot] += 1;
        }
        Ok((counts, firsts))
    }
    /// `Next`: the next non-empty part.
    pub fn next(&mut self) {
        self.thecurr += 1;
        while self.thecurr <= self.nb_parts() && self.theparts[self.thecurr as usize - 1] == 0 {
            self.thecurr += 1;
        }
    }
    /// The current part (from 0); `Standard_NoSuchObject` without one.
    fn current(&self, member: &str) -> Result<usize, Failure> {
        if self.thecurr < 1 || self.thecurr > self.nb_parts() {
            return Err(exceptions::no_such_object(&format!(
                "IFGraph_SubPartsIterator : {member}"
            )));
        }
        Ok(self.thecurr as usize - 1)
    }
    /// `IsSingle`: the current part has one entity.
    pub fn is_single(&self) -> Result<bool, Failure> {
        Ok(self.theparts[self.current("IsSingle")?] == 1)
    }
    /// `FirstEntity`: the first entity of the current part.
    pub fn first_entity(&self) -> Result<Handle<StandardTransient>, Failure> {
        let curr = self.current("FirstEntity")?;
        let firsts = self
            .thefirsts
            .as_ref()
            .ok_or_else(|| no_firsts("FirstEntity"))?;
        let nument = firsts.get(curr).copied().unwrap_or(0);
        if nument == 0 {
            return Err(exceptions::no_such_object(
                "IFGraph_SubPartsIterator : FirstEntity (current part is empty)",
            ));
        }
        self.thegraph.entity(nument)
    }
    /// `Entities`: the entities of the current part.
    pub fn entities(&self) -> Result<InterfaceEntityIterator, Failure> {
        let curr = self.current("Entities")?;
        let firsts = self
            .thefirsts
            .as_ref()
            .ok_or_else(|| no_firsts("Entities"))?;
        let mut iter = InterfaceEntityIterator::new();
        let nument = firsts.get(curr).copied().unwrap_or(0);
        if nument == 0 {
            return Ok(iter);
        }
        let nb = if self.theparts[curr] == 1 {
            nument
        } else {
            self.thegraph.size()
        };
        for i in nument..=nb {
            if self.thegraph.status(i)? == self.thecurr && self.thegraph.is_present(i)? {
                iter.get_one_item(&self.thegraph.entity(i)?);
            }
        }
        Ok(iter)
    }
}

/// The virtual `Evaluate` of `IFGraph_SubPartsIterator` and the members that call it.
pub trait IFGraphSubPartsEvaluate {
    /// The embedded iterator.
    fn sub_parts(&self) -> &IFGraphSubPartsIterator;
    /// The embedded iterator, modifiable.
    fn sub_parts_mut(&mut self) -> &mut IFGraphSubPartsIterator;
    /// `Evaluate`: nothing by default (the subclasses fill the parts).
    fn evaluate(&mut self) -> Result<(), Failure> {
        Ok(())
    }
    /// `Start`: evaluates, then counts the entities of each part.
    fn start(&mut self) -> Result<(), Failure> {
        self.evaluate()?;
        self.sub_parts_mut().count_parts()
    }
    /// `More`: starts first when not started.
    fn more(&mut self) -> Result<bool, Failure> {
        if self.sub_parts().thecurr == 0 {
            self.start()?;
        }
        let parts = self.sub_parts();
        Ok(parts.thecurr <= parts.nb_parts())
    }
}

impl IFGraphSubPartsEvaluate for IFGraphSubPartsIterator {
    fn sub_parts(&self) -> &IFGraphSubPartsIterator {
        self
    }
    fn sub_parts_mut(&mut self) -> &mut IFGraphSubPartsIterator {
        self
    }
}
