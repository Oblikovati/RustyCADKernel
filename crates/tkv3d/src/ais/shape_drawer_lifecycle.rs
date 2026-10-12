//! Map-only ColoredShape transitions; the genuine AIS_Shape caller owns presentation invalidation.
use super::{AisColoredDrawer, AisShapeDrawerMap, shape_drawer_map::find_shape_drawer_node};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
impl AisShapeDrawerMap {
    /// Reuses a nonnull custom drawer or allocates/binds one linked to the supplied actual parent.
    /// Example: `let (drawer, created)=map.custom_aspects(&shape, &parent)?`.
    pub fn custom_aspects(
        &self,
        key: &Shape,
        link: &Handle<Prs3dDrawer>,
    ) -> Result<(Handle<AisColoredDrawer>, bool), Failure> {
        if let Some(value) = self.existing_drawer(key)?
            && !value.is_null()
        {
            return Ok((value, false));
        }
        let drawer = Handle::new(AisColoredDrawer::new(link));
        self.bind(key, &drawer)?;
        Ok((drawer, true))
    }
    /// Replaces or unregisters an existing entry, leaving unknown keys untouched.
    /// Example: `let changed=map.unset_custom_aspects(&shape, false, &parent)?`.
    pub fn unset_custom_aspects(
        &self,
        key: &Shape,
        unregister: bool,
        link: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        if !self.is_bound(key) {
            return Ok(false);
        }
        if unregister {
            return Ok(self.unbind(key));
        }
        let drawer = Handle::new(AisColoredDrawer::new(link));
        self.bind(key, &drawer)?;
        Ok(true)
    }
    /// Clears a populated custom map while retaining buckets, reporting whether it changed.
    /// Example: `if map.clear_custom_aspects() { /* actual caller marks its presentations */ }`.
    pub fn clear_custom_aspects(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        self.clear(false);
        true
    }
    fn existing_drawer(&self, key: &Shape) -> Result<Option<Handle<AisColoredDrawer>>, Failure> {
        let nodes = self.nodes.read();
        let Some(node) = find_shape_drawer_node(&nodes, key) else {
            return Ok(None);
        };
        node.read_fields(|fields| Some(fields.drawer.clone()))
    }
}
