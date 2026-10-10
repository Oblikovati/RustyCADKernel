//! Retained Graphic3d commands distinguish the group style from later primitive styles.
use crate::line_style::primitive_style;
use crate::{PrimitiveDraw, RenderError, error::rejected, primitive::PrimitiveSubmission};
use rustycad_tkernel::{
    ncollection::Vec4,
    standard::{Handle, SharedState, StandardTransient},
};
use rustycad_tkmath::gp::Axis2;
use rustycad_tkservice::graphic3d::*;
use std::sync::Arc;
#[derive(Clone)]
enum GroupCommand {
    Aspects(Handle<dyn Graphic3dAspectsSource>),
    Primitives(PrimitiveSubmission),
    Text(Handle<Graphic3dText>),
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
    text_source: Arc<dyn crate::TextLayoutSource>,
}
rustycad_tkernel::impl_transient!(Graphic3dDrawGroup,"Martensite_Graphic3dDrawGroup",StandardTransient,transient,interfaces:[dyn Graphic3dGroup]);
impl Graphic3dDrawGroup {
    /// Attach to a live Graphic3d structure; `Graphic3dDrawGroup::new(&structure)`.
    pub fn new(structure: &Handle<dyn Graphic3dStructureSource>) -> Handle<Self> {
        Self::with_text_source(structure, Arc::new(crate::FreeTypeTextSource::default()))
    }
    /// Inject font I/O and layout; `Graphic3dDrawGroup::with_text_source(&structure, fonts)`.
    pub fn with_text_source(
        structure: &Handle<dyn Graphic3dStructureSource>,
        text_source: Arc<dyn crate::TextLayoutSource>,
    ) -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            base: Graphic3dGroupBase::new(structure),
            submissions: SharedState::default(),
            text_source,
        })
    }
    /// Extract visible primitives using current styles, buffers and camera; `group.primitive_draws(&camera)?`.
    pub fn primitive_draws(
        &self,
        camera: &Graphic3dCamera,
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        if self.is_deleted() || !self.structure().is_visible() {
            return Ok(vec![]);
        }
        self.draws_in_structure(
            camera,
            None,
            &self
                .structure()
                .cstructure()
                .cstructure_base()
                .fields
                .read(),
            &Graphic3dRenderingParams::new().expect("default rendering parameters"),
            &Handle::default(),
        )
    }
    /// Extract using the actual pixel viewport, including persistence; `group.primitive_draws_for_view(&camera, [800, 600])?`.
    pub fn primitive_draws_for_view(
        &self,
        camera: &Graphic3dCamera,
        extent: [u32; 2],
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        if self.is_deleted() || !self.structure().is_visible() {
            return Ok(vec![]);
        }
        self.draws_in_structure(
            camera,
            Some(extent),
            &self
                .structure()
                .cstructure()
                .cstructure_base()
                .fields
                .read(),
            &Graphic3dRenderingParams::new().expect("default rendering parameters"),
            &Handle::default(),
        )
    }
    /// Compatibility extraction name; `group.triangle_draws(&camera)?` also supports line primitives.
    pub fn triangle_draws(
        &self,
        camera: &Graphic3dCamera,
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        self.primitive_draws(camera)
    }
    /// Compatibility viewport extraction name; `group.triangle_draws_for_view(&camera, [64,64])?`.
    pub fn triangle_draws_for_view(
        &self,
        camera: &Graphic3dCamera,
        extent: [u32; 2],
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        self.primitive_draws_for_view(camera, extent)
    }
    pub(crate) fn draws_in_structure(
        &self,
        camera: &Graphic3dCamera,
        extent: Option<[u32; 2]>,
        structure: &Graphic3dCStructureFields,
        rendering: &Graphic3dRenderingParams,
        global_clipping: &Handle<Graphic3dSequenceOfHClipPlane>,
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        crate::highlight_style::validate_structure(structure)?;
        let (matrix, depth_clamp) =
            crate::persistent_transform::group_clip_transform(self, camera, extent, structure)?;
        let context = crate::text_submission::TextSceneContext {
            group: self,
            camera,
            extent,
            structure,
            rendering,
            fonts: self.text_source.as_ref(),
        };
        let model = crate::persistent_transform::clipping_model(self, structure)?;
        let clipping =
            crate::retained_clipping::structure_clipping(global_clipping, structure, model)?;
        let mut draws =
            self.extract_commands(matrix, structure.highlight, clipping.is_none(), &context)?;
        for draw in &mut draws {
            // OCCT's standard font shader advertises zero clipping planes (3d097a0).
            if draw.textured_style.is_none() {
                draw.clipping = clipping.clone();
            }
            draw.depth_clamp = depth_clamp;
            draw.mirrored = structure
                .trsf
                .get()
                .is_some_and(|datum| datum.transformation().is_negative());
            crate::highlight_style::apply_highlight(draw, structure)?;
        }
        Ok(draws)
    }
    fn extract_commands(
        &self,
        matrix: [f32; 16],
        highlighted: bool,
        allow_culling: bool,
        context: &crate::text_submission::TextSceneContext<'_>,
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        let (mut aspects, commands) = self.retained_commands()?;
        let mut draws = Vec::new();
        for command in commands {
            match command {
                GroupCommand::Aspects(style) => aspects = style,
                GroupCommand::Text(text) => draws.extend(context.draws(&text, &aspects)?),
                GroupCommand::Primitives(primitive) => {
                    let mut lowered = lower_styled_primitive(
                        &primitive,
                        &aspects,
                        self.is_closed() && allow_culling,
                        matrix,
                        highlighted,
                    )?;
                    crate::retained_lighting::apply_surface_lighting(
                        &mut lowered,
                        &primitive,
                        &aspects,
                        context,
                    )?;
                    draws.extend(lowered);
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
    fn retain_primitive(&self, primitive: PrimitiveSubmission, evaluate_bounds: bool) {
        if self.is_deleted() {
            return;
        }
        if evaluate_bounds && let Ok(positions) = primitive.positions() {
            let mut fields = self.base.fields.write();
            // Graphic3d_Group.cxx at 3d097a0 bounds Vec4 inputs by raw xyz, ignoring w.
            for [x, y, z, _] in positions {
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
    fn add_text(&self, text: &Handle<Graphic3dText>, evaluate_bounds: bool) {
        if self.is_deleted() || text.is_null() {
            return;
        }
        // OpenGl_Group::AddText mutates undersized text to the driver's default height.
        if text.height() < 2. {
            text.set_height(16.);
        }
        self.submissions
            .write()
            .commands
            .push(GroupCommand::Text(text.clone()));
        self.record_text_bounds(text, evaluate_bounds);
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

fn lower_styled_primitive(
    primitive: &PrimitiveSubmission,
    aspects: &Handle<dyn Graphic3dAspectsSource>,
    closed: bool,
    matrix: [f32; 16],
    highlighted: bool,
) -> Result<Vec<PrimitiveDraw>, RenderError> {
    let (color, line) = primitive_style(primitive.kind, aspects, closed)?;
    if primitive.kind != Graphic3dTypeOfPrimitiveArray::Points {
        let culling = crate::face_culling::resolve_culling(aspects.face_culling(), closed);
        return primitive.draws(color, matrix, line, None).map(|draws| {
            draws
                .into_iter()
                .map(|draw| draw.with_face_culling(culling))
                .collect()
        });
    }
    if aspects.marker_type() == rustycad_tkservice::aspect::AspectTypeOfMarker::Empty {
        return Ok(vec![]);
    }
    let sprite = crate::point_style::marker_style(&**aspects, highlighted)?;
    primitive.draws(color, matrix, line, Some(sprite))
}
