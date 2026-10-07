// Rust port of DataExchange/TKXSBase/Interface/Interface_ShareFlags (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_ShareFlags`: which entities of a model are shared by another one, and the
//! roots (the entities nothing shares).
use super::graph::redefined_content;
use super::{
    InterfaceBitMap, InterfaceEntityIterator, InterfaceGTool, InterfaceGeneralLib, InterfaceGraph,
    InterfaceInterfaceModel, InterfaceModelEntities, InterfaceProtocol,
};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, exceptions};
use std::sync::Arc;

/// `Interface_ShareFlags`.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceShareFlags;
/// fn roots(flags: &InterfaceShareFlags) -> i32 { flags.nb_roots() }
/// ```
pub struct InterfaceShareFlags {
    themodel: Handle<dyn InterfaceInterfaceModel>,
    theflags: InterfaceBitMap,
    theroots: Option<Vec<Handle<StandardTransient>>>,
}

impl InterfaceShareFlags {
    /// `Interface_ShareFlags(amodel, lib)`.
    pub fn with_lib(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        lib: &InterfaceGeneralLib,
    ) -> Result<Self, Failure> {
        let mut flags = Self::empty(amodel)?;
        flags.evaluate(lib, &Handle::default())?;
        Ok(flags)
    }
    /// `Interface_ShareFlags(amodel, gtool)`: with the library and the cache of the GTool.
    pub fn with_gtool(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        gtool: &Handle<InterfaceGTool>,
    ) -> Result<Self, Failure> {
        let lib = live_gtool(gtool)?.lib();
        let mut flags = Self::empty(amodel)?;
        flags.evaluate(&lib, gtool)?;
        Ok(flags)
    }
    /// `Interface_ShareFlags(amodel, protocol)`.
    pub fn with_protocol(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<Self, Failure> {
        Self::with_lib(amodel, &InterfaceGeneralLib::new(protocol))
    }
    /// `Interface_ShareFlags(amodel)`. Native defect kept as a failure: the constructor
    /// reads the GTool of its model field before it sets it (a null dereference), so it
    /// always raises `Standard_NullObject`.
    pub fn new(amodel: &Handle<dyn InterfaceInterfaceModel>) -> Result<Self, Failure> {
        let _ = amodel;
        Err(exceptions::null_object(
            "Interface_ShareFlags(amodel): the native constructor reads the GTool of its model field before setting it",
        ))
    }
    /// `Interface_ShareFlags(agraph)`: the entities with sharings in the graph are shared.
    pub fn from_graph(agraph: &InterfaceGraph) -> Result<Self, Failure> {
        let mut flags = Self::empty(agraph.model())?;
        let nb = flags.themodel.nb_entities();
        if nb == 0 {
            return Ok(flags);
        }
        let mut roots = Vec::new();
        for i in 1..=nb {
            let ent = flags.themodel.value(i)?;
            if agraph.get_sharings(&ent)?.is_some_and(|l| !l.is_empty()) {
                flags.theflags.set_true(i, 0)?;
            } else {
                roots.push(ent);
            }
        }
        flags.theroots = Some(roots);
        Ok(flags)
    }
    fn empty(amodel: &Handle<dyn InterfaceInterfaceModel>) -> Result<Self, Failure> {
        let nb = live_model(amodel)?.nb_entities();
        Ok(Self {
            themodel: amodel.clone(),
            theflags: InterfaceBitMap::with_items(nb, 0)?,
            theroots: None,
        })
    }
    /// `Evaluate` (private): flags every shared entity, then lists the roots.
    fn evaluate(
        &mut self,
        lib: &InterfaceGeneralLib,
        gtool: &Handle<InterfaceGTool>,
    ) -> Result<(), Failure> {
        let nb = self.themodel.nb_entities();
        if nb == 0 {
            return Ok(());
        }
        for i in 1..=nb {
            let entity = self.themodel.value(i)?;
            let ent = redefined_content(&self.themodel, i, &entity);
            let selected = match gtool.get() {
                None => lib.select(&ent),
                Some(live) => live.select(&ent, false),
            };
            let mut iter = InterfaceEntityIterator::new();
            if let Some((module, cn)) = selected {
                module.fill_shared(&self.themodel, cn, &ent, &mut iter);
            }
            for shared in iter.content() {
                self.theflags.set_true(self.themodel.number(&shared), 0)?; // Et Voila
            }
        }
        let mut roots = Vec::new();
        for i in 1..=nb {
            if !self.theflags.value(i, 0)? {
                roots.push(self.themodel.value(i)?);
            }
        }
        self.theroots = Some(roots);
        Ok(())
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.themodel.clone()
    }
    /// `IsShared(ent)`: `Standard_DomainError` for an entity out of the model.
    pub fn is_shared(&self, ent: &Handle<StandardTransient>) -> Result<bool, Failure> {
        let num = self.themodel.number(ent);
        if num == 0 || num > self.themodel.nb_entities() {
            return Err(exceptions::domain_error(
                "Interface ShareFlags : IsShared: the entity is not in the model; expected one of its entities",
            ));
        }
        self.theflags.value(num, 0)
    }
    /// `RootEntities`: an iterator on the roots (sharing the root list).
    pub fn root_entities(&self) -> InterfaceEntityIterator {
        match &self.theroots {
            Some(roots) => {
                InterfaceEntityIterator::from_list(&Arc::new(SharedState::new(roots.clone())))
            }
            None => InterfaceEntityIterator::new(),
        }
    }
    /// `NbRoots`.
    pub fn nb_roots(&self) -> i32 {
        self.theroots.as_ref().map_or(0, |r| r.len() as i32)
    }
    /// `Root(num = 1)`: `Standard_OutOfRange` out of the roots.
    pub fn root(&self, num: i32) -> Result<Handle<StandardTransient>, Failure> {
        let roots = self.theroots.as_deref().unwrap_or_default();
        let found = usize::try_from(num - 1).ok().and_then(|i| roots.get(i));
        found.cloned().ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "Interface_ShareFlags::Root: {num}; expected [1, {}]",
                roots.len()
            ))
        })
    }
}

fn live_model(
    model: &Handle<dyn InterfaceInterfaceModel>,
) -> Result<&dyn InterfaceInterfaceModel, Failure> {
    model.get().ok_or_else(|| {
        exceptions::null_object("Interface_ShareFlags: null model; expected a model")
    })
}

fn live_gtool(gtool: &Handle<InterfaceGTool>) -> Result<&InterfaceGTool, Failure> {
    gtool.get().ok_or_else(|| {
        exceptions::null_object("Interface_ShareFlags: null GTool; expected a GTool")
    })
}

#[cfg(test)]
mod tests {
    use super::super::test_norm::{fake_chain, fake_entity, fake_protocol};
    use super::*;

    #[test]
    fn flags_and_roots() {
        let (model, ents) = fake_chain();
        let flags = InterfaceShareFlags::with_protocol(&model, &fake_protocol()).unwrap();
        assert!(flags.is_shared(&ents[0]).unwrap() && !flags.is_shared(&ents[3]).unwrap());
        assert_eq!(flags.nb_roots(), 1);
        assert!(flags.root(1).unwrap() == ents[3]);
        assert!(flags.root(2).is_err());
        assert_eq!(flags.root_entities().nb_entities(), 1);
        assert!(flags.is_shared(&fake_entity("x", &[])).is_err());
        let by_gtool = InterfaceShareFlags::with_gtool(&model, &model.gtool()).unwrap();
        assert_eq!(by_gtool.nb_roots(), 1);
        let graph = InterfaceGraph::new(&model, true).unwrap();
        let by_graph = InterfaceShareFlags::from_graph(&graph).unwrap();
        assert!(by_graph.is_shared(&ents[2]).unwrap());
        assert!(by_graph.model() == model);
        assert_eq!(
            InterfaceShareFlags::new(&model)
                .err()
                .unwrap()
                .exception_type(),
            "Standard_NullObject"
        );
    }
}
