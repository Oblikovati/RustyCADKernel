//! The cap filter intersects reversed siblings while retaining unrelated union chains.
use crate::{
    RenderError,
    retained_clipping::{snapshot_chain, structure_planes},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
pub(crate) struct CapPlane {
    pub plane: Handle<Graphic3dClipPlane>,
    pub remaining: Vec<Vec<[f32; 4]>>,
}
pub(crate) fn capping_planes(
    global: &Handle<Graphic3dSequenceOfHClipPlane>,
    fields: &Graphic3dCStructureFields,
) -> Result<Vec<CapPlane>, RenderError> {
    let planes = structure_planes(global, fields)?;
    let chains = planes
        .iter()
        .cloned()
        .map(snapshot_chain)
        .collect::<Result<Vec<_>, _>>()?;
    let mut caps = Vec::new();
    for (chain, head) in planes.iter().enumerate().filter(|(_, p)| p.is_capping()) {
        append_chain_caps(&mut caps, head.clone(), &chains, chain);
    }
    Ok(caps)
}
fn append_chain_caps(
    caps: &mut Vec<CapPlane>,
    head: Handle<Graphic3dClipPlane>,
    chains: &[Vec<[f32; 4]>],
    chain: usize,
) {
    let mut cursor = head;
    let mut subplane = 0;
    while !cursor.is_null() {
        caps.push(CapPlane {
            plane: cursor.clone(),
            remaining: cap_filter(chains, chain, subplane),
        });
        cursor = cursor.chain_next_plane();
        subplane += 1;
    }
}
fn cap_filter(chains: &[Vec<[f32; 4]>], selected: usize, subplane: usize) -> Vec<Vec<[f32; 4]>> {
    let mut result = Vec::new();
    for (index, chain) in chains.iter().enumerate() {
        if index != selected {
            result.push(chain.clone());
            continue;
        }
        result.extend(
            chain
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != subplane)
                .map(|(_, equation)| vec![equation.map(|value| -value)]),
        );
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sibling_halfspaces_are_reversed_and_intersected_not_unioned() {
        let a = [1., 0., 0., 2.];
        let b = [0., 1., 0., 3.];
        let c = [0., 0., 1., 4.];
        let other = vec![a, b];
        assert_eq!(
            cap_filter(&[vec![a, b, c], other.clone()], 0, 1),
            vec![vec![a.map(|v| -v)], vec![c.map(|v| -v)], other]
        );
        assert!(cap_filter(&[vec![a]], 0, 0).is_empty());
    }
}
