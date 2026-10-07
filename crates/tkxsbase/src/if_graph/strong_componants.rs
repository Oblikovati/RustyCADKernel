// Rust port of DataExchange/TKXSBase/IFGraph/IFGraph_StrongComponants and IFGraph_SCRoots
// (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFGraph_StrongComponants`: one part per loaded entity; `IFGraph_SCRoots`: the
//! components not reached from a previous one.
use super::{IFGraphSubPartsEvaluate, IFGraphSubPartsIterator};
use crate::interface::InterfaceGraph;
use rustycad_tkernel::standard::Failure;
use std::io::Write;
use std::ops::{Deref, DerefMut};

/// `IFGraph_StrongComponants`. Native behaviour kept: each loaded entity is its own
/// component (no cycle is searched).
///
/// ```
/// use rustycad_tkxsbase::if_graph::IFGraphStrongComponants;
/// fn count(graph: &rustycad_tkxsbase::interface::InterfaceGraph) -> i32 {
///     let mut comps = IFGraphStrongComponants::new(graph, true).unwrap();
///     use rustycad_tkxsbase::if_graph::IFGraphSubPartsEvaluate;
///     comps.start().unwrap();
///     comps.nb_parts()
/// }
/// ```
pub struct IFGraphStrongComponants {
    base: IFGraphSubPartsIterator,
}

impl Deref for IFGraphStrongComponants {
    type Target = IFGraphSubPartsIterator;
    fn deref(&self) -> &IFGraphSubPartsIterator {
        &self.base
    }
}

impl DerefMut for IFGraphStrongComponants {
    fn deref_mut(&mut self) -> &mut IFGraphSubPartsIterator {
        &mut self.base
    }
}

impl IFGraphStrongComponants {
    /// `IFGraph_StrongComponants(agraph, whole)`.
    pub fn new(agraph: &InterfaceGraph, whole: bool) -> Result<Self, Failure> {
        Ok(Self {
            base: IFGraphSubPartsIterator::new(agraph, whole)?,
        })
    }
    /// The implicit copy constructor: the iterating copy of `IFGraph_SubPartsIterator`.
    pub fn from_other(other: &mut dyn IFGraphSubPartsEvaluate) -> Result<Self, Failure> {
        Ok(Self {
            base: IFGraphSubPartsIterator::from_other(other)?,
        })
    }
}

/// `StrongComponants::Evaluate` on an iterator: a part per loaded entity.
fn evaluate_components(parts: &mut IFGraphSubPartsIterator) -> Result<(), Failure> {
    let iter = parts.loaded()?;
    let mut graph = InterfaceGraph::from_graph(&parts.thegraph, false)?;
    graph.get_from_iter(&iter, 0)?;
    for i in 1..=graph.size() {
        if !graph.is_present(i)? {
            continue;
        }
        parts.add_part();
        parts.get_from_entity(&graph.entity(i)?, false)?;
    }
    Ok(())
}

impl IFGraphSubPartsEvaluate for IFGraphStrongComponants {
    fn sub_parts(&self) -> &IFGraphSubPartsIterator {
        &self.base
    }
    fn sub_parts_mut(&mut self) -> &mut IFGraphSubPartsIterator {
        &mut self.base
    }
    /// `Evaluate`: a part per entity loaded and not in a part.
    fn evaluate(&mut self) -> Result<(), Failure> {
        evaluate_components(&mut self.base)
    }
}

/// `IFGraph_SCRoots`: embeds its `IFGraph_StrongComponants` (dereferences to it).
pub struct IFGraphSCRoots {
    base: IFGraphStrongComponants,
}

impl Deref for IFGraphSCRoots {
    type Target = IFGraphStrongComponants;
    fn deref(&self) -> &IFGraphStrongComponants {
        &self.base
    }
}

impl DerefMut for IFGraphSCRoots {
    fn deref_mut(&mut self) -> &mut IFGraphStrongComponants {
        &mut self.base
    }
}

impl IFGraphSCRoots {
    /// `IFGraph_SCRoots(agraph, whole)`.
    pub fn new(agraph: &InterfaceGraph, whole: bool) -> Result<Self, Failure> {
        Ok(Self {
            base: IFGraphStrongComponants::new(agraph, whole)?,
        })
    }
    /// `IFGraph_SCRoots(subparts)`: the iterating copy of `subparts`.
    pub fn from_components(subparts: &mut IFGraphStrongComponants) -> Result<Self, Failure> {
        Ok(Self {
            base: IFGraphStrongComponants::from_other(subparts)?,
        })
    }
}

impl IFGraphSubPartsEvaluate for IFGraphSCRoots {
    fn sub_parts(&self) -> &IFGraphSubPartsIterator {
        &self.base
    }
    fn sub_parts_mut(&mut self) -> &mut IFGraphSubPartsIterator {
        &mut self.base
    }
    /// `Evaluate`: a part per component whose first entity is not reached from a previous
    /// one. Native behaviour kept: the trace lines go to the standard output.
    fn evaluate(&mut self) -> Result<(), Failure> {
        let model_graph = InterfaceGraph::new(&self.model(), true)?;
        let mut complist = IFGraphStrongComponants::new(&model_graph, false)?;
        complist.get_from_iter(&*self.loaded()?)?;
        let mut graph = InterfaceGraph::from_graph(&self.thegraph, false)?;
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, " SCRoots:");
        graph.reset_status()?;
        complist.start()?;
        while complist.more()? {
            self.take_component(&mut graph, &complist, &mut out)?;
            complist.next();
        }
        Ok(())
    }
}

impl IFGraphSCRoots {
    /// One iteration of `Evaluate`: the current component of `complist` becomes a part
    /// when its first entity is not yet reached in `graph`.
    fn take_component(
        &mut self,
        graph: &mut InterfaceGraph,
        complist: &IFGraphStrongComponants,
        out: &mut dyn Write,
    ) -> Result<(), Failure> {
        let ent = complist.first_entity()?;
        let num = graph.entity_number(&ent);
        let present = graph.is_present(num)?;
        let _ = writeln!(
            out,
            "   Iteration,num={num}{}",
            if present { " Taken" } else { " To take" }
        );
        if present {
            return Ok(());
        }
        graph.get_from_entity(&ent, true, 0)?;
        let list = complist.entities()?;
        self.add_part();
        self.get_from_iter(&list)
    }
}
