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
    draws: Vec<PrimitiveDraw>,
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
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        let groups = batches
            .into_iter()
            .map(|batch| self.snapshot_batch(batch))
            .collect::<Result<Vec<_>, _>>()?;
        let mut caps = Vec::new();
        for cap in capping_planes(self.clipping, self.fields)? {
            for batch in &groups {
                self.append_caps(&mut caps, batch, &cap)?;
            }
        }
        // Ordinary geometry draws the instance before its own groups; capping reverses that order.
        let mut draws: Vec<_> = groups
            .into_iter()
            .rev()
            .flatten()
            .flat_map(|group| group.draws)
            .collect();
        draws.extend(caps);
        Ok(draws)
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
        let draws = source.draws_in_structure(
            self.camera,
            Some(self.extent),
            self.fields,
            self.rendering,
            self.clipping,
        )?;
        Ok(CapGroup {
            group: retained,
            draws,
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
        let aspect = if cap.plane.to_use_object_properties() {
            groups[0].group.aspects()
        } else {
            Handle::default()
        };
        let surface = self.cap_surface(cap, &aspect)?;
        draws.extend(mask);
        draws.push(surface);
        Ok(())
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
            .draws
            .iter()
            .filter(|draw| is_cap_geometry(draw))
            .map(|draw| {
                let mut mask = draw.clone().with_clipping(clipping.clone());
                mask.lighting = None;
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
        group: &Handle<dyn Graphic3dAspectsSource>,
    ) -> Result<PrimitiveDraw, RenderError> {
        let aspect = crate::capping_style::cap_aspect(&cap.plane, group);
        let color = crate::style::surface_color(&aspect, true)?;
        let orientation = crate::capping_geometry::cap_orientation(cap.plane.get_equation().data());
        let (matrix, clamp) = crate::persistent_transform::plane_clip_transform(
            self.camera,
            orientation,
            self.fields,
            self.extent,
        )?;
        let mut draw = cap_draw(color, matrix, &aspect)?;
        crate::capping_lighting::apply_cap_lighting(&mut draw, &aspect, self, orientation)?;
        draw.depth_clamp = clamp;
        draw = draw.with_clipping(RenderClipping::new(
            cap.remaining.clone(),
            orientation.data(),
        )?);
        crate::highlight_style::apply_highlight(&mut draw, self.fields)?;
        Ok(draw)
    }
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
            true,
        )))
}
