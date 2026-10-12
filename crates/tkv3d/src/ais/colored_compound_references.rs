//! Const Key/Value aliases track allocated node identity rather than a moving numeric index.
use super::{
    AisColoredDrawer, AisColoredDrawerCompounds,
    colored_compound_map::find_colored_compound,
    colored_compound_node::{ColoredCompoundNode, colored_compound_removed},
};
use rustycad_tkbrep::topods::{Compound, Shape, TopoDSShapeReadSource};
use rustycad_tkernel::standard::{Failure, Handle};
use std::sync::{Arc, Weak};
/// Borrowed allocated-node fields; `let alias=map.node_ref(1)?; map.resize(1000); alias.compound_address()?`.
pub struct AisColoredCompoundRef<'a> {
    owner: &'a AisColoredDrawerCompounds,
    node: Weak<ColoredCompoundNode>,
}
impl AisColoredDrawerCompounds {
    /// Borrow native fields by one-based index; `map.node_ref(1)?`.
    pub fn node_ref(&self, index: usize) -> Result<AisColoredCompoundRef<'_>, Failure> {
        let indexes = self.indexes.read();
        let node = indexes
            .get(index.wrapping_sub(1))
            .ok_or_else(|| colored_compound_index(index, self.len()))?;
        Ok(AisColoredCompoundRef {
            owner: self,
            node: node.clone(),
        })
    }
    /// Find a value without altering the output on a miss; `map.find_into(&drawer,&mut compound)?`.
    pub fn find_into(
        &self,
        drawer: &Handle<AisColoredDrawer>,
        output: &mut Compound,
    ) -> Result<bool, Failure> {
        let nodes = self.nodes.read();
        let Some(node) = find_colored_compound(&nodes, drawer) else {
            return Ok(false);
        };
        let node = Arc::clone(node);
        drop(nodes);
        let replacement = node.compound.clone_compound()?;
        *output = replacement;
        Ok(true)
    }
}
impl AisColoredCompoundRef<'_> {
    pub(super) fn live_node(&self) -> Result<Arc<ColoredCompoundNode>, Failure> {
        let _lifetime = self.owner;
        let node = self.node.upgrade().ok_or_else(colored_compound_removed)?;
        node.check_live()?;
        Ok(node)
    }
    /// Read the actual Key field address; `alias.drawer_address()?`.
    pub fn drawer_address(&self) -> Result<*const (), Failure> {
        let node = self.live_node()?;
        Ok(node.drawer.reference_address())
    }
    /// Read the actual Value field address; `alias.compound_address()?`.
    pub fn compound_address(&self) -> Result<*const (), Failure> {
        let node = self.live_node()?;
        Ok(std::ptr::from_ref(&node.compound).cast())
    }
    /// Explicitly retain a topology snapshot; `let value=alias.clone_compound()?`.
    pub fn clone_compound(&self) -> Result<Compound, Failure> {
        let node = self.live_node()?;
        node.compound.clone_compound()
    }
    /// Lend current topology without acquiring an owning snapshot; `alias.with_shape(|shape|Ok(shape.nb_children()))?`.
    pub fn with_shape<R>(
        &self,
        project: impl FnOnce(&Shape) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        let node = self.live_node()?;
        node.compound.with_shape(project)
    }
    /// Assign the actual native Value slot; `alias.assign_compound(&replacement)?`.
    pub fn assign_compound(&self, value: &Compound) -> Result<(), Failure> {
        let node = self.live_node()?;
        node.compound.assign(value)
    }
    /// Inspect actual compound ownership without an extra snapshot; `alias.compound_ref_count()?`.
    pub fn compound_ref_count(&self) -> Result<i32, Failure> {
        let node = self.live_node()?;
        Ok(node.compound.ref_count())
    }
    /// Inspect actual drawer ownership without retaining it; `alias.drawer_ref_count()?`.
    pub fn drawer_ref_count(&self) -> Result<i32, Failure> {
        let node = self.live_node()?;
        Ok(node.drawer.ref_count())
    }
}
fn colored_compound_index(index: usize, extent: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("colored compound index={index}; expected 1..={extent}"),
        "",
    )
}

impl TopoDSShapeReadSource for AisColoredCompoundRef<'_> {
    fn read_current_shape<R>(
        &self,
        project: impl FnOnce(&Shape) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        let reference = self;
        reference.with_shape(project)
    }
}
