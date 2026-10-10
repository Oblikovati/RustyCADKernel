//! Retained Graphic3d commands distinguish the group style from later primitive styles.
use crate::style::unlit_color;
use crate::{RenderError, TriangleDraw, error::rejected, primitive::PrimitiveSubmission};
use rustycad_tkernel::{
    ncollection::{Mat4, Vec4},
    standard::{Handle, SharedState, StandardTransient},
};
use rustycad_tkmath::gp::Axis2;
use rustycad_tkservice::graphic3d::*;
#[derive(Clone)]
enum GroupCommand {
    Aspects(Handle<dyn Graphic3dAspectsSource>),
    Primitives(PrimitiveSubmission),
}
#[derive(Default)]
struct GroupSubmissionState {
    aspects: Handle<dyn Graphic3dAspectsSource>,
    commands: Vec<GroupCommand>,
    unsupported: Option<&'static str>,
}
/// A retained group feeding Martensite draw extraction; `Graphic3dDrawGroup::new(&structure)`.
pub struct Graphic3dDrawGroup {
    transient: StandardTransient,
    base: Graphic3dGroupBase,
    submissions: SharedState<GroupSubmissionState>,
}
rustycad_tkernel::impl_transient!(Graphic3dDrawGroup,"Martensite_Graphic3dDrawGroup",StandardTransient,transient,interfaces:[dyn Graphic3dGroup]);
impl Graphic3dDrawGroup {
    /// Attach to a live Graphic3d structure; `Graphic3dDrawGroup::new(&structure)`.
    pub fn new(structure: &Handle<dyn Graphic3dStructureSource>) -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            base: Graphic3dGroupBase::new(structure),
            submissions: SharedState::default(),
        })
    }
    /// Extract visible triangles using current styles, buffers and camera; `group.triangle_draws(&camera)?`.
    pub fn triangle_draws(
        &self,
        camera: &Graphic3dCamera,
    ) -> Result<Vec<TriangleDraw>, RenderError> {
        if self.is_deleted() || !self.structure().is_visible() {
            return Ok(vec![]);
        }
        let matrix = self.clip_transform(camera)?;
        let (mut aspects, commands) = self.retained_commands()?;
        let mut draws = Vec::new();
        for command in commands {
            match command {
                GroupCommand::Aspects(style) => aspects = style,
                GroupCommand::Primitives(primitive) => {
                    draws.extend(primitive.draws(unlit_color(&aspects, self.is_closed())?, matrix)?)
                }
            }
        }
        Ok(draws)
    }
    fn retained_commands(
        &self,
    ) -> Result<(Handle<dyn Graphic3dAspectsSource>, Vec<GroupCommand>), RenderError> {
        let state = self.submissions.read();
        if let Some(feature) = state.unsupported {
            return Err(rejected(feature, "an implemented group rendering feature"));
        }
        Ok((state.aspects.clone(), state.commands.clone()))
    }
    fn validate_structure(&self) -> Result<(), RenderError> {
        let parent = self.structure();
        if !self.transform_persistence().is_null() || !parent.transform_persistence().is_null() {
            return Err(rejected(
                "transform persistence",
                "ordinary transforms in this renderer slice",
            ));
        }
        if !parent.clip_planes().is_null() || parent.is_highlighted() {
            return Err(rejected(
                "structure clipping or highlighting",
                "unclipped ordinary structure in this renderer slice",
            ));
        }
        Ok(())
    }
    fn clip_transform(&self, camera: &Graphic3dCamera) -> Result<[f32; 16], RenderError> {
        self.validate_structure()?;
        let mut group = Mat4::<f64>::default();
        self.transformation().get_mat4(&mut group);
        let mut model = Mat4::<f64>::default();
        if let Some(datum) = self.structure().transformation().get() {
            datum.transformation().get_mat4(&mut model);
        }
        Ok(
            (camera.projection_matrix() * camera.orientation_matrix() * model * group)
                .data()
                .map(|v| v as f32),
        )
    }
    fn retain_primitive(&self, primitive: PrimitiveSubmission, evaluate_bounds: bool) {
        if self.is_deleted() {
            return;
        }
        if evaluate_bounds && let Ok(positions) = primitive.positions() {
            let mut fields = self.base.fields.write();
            for [x, y, z] in positions {
                fields.bounds.add(&Vec4::new(x, y, z, 1.));
            }
        }
        self.submissions
            .write()
            .commands
            .push(GroupCommand::Primitives(primitive));
        self.update();
    }
}
impl Graphic3dGroup for Graphic3dDrawGroup {
    fn group_base(&self) -> &Graphic3dGroupBase {
        &self.base
    }
    fn clear(&self, update: bool) {
        self.base.fields.write().bounds.clear();
        *self.submissions.write() = GroupSubmissionState::default();
        if update {
            self.update();
        }
    }
    fn aspects(&self) -> Handle<dyn Graphic3dAspectsSource> {
        self.submissions.read().aspects.clone()
    }
    fn set_group_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        if self.is_deleted() {
            return;
        }
        self.submissions.write().aspects = aspect.clone();
        self.update();
    }
    fn set_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        if self.aspects().is_null() {
            self.set_group_primitives_aspect(aspect);
            return;
        }
        if self.is_deleted() {
            return;
        }
        self.submissions
            .write()
            .commands
            .push(GroupCommand::Aspects(aspect.clone()));
        self.update();
    }
    fn synchronize_aspects(&self) {
        self.update();
    }
    fn replace_aspects(&self, map: &Graphic3dAspectsReplaceMap) {
        let mut state = self.submissions.write();
        replace_style(&mut state.aspects, map);
        for command in &mut state.commands {
            if let GroupCommand::Aspects(style) = command {
                replace_style(style, map);
            }
        }
    }
    fn add_primitive_array_buffers(
        &self,
        kind: Graphic3dTypeOfPrimitiveArray,
        indices: &Handle<dyn Graphic3dIndexBufferSource>,
        attributes: &Handle<dyn Graphic3dBufferSource>,
        bounds: &Handle<Graphic3dBoundBuffer>,
        evaluate_bounds: bool,
    ) {
        self.retain_primitive(
            PrimitiveSubmission {
                kind,
                indices: indices.clone(),
                attributes: attributes.clone(),
                bounds: bounds.clone(),
            },
            evaluate_bounds,
        );
    }
    fn add_text(&self, _: &Handle<Graphic3dText>, _: bool) {
        self.submissions.write().unsupported = Some("text");
    }
    fn set_stencil_test_options(&self, enabled: bool) {
        if enabled {
            self.submissions.write().unsupported = Some("stencil test");
        }
    }
    fn set_flipping_options(&self, enabled: bool, _: &Axis2) {
        if enabled {
            self.submissions.write().unsupported = Some("flipping");
        }
    }
}
fn replace_style(style: &mut Handle<dyn Graphic3dAspectsSource>, map: &Graphic3dAspectsReplaceMap) {
    if let Some((_, replacement)) = map.get(&(style.object_address() as usize)) {
        *style = replacement.clone();
    }
}
