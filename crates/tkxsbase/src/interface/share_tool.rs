// Rust port of DataExchange/TKXSBase/Interface/Interface_ShareTool (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_ShareTool`: the sharing queries of a model, over a shared graph.
use super::errors::interface_error;
use super::{
    InterfaceEntityIterator, InterfaceGTool, InterfaceGeneralLib, InterfaceGraph, InterfaceHGraph,
    InterfaceInterfaceModel, InterfaceModelEntities, InterfaceProtocol,
};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeType, StandardTransient};
use std::io::{self, Write};
use std::sync::RwLockReadGuard;

/// `Interface_ShareTool`.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceShareTool;
/// fn shared(tool: &InterfaceShareTool, ent: &rustycad_tkernel::standard::Handle<rustycad_tkernel::standard::StandardTransient>) -> bool {
///     tool.is_shared(ent).unwrap_or(false)
/// }
/// ```
#[derive(Clone)]
pub struct InterfaceShareTool {
    the_h_graph: Handle<InterfaceHGraph>,
}

impl InterfaceShareTool {
    /// `Interface_ShareTool(amodel, lib)`.
    pub fn with_lib(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        lib: &InterfaceGeneralLib,
    ) -> Result<Self, Failure> {
        Ok(Self::from_h_graph(&Handle::new(InterfaceHGraph::with_lib(
            amodel, lib, true,
        )?)))
    }
    /// `Interface_ShareTool(amodel, gtool)`.
    pub fn with_gtool(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        gtool: &Handle<InterfaceGTool>,
    ) -> Result<Self, Failure> {
        Ok(Self::from_h_graph(&Handle::new(
            InterfaceHGraph::with_gtool(amodel, gtool, true)?,
        )))
    }
    /// `Interface_ShareTool(amodel, protocol)`.
    pub fn with_protocol(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<Self, Failure> {
        Ok(Self::from_h_graph(&Handle::new(
            InterfaceHGraph::with_protocol(amodel, protocol, true)?,
        )))
    }
    /// `Interface_ShareTool(amodel)`.
    pub fn new(amodel: &Handle<dyn InterfaceInterfaceModel>) -> Result<Self, Failure> {
        Ok(Self::from_h_graph(&Handle::new(InterfaceHGraph::new(
            amodel, true,
        )?)))
    }
    /// `Interface_ShareTool(agraph)`: native quirk kept, a new graph of the model of
    /// `agraph` is evaluated (the graph itself is not used).
    pub fn from_graph(agraph: &InterfaceGraph) -> Result<Self, Failure> {
        Self::new(agraph.model())
    }
    /// `Interface_ShareTool(ahgraph)`: shares the graph.
    pub fn from_h_graph(ahgraph: &Handle<InterfaceHGraph>) -> Self {
        Self {
            the_h_graph: ahgraph.clone(),
        }
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.graph().model().clone()
    }
    /// `Graph`: the graph, read only (a guard).
    pub fn graph(&self) -> RwLockReadGuard<'_, InterfaceGraph> {
        self.the_h_graph.graph()
    }
    /// `RootEntities`.
    pub fn root_entities(&self) -> Result<InterfaceEntityIterator, Failure> {
        self.graph().root_entities()
    }
    /// `IsShared(ent)`: `ent` shares at least one entity.
    pub fn is_shared(&self, ent: &Handle<StandardTransient>) -> Result<bool, Failure> {
        Ok(!self.graph().get_shareds(ent)?.is_empty())
    }
    /// `Shareds(ent)`.
    pub fn shareds(
        &self,
        ent: &Handle<StandardTransient>,
    ) -> Result<InterfaceEntityIterator, Failure> {
        self.graph().shareds(ent)
    }
    /// `Sharings(ent)`.
    pub fn sharings(
        &self,
        ent: &Handle<StandardTransient>,
    ) -> Result<InterfaceEntityIterator, Failure> {
        self.graph().sharings(ent)
    }
    /// `NbTypedSharings(ent, atype)`.
    pub fn nb_typed_sharings(
        &self,
        ent: &Handle<StandardTransient>,
        atype: &RuntimeType,
    ) -> Result<i32, Failure> {
        let list = self.graph().get_sharings(ent)?.unwrap_or_default();
        Ok(list
            .iter()
            .filter(|e| e.is_some() && e.is_kind(atype))
            .count() as i32)
    }
    /// `TypedSharing(ent, atype)`: the single sharing entity of type `atype`;
    /// `Interface_InterfaceError` when none or several (null when `ent` is out of the
    /// model).
    pub fn typed_sharing(
        &self,
        ent: &Handle<StandardTransient>,
        atype: &RuntimeType,
    ) -> Result<Handle<StandardTransient>, Failure> {
        let Some(list) = self.graph().get_sharings(ent)? else {
            return Ok(Handle::default());
        };
        let mut found = list.iter().filter(|e| e.is_some() && e.is_kind(atype));
        let first = found.next().cloned();
        if found.next().is_some() {
            return Err(interface_error(
                "Interface ShareTool : TypedSharing, more than one found",
            ));
        }
        first.ok_or_else(|| interface_error("Interface ShareTool : TypedSharing, not found"))
    }
    /// `All(ent, rootlast = true)`: `ent` and everything it shares, recursively; for the
    /// model itself, every entity root by root. The shared entities come before their
    /// sharing ones unless `rootlast` (always for the model).
    pub fn all(
        &self,
        ent: &Handle<StandardTransient>,
        rootlast: bool,
    ) -> Result<InterfaceEntityIterator, Failure> {
        let model = self.model();
        let nb = model.nb_entities();
        let is_model = *ent == model.cast::<StandardTransient>();
        let fl = if is_model {
            self.model_ranks(&model, nb, rootlast)?
        } else {
            self.entity_ranks(&model, nb, ent)?
        };
        let mut ord = vec![0; (nb + 1) as usize];
        for i in 1..=nb {
            ord[fl[i as usize] as usize] = i;
        }
        let mut list = InterfaceEntityIterator::new();
        let ranks: Vec<i32> = if rootlast && !is_model {
            (1..=nb).collect()
        } else {
            (1..=nb).rev().collect()
        };
        for i in ranks {
            if ord[i as usize] != 0 {
                list.add_item(&model.value(ord[i as usize])?);
            }
        }
        Ok(list)
    }
    /// The visit ranks of `All` for the model: the entities of each root, then the rest.
    fn model_ranks(
        &self,
        model: &Handle<dyn InterfaceInterfaceModel>,
        nb: i32,
        rootlast: bool,
    ) -> Result<Vec<i32>, Failure> {
        let mut fl = vec![0; (nb + 1) as usize];
        let mut n0 = 0;
        for root in self.root_entities()?.content() {
            for sub in self.all(&root, rootlast)?.content() {
                let nm = model.number(&sub) as usize;
                if fl[nm] > 0 {
                    continue;
                }
                n0 += 1;
                fl[nm] = n0;
            }
        }
        for slot in fl.iter_mut().skip(1).filter(|v| **v == 0) {
            n0 += 1;
            *slot = n0;
        }
        Ok(fl)
    }
    /// The visit ranks of `All` for one entity: breadth first over the shared entities.
    fn entity_ranks(
        &self,
        model: &Handle<dyn InterfaceInterfaceModel>,
        nb: i32,
        ent: &Handle<StandardTransient>,
    ) -> Result<Vec<i32>, Failure> {
        let mut fl = vec![0; (nb + 1) as usize];
        let mut n0 = 0;
        let mut sq = vec![ent.clone()];
        let mut i = 0;
        while i < sq.len() {
            let en = sq[i].clone();
            i += 1;
            let num = model.number(&en) as usize;
            if fl[num] != 0 {
                continue; // already seen
            }
            n0 += 1;
            fl[num] = n0;
            sq.extend(self.shareds(&en)?.content());
        }
        Ok(fl)
    }
    /// `Print(iter, S)`: the count, then the number and label of each entity.
    pub fn print(&self, iter: &InterfaceEntityIterator, s: &mut dyn Write) -> io::Result<()> {
        write!(s, " Nb.Entities : {} : ", iter.nb_entities())?;
        let model = self.model();
        for ent in iter.content() {
            s.write_all(b" n0/id:")?;
            if let Some(live) = model.get() {
                live.print(&ent, s, 0)?;
            }
        }
        s.write_all(b"\n")
    }
}
