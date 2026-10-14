//! Snapshot closed groups once, keeping native structure boundaries for stencil parity.
use crate::{
    Graphic3dDrawGroup, PrimitiveDraw, PrimitiveTopology, RenderCappingPass, RenderClipping,
    RenderError,
    capping_planes::{CapPlane, capping_planes},
    error::rejected,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
struct CapGroup {
    group: Handle<Graphic3dDrawGroup>,
    program: crate::stencil_program::StencilProgram,
}
pub(crate) struct CappingScene<'a> {
    pub camera: &'a Graphic3dCamera,
    pub extent: [u32; 2],
    pub fields: &'a Graphic3dCStructureFields,
    pub rendering: &'a Graphic3dRenderingParams,
    pub clipping: &'a Handle<Graphic3dSequenceOfHClipPlane>,
}
impl CappingScene<'_> {
    pub fn draws(
        &self,
        batches: Vec<Vec<Handle<dyn Graphic3dGroup>>>,
    ) -> Result<crate::stencil_program::StencilProgram, RenderError> {
        let groups = batches
            .into_iter()
            .map(|batch| self.snapshot_batch(batch))
            .collect::<Result<Vec<_>, _>>()?;
        let (caps, initialized) = self.stencil_caps(&groups)?;
        // Ordinary geometry draws the instance before its own groups; capping reverses that order.
        let mut program = crate::stencil_program::StencilProgram::default();
        for group in groups.into_iter().rev().flatten() {
            program.append(group.program);
        }
        program.geometry(caps);
        if initialized {
            program.command(crate::stencil_program::StencilCommand::CappingRestore);
        }
        Ok(program)
    }
    fn stencil_caps(
        &self,
        groups: &[Vec<CapGroup>],
    ) -> Result<(Vec<PrimitiveDraw>, bool), RenderError> {
        let planes = capping_planes(self.clipping, self.fields)?;
        let initialized =
            !planes.is_empty() && groups.iter().flatten().any(|group| group.group.is_closed());
        let mut caps = Vec::new();
        for cap in planes {
            for batch in groups {
                self.append_caps(&mut caps, batch, &cap)?;
            }
        }
        Ok((caps, initialized))
    }
    fn snapshot_batch(
        &self,
        batch: Vec<Handle<dyn Graphic3dGroup>>,
    ) -> Result<Vec<CapGroup>, RenderError> {
        batch
            .into_iter()
            .map(|group| self.snapshot_group(group))
            .collect()
    }
    fn snapshot_group(&self, group: Handle<dyn Graphic3dGroup>) -> Result<CapGroup, RenderError> {
        let retained = group.cast::<Graphic3dDrawGroup>();
        let source = retained
            .get()
            .ok_or_else(|| rejected(group.object_address(), "a Martensite draw group"))?;
        let program = source.program_in_structure(
            self.camera,
            Some(self.extent),
            self.fields,
            self.rendering,
            self.clipping,
        )?;
        Ok(CapGroup {
            group: retained,
            program,
        })
    }
    fn append_caps(
        &self,
        draws: &mut Vec<PrimitiveDraw>,
        groups: &[CapGroup],
        cap: &CapPlane,
    ) -> Result<(), RenderError> {
        let closed: Vec<_> = groups
            .iter()
            .filter(|group| group.group.is_closed())
            .collect();
        if cap.plane.to_use_object_properties() {
            for group in closed {
                self.append_batch(draws, &[group], cap)?;
            }
        } else {
            self.append_batch(draws, &closed, cap)?;
        }
        Ok(())
    }
    fn append_batch(
        &self,
        draws: &mut Vec<PrimitiveDraw>,
        groups: &[&CapGroup],
        cap: &CapPlane,
    ) -> Result<(), RenderError> {
        let mut mask = Vec::new();
        for group in groups {
            mask.extend(self.mask_draws(group, cap)?);
        }
        let Some(first) = mask.first_mut() else {
            return Ok(());
        };
        first.capping_pass = RenderCappingPass::MaskStart;
        let aspect = cap_batch_aspect(cap, groups[0]);
        let surfaces = self.cap_surfaces(cap, &aspect, &groups[0].group)?;
        for draw in &mut mask {
            draw.surface_blend = surfaces[0].surface_blend;
        }
        draws.extend(mask);
        draws.extend(surfaces);
        Ok(())
    }
    fn cap_surfaces(
        &self,
        cap: &CapPlane,
        object: &Handle<dyn Graphic3dAspectsSource>,
        group: &Graphic3dDrawGroup,
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        let aspect = crate::capping_style::cap_aspect(&cap.plane, object);
        let surface = self.cap_surface(cap, &aspect, &group.texture_images)?;
        let mut surfaces = vec![surface];
        self.lower_cap_edges(&mut surfaces, &aspect, group)?;
        Ok(surfaces)
    }
    fn lower_cap_edges(
        &self,
        surfaces: &mut Vec<PrimitiveDraw>,
        aspect: &Handle<dyn Graphic3dAspectsSource>,
        group: &Graphic3dDrawGroup,
    ) -> Result<(), RenderError> {
        let policy = crate::retained_mesh_edges::driver_mesh_edge_pipeline(&group.structure());
        crate::retained_mesh_edges::apply_retained_mesh_edges(
            surfaces,
            &**aspect,
            policy,
            group.texture_images.coordinate_pipeline,
        )?;
        crate::retained_mesh_edges::apply_graphic3d_raster_widths(surfaces, self.rendering)?;
        let projection = crate::persistent_transform::cap_projection_only(
            self.camera,
            self.fields,
            self.extent,
        )?;
        crate::retained_eye_edge_matrix::apply_eye_edge_matrices(surfaces, projection)
    }
    fn mask_draws(
        &self,
        group: &CapGroup,
        cap: &CapPlane,
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        let equation = cap.plane.get_equation().data().map(|v| v as f32);
        let (matrix, model, clamp) =
            crate::persistent_transform::cap_mask_transform(self.camera, self.fields, self.extent)?;
        let clipping = RenderClipping::new(vec![vec![equation]], model)?;
        group
            .program
            .draws
            .iter()
            .filter(|draw| is_cap_geometry(draw) && draw.polygon_edge_style.is_none())
            .map(|draw| {
                let mut mask = draw.clone().with_clipping(clipping.clone());
                crate::capping_mask_aspect::apply_native_stencil_aspect(&mut mask);
                mask.clip_transform = matrix;
                mask.depth_clamp = clamp;
                mask.mirrored = false;
                mask.with_capping_pass(RenderCappingPass::MaskContinue)
            })
            .collect()
    }
    fn cap_surface(
        &self,
        cap: &CapPlane,
        aspect: &Handle<dyn Graphic3dAspectsSource>,
        texture_images: &crate::retained_texture_images::RetainedTextureImages,
    ) -> Result<PrimitiveDraw, RenderError> {
        let color = crate::style::surface_color(aspect, true)?;
        let orientation = crate::capping_geometry::cap_orientation(cap.plane.get_equation().data());
        let (matrix, clamp) = crate::persistent_transform::plane_clip_transform(
            self.camera,
            orientation,
            self.fields,
            self.extent,
        )?;
        let mut draw = cap_draw(color, matrix, aspect)?;
        self.finish_cap_surface(&mut draw, cap, aspect, texture_images, orientation, clamp)?;
        Ok(draw)
    }
    fn finish_cap_surface(
        &self,
        draw: &mut PrimitiveDraw,
        cap: &CapPlane,
        aspect: &Handle<dyn Graphic3dAspectsSource>,
        texture_images: &crate::retained_texture_images::RetainedTextureImages,
        orientation: rustycad_tkernel::ncollection::Mat4<f32>,
        clamp: bool,
    ) -> Result<(), RenderError> {
        crate::surface_alpha::apply_surface_alpha(draw, &**aspect);
        crate::capping_texture::apply_cap_texture(
            draw,
            aspect,
            texture_images,
            self,
            orientation,
            &cap.plane,
        )?;
        crate::capping_lighting::apply_cap_lighting(draw, aspect, self, orientation)?;
        self.finish_clipped_cap(draw, cap, orientation, clamp)
    }
    fn finish_clipped_cap(
        &self,
        draw: &mut PrimitiveDraw,
        cap: &CapPlane,
        orientation: rustycad_tkernel::ncollection::Mat4<f32>,
        clamp: bool,
    ) -> Result<(), RenderError> {
        draw.depth_clamp = clamp;
        draw.clipping = Some(RenderClipping::new(
            cap.remaining.clone(),
            orientation.data(),
        )?);
        crate::highlight_style::apply_highlight(draw, self.fields)
    }
}
fn cap_batch_aspect(cap: &CapPlane, group: &CapGroup) -> Handle<dyn Graphic3dAspectsSource> {
    if cap.plane.to_use_object_properties() {
        return group.group.aspects();
    }
    Handle::default()
}

fn is_cap_geometry(draw: &PrimitiveDraw) -> bool {
    draw.topology() == PrimitiveTopology::Triangles
        && draw.textured_style.is_none()
        && !draw.indices.is_empty()
}
fn cap_draw(
    color: [f32; 4],
    matrix: [f32; 16],
    aspect: &Handle<dyn Graphic3dAspectsSource>,
) -> Result<PrimitiveDraw, RenderError> {
    let vertices = crate::capping_geometry::cap_vertices(color);
    let (mode, factor, units) = aspect.polygon_offsets();
    let offset = crate::RenderPolygonOffset::new(mode, factor, units)?;
    Ok(PrimitiveDraw::new(vertices, (0..12).collect(), matrix)?
        .with_polygon_offset(offset)
        .with_capping_pass(RenderCappingPass::Surface)?
        .with_face_culling(crate::face_culling::resolve_culling(
            aspect.face_culling(),
            crate::face_culling::allow_surface_auto_culling(&**aspect),
        )))
}
