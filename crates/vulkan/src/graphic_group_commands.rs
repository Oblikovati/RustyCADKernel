//! Executable native group elements remain separate from retained style.
use crate::{PrimitiveDraw, RenderError, primitive::PrimitiveSubmission};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Axis2;
use rustycad_tkservice::graphic3d::*;
#[derive(Clone)]
pub(super) enum GroupCommand {
    Aspects(Handle<dyn Graphic3dAspectsSource>),
    Flipping(Option<Axis2>),
    Stencil(bool),
    Geometry(GroupGeometry),
}
#[derive(Clone)]
pub(super) enum GroupGeometry {
    Primitives(PrimitiveSubmission),
    Text(Handle<Graphic3dText>),
}
impl GroupCommand {
    pub(super) fn advance(
        self,
        aspects: &mut Handle<dyn Graphic3dAspectsSource>,
        flips: &mut crate::group_flipping::GroupFlipStack,
        program: &mut crate::stencil_program::StencilProgram,
    ) -> Option<GroupGeometry> {
        match self {
            Self::Aspects(style) => advance_aspect(aspects, style),
            Self::Flipping(plane) => advance_flipping(flips, plane),
            Self::Stencil(enabled) => advance_stencil(program, enabled),
            Self::Geometry(geometry) => Some(geometry),
        }
    }
}
impl GroupGeometry {
    pub(super) fn lower(
        self,
        aspects: &Handle<dyn Graphic3dAspectsSource>,
        closed: bool,
        matrix: [f32; 16],
        highlighted: bool,
        context: &crate::text_submission::TextSceneContext<'_>,
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        match self {
            Self::Text(text) => context.draws(&text, aspects).map(|mut draws| {
                crate::text_culling::apply_text_face_culling(&mut draws, &**aspects, closed);
                draws
            }),
            Self::Primitives(primitive) => super::lower_group_surface(
                &primitive,
                aspects,
                closed,
                matrix,
                highlighted,
                context,
            ),
        }
    }
}

fn advance_aspect(
    aspects: &mut Handle<dyn Graphic3dAspectsSource>,
    style: Handle<dyn Graphic3dAspectsSource>,
) -> Option<GroupGeometry> {
    *aspects = style;
    None
}
fn advance_flipping(
    flips: &mut crate::group_flipping::GroupFlipStack,
    plane: Option<Axis2>,
) -> Option<GroupGeometry> {
    flips.apply(plane);
    None
}
fn advance_stencil(
    program: &mut crate::stencil_program::StencilProgram,
    enabled: bool,
) -> Option<GroupGeometry> {
    program.command(crate::stencil_program::StencilCommand::Options(enabled));
    None
}
