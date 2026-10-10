//! Resolve view/local ownership before snapshotting native world-space clip chains.
use crate::{RenderClipping, RenderError, error::rejected};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;

pub(crate) fn structure_clipping(
    global: &Handle<Graphic3dSequenceOfHClipPlane>,
    structure: &Graphic3dCStructureFields,
    model: [f32; 16],
) -> Result<Option<RenderClipping>, RenderError> {
    let local = &structure.clip_planes;
    let override_global = local
        .get()
        .is_some_and(|planes| planes.to_override_global());
    let mut chains = if override_global {
        Vec::new()
    } else {
        plane_chains(global)?
    };
    chains.extend(plane_chains(local)?);
    if chains.is_empty() {
        return Ok(None);
    }
    if !structure.trsf_pers.is_null() {
        return Err(rejected(
            structure.trsf_pers.mode(),
            "ordinary structure transforms until persistent clipping is verified",
        ));
    }
    RenderClipping::new(chains, model).map(Some)
}
fn plane_chains(
    planes: &Handle<Graphic3dSequenceOfHClipPlane>,
) -> Result<Vec<Vec<[f32; 4]>>, RenderError> {
    let Some(planes) = planes.get() else {
        return Ok(vec![]);
    };
    let mut chains = Vec::new();
    for plane in planes.items() {
        let head = plane
            .get()
            .ok_or_else(|| rejected("null clip plane", "a non-null sequence member"))?;
        if !head.is_on() {
            continue;
        }
        if head.is_capping() {
            return Err(rejected(
                head.get_id(),
                "clipping without capping until the cap pass is implemented",
            ));
        }
        chains.push(snapshot_chain(plane)?);
    }
    Ok(chains)
}
fn snapshot_chain(mut cursor: Handle<Graphic3dClipPlane>) -> Result<Vec<[f32; 4]>, RenderError> {
    let mut equations = Vec::new();
    while let Some(plane) = cursor.get() {
        let equation = plane.get_equation();
        equations.push([
            equation.x as f32,
            equation.y as f32,
            equation.z as f32,
            equation.w as f32,
        ]);
        cursor = plane.chain_next_plane();
    }
    Ok(equations)
}
