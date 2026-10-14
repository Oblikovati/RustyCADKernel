//! One spatial snapshot feeds the ordered native element program.
use super::{Graphic3dDrawGroup, GroupCommand, GroupGeometry, apply_command_clipping};
use crate::{PrimitiveDraw, RenderClipping, RenderError, stencil_program::StencilProgram};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
struct GroupSpatialSnapshot {
    matrix: [f32; 16],
    depth_clamp: bool,
    clipping: Option<RenderClipping>,
}
struct GroupDrawPolicy {
    highlighted: bool,
    allow_culling: bool,
}
impl GroupDrawPolicy {
    fn new(highlighted: bool, allow_culling: bool) -> Self {
        Self {
            highlighted,
            allow_culling,
        }
    }
}
impl Graphic3dDrawGroup {
    pub(crate) fn program_in_structure(
        &self,
        camera: &Graphic3dCamera,
        extent: Option<[u32; 2]>,
        structure: &Graphic3dCStructureFields,
        rendering: &Graphic3dRenderingParams,
        global_clipping: &Handle<Graphic3dSequenceOfHClipPlane>,
    ) -> Result<StencilProgram, RenderError> {
        let spatial = self.spatial_snapshot(camera, extent, structure, global_clipping)?;
        let context = self.stencil_text_context(
            (camera, extent),
            structure,
            rendering,
            spatial.clipping.as_ref(),
        );
        let policy = GroupDrawPolicy::new(structure.highlight, spatial.clipping.is_none());
        let mut program = self.extract_commands(spatial.matrix, policy, &context)?;
        finish_group_spatial_state(&mut program.draws, spatial.depth_clamp, structure)?;
        Ok(program)
    }
    fn spatial_snapshot(
        &self,
        camera: &Graphic3dCamera,
        extent: Option<[u32; 2]>,
        structure: &Graphic3dCStructureFields,
        global_clipping: &Handle<Graphic3dSequenceOfHClipPlane>,
    ) -> Result<GroupSpatialSnapshot, RenderError> {
        crate::highlight_style::validate_structure(structure)?;
        let (matrix, depth_clamp) =
            crate::persistent_transform::group_clip_transform(self, camera, extent, structure)?;
        let model = crate::persistent_transform::clipping_model(self, structure)?;
        let clipping =
            crate::retained_clipping::structure_clipping(global_clipping, structure, model)?;
        Ok(GroupSpatialSnapshot {
            matrix,
            depth_clamp,
            clipping,
        })
    }
    fn stencil_text_context<'a>(
        &'a self,
        projection: (&'a Graphic3dCamera, Option<[u32; 2]>),
        structure: &'a Graphic3dCStructureFields,
        rendering: &'a Graphic3dRenderingParams,
        clipping: Option<&'a RenderClipping>,
    ) -> crate::text_submission::TextSceneContext<'a> {
        crate::text_submission::TextSceneContext {
            group: self,
            camera: projection.0,
            extent: projection.1,
            structure,
            rendering,
            fonts: self.text_source.as_ref(),
            model_override: None,
            clipping,
        }
    }
    fn extract_commands(
        &self,
        matrix: [f32; 16],
        policy: GroupDrawPolicy,
        context: &crate::text_submission::TextSceneContext<'_>,
    ) -> Result<StencilProgram, RenderError> {
        let (mut aspects, commands) = self.retained_commands();
        let mut program = StencilProgram::default();
        let mut flips = crate::group_flipping::GroupFlipStack::scene(context, matrix)?;
        for command in commands {
            let Some(geometry) = command.advance(&mut aspects, &mut flips, &mut program) else {
                continue;
            };
            let text = matches!(geometry, GroupGeometry::Text(_));
            let draws =
                self.lower_ordered_geometry(geometry, &aspects, &flips, &policy, context)?;
            program.ordered_geometry(draws, text);
        }
        Ok(program)
    }
    fn lower_ordered_geometry(
        &self,
        geometry: GroupGeometry,
        aspects: &Handle<dyn Graphic3dAspectsSource>,
        flips: &crate::group_flipping::GroupFlipStack,
        policy: &GroupDrawPolicy,
        context: &crate::text_submission::TextSceneContext<'_>,
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        let mut active = *context;
        active.model_override = flips.override_model();
        let mut lowered = geometry.lower(
            aspects,
            self.is_closed() && policy.allow_culling,
            flips.clip(),
            policy.highlighted,
            &active,
        )?;
        apply_command_clipping(&mut lowered, flips, context.clipping)?;
        Ok(lowered)
    }
    fn retained_commands(&self) -> (Handle<dyn Graphic3dAspectsSource>, Vec<GroupCommand>) {
        let state = self.submissions.read();
        (state.aspects.clone(), state.commands.clone())
    }
}
fn finish_group_spatial_state(
    draws: &mut [PrimitiveDraw],
    clamp: bool,
    structure: &Graphic3dCStructureFields,
) -> Result<(), RenderError> {
    for draw in draws {
        draw.depth_clamp = clamp;
        draw.mirrored = structure
            .trsf
            .get()
            .is_some_and(|datum| datum.transformation().is_negative());
        crate::highlight_style::apply_highlight(draw, structure)?;
    }
    Ok(())
}
