//! The structure and the group of the null driver: they keep the groups, the aspects and the
//! highlight state as `OpenGl_Structure` and `OpenGl_Group` do, without graphic resources.
use crate::graphic3d::{
    Graphic3dAspectsReplaceMap, Graphic3dAspectsSource, Graphic3dCStructure,
    Graphic3dCStructureBase, Graphic3dGroup, Graphic3dGroupBase,
    Graphic3dPresentationAttributesSource, Graphic3dStructureManager, Graphic3dStructureSource,
};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::Axis2;

/// A driver structure of the null driver; `NullCStructure::new(&manager)?`.
pub struct NullCStructure {
    transient: StandardTransient,
    base: Graphic3dCStructureBase,
}
rustycad_tkernel::impl_transient!(NullCStructure, "NullCStructure", StandardTransient, transient,
    interfaces: [dyn Graphic3dCStructure]);

impl NullCStructure {
    /// A structure of the manager's driver; `NullCStructure::new(&manager)?`.
    pub fn new(manager: &Handle<Graphic3dStructureManager>) -> Result<Self, Failure> {
        Ok(Self {
            transient: StandardTransient::default(),
            base: Graphic3dCStructureBase::new(manager)?,
        })
    }
}

impl Graphic3dCStructure for NullCStructure {
    fn cstructure_base(&self) -> &Graphic3dCStructureBase {
        &self.base
    }
    fn on_visibility_changed(&self) {}
    /// Drops the groups and the 2D text and highlight flags, as `OpenGl_Structure::Clear`.
    fn clear(&self) {
        let mut fields = self.base.fields.write();
        fields.groups.clear();
        fields.is_2d_text = false;
        fields.is_for_highlight = false;
    }
    fn connect(&self, structure: &dyn Graphic3dCStructure) {
        let _ = structure;
    }
    fn disconnect(&self, structure: &dyn Graphic3dCStructure) {
        let _ = structure;
    }
    fn graphic_highlight(&self, style: &Handle<dyn Graphic3dPresentationAttributesSource>) {
        let mut fields = self.base.fields.write();
        fields.highlight_style = style.clone();
        fields.highlight = true;
    }
    fn graphic_unhighlight(&self) {
        let mut fields = self.base.fields.write();
        fields.highlight = false;
        fields.highlight_style = Handle::default();
    }
    /// A new structure of the manager sharing the groups.
    fn shadow_link(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
        let shadow = NullCStructure::new(manager)?;
        shadow.base.fields.write().groups = self.base.fields.read().groups.clone();
        Ok(Handle::new(shadow).cast())
    }
    fn new_group(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
        let group: Handle<dyn Graphic3dGroup> = Handle::new(NullGroup::new(structure)).cast();
        self.base.fields.write().groups.push(group.clone());
        Ok(group)
    }
    /// Clears the group (without manager update) and removes it from the sequence.
    fn remove_group(&self, group: &Handle<dyn Graphic3dGroup>) {
        let Some(value) = group.get() else { return };
        let index = self
            .base
            .fields
            .read()
            .groups
            .iter()
            .position(|g| g == group);
        if let Some(index) = index {
            value.clear(false);
            self.base.fields.write().groups.remove(index);
        }
    }
}

/// The aspect and option state of a null group.
#[derive(Default)]
struct NullGroupState {
    aspects: Handle<dyn Graphic3dAspectsSource>,
    stencil_test: bool,
}

/// A group of the null driver; `NullGroup::new(&structure)`.
pub struct NullGroup {
    transient: StandardTransient,
    base: Graphic3dGroupBase,
    state: SharedState<NullGroupState>,
}
rustycad_tkernel::impl_transient!(NullGroup, "NullGroup", StandardTransient, transient, interfaces: [dyn Graphic3dGroup]);

impl NullGroup {
    /// A group of a structure; `NullGroup::new(&structure)`.
    pub fn new(structure: &Handle<dyn Graphic3dStructureSource>) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: Graphic3dGroupBase::new(structure),
            state: SharedState::default(),
        }
    }
    /// Whether the stencil test option is on; `group.stencil_test()`.
    pub fn stencil_test(&self) -> bool {
        self.state.read().stencil_test
    }
}

impl Graphic3dGroup for NullGroup {
    fn group_base(&self) -> &Graphic3dGroupBase {
        &self.base
    }
    fn aspects(&self) -> Handle<dyn Graphic3dAspectsSource> {
        self.state.read().aspects.clone()
    }
    fn set_group_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        self.state.write().aspects = aspect.clone();
    }
    fn set_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        self.state.write().aspects = aspect.clone();
    }
    fn synchronize_aspects(&self) {}
    /// Replaces the group aspects when the map has them.
    fn replace_aspects(&self, map: &Graphic3dAspectsReplaceMap) {
        let key = self.state.read().aspects.object_address() as usize;
        if let Some((_, replacement)) = map.get(&key) {
            self.state.write().aspects = replacement.clone();
        }
    }
    fn set_stencil_test_options(&self, is_enabled: bool) {
        self.state.write().stencil_test = is_enabled;
    }
    fn set_flipping_options(&self, is_enabled: bool, ref_plane: &Axis2) {
        let _ = (is_enabled, ref_plane);
    }
}
