//! Borrowed specialization references preserve node identity without retaining native owned fields.
use super::{
    AisColoredDrawer, AisShapeDrawerMap,
    shape_drawer_map::find_shape_drawer_node,
    shape_drawer_node::{ShapeDrawerNode, removed_shape_drawer_node},
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use std::sync::{Arc, Weak};
/// A live borrowed native map node; `let node=map.node_ref(&shape)?`.
#[derive(Clone)]
pub struct AisShapeDrawerNodeRef<'a> {
    pub(super) owner: &'a AisShapeDrawerMap,
    pub(super) node: Weak<ShapeDrawerNode>,
}
/// A writable alias to the actual handle-valued node field; `map.change_value_ref(&shape)?.assign(&drawer)?`.
#[derive(Clone)]
pub struct AisShapeDrawerValueMut<'a> {
    reference: AisShapeDrawerNodeRef<'a>,
}
impl AisShapeDrawerMap {
    /// Borrows the actual node found through native IsSame hashing/equality.
    /// Example: `map.node_ref(&shape.reversed())?.orientation()?`.
    pub fn node_ref(&self, key: &Shape) -> Result<AisShapeDrawerNodeRef<'_>, Failure> {
        let nodes = self.nodes.read();
        let node =
            find_shape_drawer_node(&nodes, key).ok_or_else(|| missing_shape_drawer_key(key))?;
        Ok(AisShapeDrawerNodeRef {
            owner: self,
            node: Arc::downgrade(node),
        })
    }
    /// Borrows the current handle slot for assignment without changing the key.
    /// Example: `map.change_value_ref(&shape)?.assign(&replacement)?`.
    pub fn change_value_ref(&self, key: &Shape) -> Result<AisShapeDrawerValueMut<'_>, Failure> {
        let reference = self.node_ref(key)?;
        Ok(AisShapeDrawerValueMut { reference })
    }
}
impl AisShapeDrawerNodeRef<'_> {
    /// Compares allocated native node identity, independent of table/slot movement.
    /// Example: `first.is_same_reference(&map.node_ref(&reversed)?)`.
    pub fn is_same_reference(&self, other: &Self) -> bool {
        let _bounded_owner = self.owner;
        Weak::ptr_eq(&self.node, &other.node)
    }
    /// Reads the originally stored orientation, not that of an equivalent query.
    /// Example: `node.orientation()?`.
    pub fn orientation(&self) -> Result<Orientation, Failure> {
        let node = self.live_node()?;
        node.read_fields(|fields| fields.key.orientation())
    }
    /// Reads a shared TShape flag through the actual original key.
    /// Example: `node.key_is_closed()?`.
    pub fn key_is_closed(&self) -> Result<bool, Failure> {
        let node = self.live_node()?;
        node.read_fields(|fields| fields.key.closed())?
    }
    /// Identifies the actual stored key slot without copying its Shape.
    /// Example: `node.key_reference_address()?`.
    pub fn key_reference_address(&self) -> Result<*const (), Failure> {
        let node = self.live_node()?;
        node.read_fields(|fields| std::ptr::from_ref(fields.key).cast())
    }
    /// Identifies the actual handle slot, preserved by replacement and rehash.
    /// Example: `node.value_reference_address()?`.
    pub fn value_reference_address(&self) -> Result<*const (), Failure> {
        let node = self.live_node()?;
        node.read_fields(|fields| std::ptr::from_ref(fields.drawer).cast())
    }
    /// Checks original key identity and location without cloning its TShape handle.
    /// Example: `node.is_same_key(&shape)?`.
    pub fn is_same_key(&self, key: &Shape) -> Result<bool, Failure> {
        let node = self.live_node()?;
        node.read_fields(|fields| fields.key.is_same(key))
    }
    /// Copies the key only on explicit request; the copy owns one TShape reference.
    /// Example: `let owned_key=node.key_snapshot()?`.
    pub fn key_snapshot(&self) -> Result<Shape, Failure> {
        let node = self.live_node()?;
        node.read_fields(|fields| fields.key.clone())
    }
    /// Reads handle identity without adding target ownership.
    /// Example: `node.drawer_address()?`.
    pub fn drawer_address(&self) -> Result<*const (), Failure> {
        let node = self.live_node()?;
        node.read_fields(|fields| fields.drawer.object_address())
    }
    /// Reads the actual value's intrusive ownership count without making a handle copy.
    /// Example: `node.drawer_ref_count()?`.
    pub fn drawer_ref_count(&self) -> Result<i32, Failure> {
        let node = self.live_node()?;
        node.read_fields(|fields| fields.drawer.ref_count())
    }
    /// Explicitly retains a value snapshot; later slot replacement leaves it unchanged.
    /// Example: `let owned_drawer=node.clone_drawer()?`.
    pub fn clone_drawer(&self) -> Result<Handle<AisColoredDrawer>, Failure> {
        let node = self.live_node()?;
        node.read_fields(|fields| fields.drawer.clone())
    }
    pub(super) fn live_node(&self) -> Result<Arc<ShapeDrawerNode>, Failure> {
        // Weak references cannot postpone native key/value release when aliases remain unused.
        self.node.upgrade().ok_or_else(removed_shape_drawer_node)
    }
}
impl<'a> AisShapeDrawerValueMut<'a> {
    /// Replaces the actual value; const node aliases immediately observe the same slot.
    /// Example: `value.assign(&drawer)?`.
    pub fn assign(&self, value: &Handle<AisColoredDrawer>) -> Result<(), Failure> {
        let node = self.reference.live_node()?;
        node.replace_value(value)
    }
    /// Borrows the same node for read observations; `value.as_const().drawer_address()?`.
    pub fn as_const(&self) -> AisShapeDrawerNodeRef<'a> {
        let reference = &self.reference;
        reference.clone()
    }
}
fn missing_shape_drawer_key(key: &Shape) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("ShapeDrawerMap key={key:?}; expected a bound IsSame key"),
        "",
    )
}
