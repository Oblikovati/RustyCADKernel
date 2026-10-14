//! Renderer-created groups make ordinary Graphic3d submissions reach the draw stream.
use crate::{Graphic3dDrawGroup, RenderError, error::rejected};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkservice::graphic3d::*;
use std::collections::BTreeSet;
pub(crate) struct RenderStructure {
    transient: StandardTransient,
    base: Graphic3dCStructureBase,
    connections: SharedState<BTreeSet<i32>>,
    this: SharedState<WeakHandle<Self>>,
    parent: Option<Handle<Self>>,
    text_source: std::sync::Arc<dyn crate::TextLayoutSource>,
}
rustycad_tkernel::impl_transient!(RenderStructure, "Martensite_RenderStructure", StandardTransient, transient, interfaces:[dyn Graphic3dCStructure]);
impl RenderStructure {
    pub fn new(
        manager: &Handle<Graphic3dStructureManager>,
        text_source: std::sync::Arc<dyn crate::TextLayoutSource>,
    ) -> Result<Handle<Self>, Failure> {
        Self::with_parent(manager, None, text_source)
    }
    fn with_parent(
        manager: &Handle<Graphic3dStructureManager>,
        parent: Option<Handle<Self>>,
        text_source: std::sync::Arc<dyn crate::TextLayoutSource>,
    ) -> Result<Handle<Self>, Failure> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: Graphic3dCStructureBase::new(manager)?,
            connections: SharedState::default(),
            this: SharedState::default(),
            parent,
            text_source,
        });
        *object.this.write() = object.downgrade();
        object.inherit_shadow_state();
        Ok(object)
    }
    fn inherit_shadow_state(&self) {
        let Some(parent) = &self.parent else {
            return;
        };
        let source = parent.base.fields.read();
        let mut target = self.base.fields.write();
        target.is_infinite = source.is_infinite;
        target.bnd_box = source.bnd_box;
        target.trsf = source.trsf.clone();
        target.trsf_pers = source.trsf_pers.clone();
    }
    fn geometry_batches(&self) -> Result<Vec<Vec<Handle<dyn Graphic3dGroup>>>, RenderError> {
        if !self.connections.read().is_empty() {
            return Err(rejected(
                self.connections.read().clone(),
                "independent or shadow structures until connected rendering is implemented",
            ));
        }
        let mut groups = vec![self.groups()];
        if let Some(parent) = &self.parent {
            groups.extend(parent.geometry_batches()?);
        }
        Ok(groups)
    }
    pub fn draws(
        &self,
        camera: &Graphic3dCamera,
        view_id: i32,
        extent: [u32; 2],
        rendering: &Graphic3dRenderingParams,
        clipping: &Handle<Graphic3dSequenceOfHClipPlane>,
    ) -> Result<crate::stencil_program::StencilProgram, RenderError> {
        if !self.is_visible_in_view(view_id) || self.is_culled() {
            return Ok(Default::default());
        }
        let fields = self.base.fields.read().clone();
        if crate::retained_clipping::structure_is_clipped(clipping, &fields)? {
            return Ok(Default::default());
        }
        crate::retained_capping::CappingScene {
            camera,
            extent,
            fields: &fields,
            rendering,
            clipping,
        }
        .draws(self.geometry_batches()?)
    }
}
impl Graphic3dCStructure for RenderStructure {
    fn cstructure_base(&self) -> &Graphic3dCStructureBase {
        &self.base
    }
    fn on_visibility_changed(&self) {}
    fn clear(&self) {
        for group in self.groups() {
            group.clear(false);
        }
        let mut fields = self.base.fields.write();
        fields.groups.clear();
        fields.is_2d_text = false;
        fields.is_for_highlight = false;
    }
    fn connect(&self, structure: &dyn Graphic3dCStructure) {
        self.connections.write().insert(structure.identification());
    }
    fn disconnect(&self, structure: &dyn Graphic3dCStructure) {
        self.connections.write().remove(&structure.identification());
    }
    fn graphic_highlight(&self, style: &Handle<dyn Graphic3dPresentationAttributesSource>) {
        let mut fields = self.base.fields.write();
        fields.highlight_style = style.clone();
        fields.highlight = true;
    }
    fn graphic_unhighlight(&self) {
        let mut fields = self.base.fields.write();
        fields.highlight_style = Handle::default();
        fields.highlight = false;
    }
    fn shadow_link(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
        // OpenGl_StructureShadow at 3d097a0 flattens a shadow of a shadow to the original parent.
        let parent = self
            .parent
            .clone()
            .unwrap_or_else(|| self.this.read().upgrade());
        Ok(Self::with_parent(manager, Some(parent), self.text_source.clone())?.cast())
    }
    fn new_group(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
        let group: Handle<dyn Graphic3dGroup> =
            Graphic3dDrawGroup::with_text_source(structure, self.text_source.clone()).cast();
        self.base.fields.write().groups.push(group.clone());
        Ok(group)
    }
    fn remove_group(&self, group: &Handle<dyn Graphic3dGroup>) {
        if let Some(value) = group.get() {
            value.clear(false);
        }
        self.base
            .fields
            .write()
            .groups
            .retain(|stored| stored != group);
    }
}
