// Rust port of DataExchange/TKXSBase/Interface/Interface_GraphContent (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_GraphContent`: an entity iterator filled from a graph, whose content can be
//! evaluated on demand (`Evaluate`, redefined by the IFGraph tools).
use super::{InterfaceEntityIterator, InterfaceGraph};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use std::ops::{Deref, DerefMut};

/// `Interface_GraphContent`: embeds its `Interface_EntityIterator` (dereferences to it).
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceGraphContent, InterfaceGraphContentEvaluate};
/// let mut content = InterfaceGraphContent::new();
/// assert_eq!(content.result().unwrap().nb_entities(), 0);
/// ```
#[derive(Clone, Default)]
pub struct InterfaceGraphContent {
    iter: InterfaceEntityIterator,
}

impl Deref for InterfaceGraphContent {
    type Target = InterfaceEntityIterator;
    fn deref(&self) -> &InterfaceEntityIterator {
        &self.iter
    }
}

impl DerefMut for InterfaceGraphContent {
    fn deref_mut(&mut self) -> &mut InterfaceEntityIterator {
        &mut self.iter
    }
}

impl InterfaceGraphContent {
    /// `Interface_GraphContent()`: empty.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Interface_GraphContent(agraph)`: the entities present in the graph.
    pub fn from_graph(agraph: &InterfaceGraph) -> Result<Self, Failure> {
        let mut content = Self::new();
        content.get_from_graph(agraph)?;
        Ok(content)
    }
    /// `Interface_GraphContent(agraph, stat)`: the present entities of status `stat`.
    pub fn from_graph_status(agraph: &InterfaceGraph, stat: i32) -> Result<Self, Failure> {
        let mut content = Self::new();
        content.get_from_graph_status(agraph, stat)?;
        Ok(content)
    }
    /// `Interface_GraphContent(agraph, ent)`: the entities `ent` shares that are present
    /// in the graph.
    pub fn from_entity(
        agraph: &InterfaceGraph,
        ent: &Handle<StandardTransient>,
    ) -> Result<Self, Failure> {
        let mut content = Self::new();
        for curent in agraph.shareds(ent)?.content() {
            if agraph.is_present(agraph.entity_number(&curent))? {
                content.get_one_item(&curent);
            }
        }
        Ok(content)
    }
    /// `GetFromGraph(agraph)`: adds the entities present in the graph.
    pub fn get_from_graph(&mut self, agraph: &InterfaceGraph) -> Result<(), Failure> {
        for i in 1..=agraph.size() {
            if agraph.is_present(i)? {
                self.get_one_item(&agraph.entity(i)?);
            }
        }
        Ok(())
    }
    /// `GetFromGraph(agraph, stat)`: adds the present entities of status `stat`.
    pub fn get_from_graph_status(
        &mut self,
        agraph: &InterfaceGraph,
        stat: i32,
    ) -> Result<(), Failure> {
        for i in 1..=agraph.size() {
            if agraph.is_present(i)? && agraph.status(i)? == stat {
                self.get_one_item(&agraph.entity(i)?);
            }
        }
        Ok(())
    }
}

/// The virtual `Evaluate` of `Interface_GraphContent` and the members that call it.
pub trait InterfaceGraphContentEvaluate {
    /// The embedded graph content.
    fn graph_content(&mut self) -> &mut InterfaceGraphContent;
    /// `Evaluate`: nothing by default.
    fn evaluate(&mut self) -> Result<(), Failure> {
        Ok(())
    }
    /// `Begin`: evaluates, then starts the iteration.
    fn begin(&mut self) -> Result<(), Failure> {
        self.evaluate()?;
        self.graph_content().start();
        Ok(())
    }
    /// `Result`: a new iterator on the evaluated content.
    fn result(&mut self) -> Result<InterfaceEntityIterator, Failure> {
        self.begin()?;
        let mut iter = InterfaceEntityIterator::new();
        let content = self.graph_content();
        while content.more() {
            iter.get_one_item(&content.value()?);
            content.next();
        }
        Ok(iter)
    }
}

impl InterfaceGraphContentEvaluate for InterfaceGraphContent {
    fn graph_content(&mut self) -> &mut InterfaceGraphContent {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_norm::fake_chain;
    use super::*;

    #[test]
    fn contents_from_graphs() {
        let (model, ents) = fake_chain();
        let mut graph = InterfaceGraph::new(&model, true).unwrap();
        graph.get_from_entity(&ents[2], true, 5).unwrap();
        let mut all = InterfaceGraphContent::from_graph(&graph).unwrap();
        assert_eq!(all.result().unwrap().nb_entities(), 3);
        assert_eq!(
            InterfaceGraphContent::from_graph_status(&graph, 5)
                .unwrap()
                .nb_entities(),
            3
        );
        assert_eq!(
            InterfaceGraphContent::from_graph_status(&graph, 1)
                .unwrap()
                .nb_entities(),
            0
        );
        let shared = InterfaceGraphContent::from_entity(&graph, &ents[2]).unwrap();
        assert_eq!(shared.nb_entities(), 2);
        assert_eq!(
            InterfaceGraphContent::from_entity(&graph, &ents[3])
                .unwrap()
                .nb_entities(),
            1
        );
    }
}
