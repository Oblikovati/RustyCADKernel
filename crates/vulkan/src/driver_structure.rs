//! Renderer-created groups make ordinary Graphic3d submissions reach the draw stream.
use crate::{Graphic3dDrawGroup, RenderError, TriangleDraw, error::rejected};
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkservice::graphic3d::*;
use std::collections::BTreeSet;
pub(crate) struct RenderStructure {
    transient: StandardTransient,
    base: Graphic3dCStructureBase,
    connections: SharedState<BTreeSet<i32>>,
}
rustycad_tkernel::impl_transient!(RenderStructure, "Martensite_RenderStructure", StandardTransient, transient, interfaces:[dyn Graphic3dCStructure]);
impl RenderStructure {
    pub fn new(manager: &Handle<Graphic3dStructureManager>) -> Result<Self, Failure> {
        Ok(Self {
            transient: StandardTransient::default(),
            base: Graphic3dCStructureBase::new(manager)?,
            connections: SharedState::default(),
        })
    }
    pub fn draws(
        &self,
        camera: &Graphic3dCamera,
        view_id: i32,
    ) -> Result<Vec<TriangleDraw>, RenderError> {
        if !self.is_visible_in_view(view_id) {
            return Ok(vec![]);
        }
        if !self.connections.read().is_empty() {
            return Err(rejected(
                self.connections.read().clone(),
                "independent structures until connected rendering is implemented",
            ));
        }
        self.groups()
            .iter()
            .map(|group| {
                let retained = group.cast::<Graphic3dDrawGroup>();
                retained
                    .get()
                    .ok_or_else(|| rejected(group.object_address(), "a Martensite draw group"))?
                    .triangle_draws(camera)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|groups| groups.into_iter().flatten().collect())
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
        _: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
        Err(program_error(
            "received shadow structure request; expected implemented Martensite shadow rendering",
        ))
    }
    fn new_group(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
        let group: Handle<dyn Graphic3dGroup> = Graphic3dDrawGroup::new(structure).cast();
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
